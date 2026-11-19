// Turns one function's bytecode into statements by replaying Luau's register stack.
//
// A value written to a free register is pending: not yet placed anywhere in the output.
// Luau evaluates an expression's operands into consecutive temporaries and consumes them
// last-in-first-out, so when an instruction reads exactly the most recent pending values,
// in the order they were created, they fold into its expression. Anything else (a value
// read out of order, or still pending when a statement runs) is turned into a `local`
// first, oldest first. Either way every value is evaluated in the order the bytecode
// computed it, which is what keeps the output's side effects in the original order.

use crate::{
    Error,
    ast::{BinOp, Capture, Closure, Expr, Field, Function, Stmt, UnOp},
    bytecode::{Chunk, Constant, Insn, Op, Proto},
    liveness::{self, Regs},
    print::is_identifier,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Free,
    Local(u32),
    Pending(u64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Span {
    One,
    // a fixed number of results from one call or `...`
    Many(u8),
    // every result, up to the top of the stack
    All,
}

#[derive(Debug)]
enum Value {
    Expr(Expr),
    // fields in the order their stores ran (Luau flushes positional values before each keyed
    // one, so that is the source order), and how many positional values there are so far
    Table(Vec<Field>, u32),
    // NAMECALL's method and self, waiting for their CALL
    Method(Expr, Vec<u8>),
}

#[derive(Debug)]
struct Pending {
    id: u64,
    reg: u8,
    span: Span,
    value: Value,
}

enum Operand {
    Expr(Expr),
    Method(Expr, Vec<u8>),
}

impl Operand {
    fn expr(self) -> Expr {
        match self {
            Operand::Expr(e) => e,
            Operand::Method(..) => unreachable!("methods are only read by CALL"),
        }
    }
}

struct Fastcall {
    call_pc: usize,
    // pending values created before the FASTCALL keep their order; the fallback code after
    // it only loads locals, constants and the callee, so those can come in any order
    first_after: u64,
}

struct Lifter<'a> {
    proto_id: usize,
    proto: &'a Proto,
    chunk: &'a Chunk,
    slots: Vec<Slot>,
    pending: Vec<Pending>,
    next_id: u64,
    body: Vec<Stmt>,
    names: Vec<Option<Vec<u8>>>,
    // a local a closure captured by value is never assigned again in the source, so a later
    // write to its register is a new local
    by_value: Vec<bool>,
    // a local a closure captured by reference: every later write is an assignment it sees
    // (until CLOSEUPVALS closes it)
    by_ref: Vec<bool>,
    // captured by reference at any point: some closure may assign it
    ever_by_ref: Vec<bool>,
    fastcall: Option<Fastcall>,
    // the NAMECALL waiting for its CALL: base register and method name
    namecall: Option<(u8, Vec<u8>)>,
    pc: usize,
    at: usize,
    live: Vec<Regs>,
    insns: Vec<Insn>,
    trace: bool,
}

pub fn lift(chunk: &Chunk, proto_id: usize) -> Result<Function, Error> {
    let proto = &chunk.protos[proto_id];
    let mut lifter = Lifter {
        proto_id,
        proto,
        chunk,
        slots: vec![Slot::Free; 256],
        pending: Vec::new(),
        next_id: 0,
        body: Vec::new(),
        names: Vec::new(),
        by_value: Vec::new(),
        by_ref: Vec::new(),
        ever_by_ref: Vec::new(),
        fastcall: None,
        namecall: None,
        pc: 0,
        at: 0,
        live: Vec::new(),
        insns: Vec::new(),
        trace: std::env::var_os("SUNBERRY_TRACE").is_some(),
    };
    let params = (0..proto.num_params)
        .map(|reg| {
            let local = lifter.new_local(reg, 0);
            lifter.slots[reg as usize] = Slot::Local(local);
            local
        })
        .collect();

    let insns = proto.insns();
    lifter.live = liveness::live_after(&insns);
    lifter.insns = insns.clone();
    let trace = lifter.trace;
    let mut i = 0;
    while i < insns.len() {
        lifter.pc = insns[i].pc;
        lifter.at = i;
        if trace {
            let stack = lifter.pending.iter().map(|p| format!("R{}{:?}", p.reg, p.span)).collect::<Vec<_>>();
            eprintln!("f{proto_id} {:>4} {:<12} pending [{}]", insns[i].pc, insns[i].op.name(), stack.join(" "));
        }
        lifter.scope_changes()?;
        i += lifter.step(&insns, i)?;
    }
    lifter.flush()?;
    let body = pin_constants(lifter.body, &lifter.ever_by_ref);
    Ok(Function { params, is_vararg: proto.is_vararg, body, names: lifter.names })
}

// From O1 up, Luau folds a local that is never assigned and starts as a constant into every
// use, and folds what it can at compile time, where `x ^ 3` is pow() instead of the VM's
// x*x*x. A local the bytecode keeps in a register and reads wasn't folded, so the source
// assigned it somewhere (or was O0, where nothing folds). The output must not be more
// foldable than that: such a local is declared bare and then assigned.
fn pin_constants(body: Vec<Stmt>, by_ref: &[bool]) -> Vec<Stmt> {
    let mut assigned = vec![false; by_ref.len()];
    for stmt in &body {
        if let Stmt::Assign { targets, .. } = stmt {
            for target in targets {
                if let Expr::Local(l) = target {
                    assigned[*l as usize] = true;
                }
            }
        }
    }
    let mut out = Vec::with_capacity(body.len());
    for stmt in body {
        match stmt {
            Stmt::Local { locals, values }
                if locals.len() == 1
                    && values.len() == 1
                    && !assigned[locals[0] as usize]
                    && !by_ref[locals[0] as usize]
                    && foldable(&values[0]) =>
            {
                let local = locals[0];
                out.push(Stmt::Local { locals, values: Vec::new() });
                out.push(Stmt::Assign { targets: vec![Expr::Local(local)], values });
            }
            stmt => out.push(stmt),
        }
    }
    out
}

// what Luau's constant folding might turn into a constant (an overestimate is only uglier):
// literals, operators on them, and library members and builtin calls of them
fn foldable(expr: &Expr) -> bool {
    match expr {
        Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_) | Expr::Global(_) => true,
        Expr::Unary(_, e) => foldable(e),
        Expr::Binary(_, a, b) => foldable(a) && foldable(b),
        Expr::Index(t, k) => matches!(t.as_ref(), Expr::Global(_)) && foldable(k),
        Expr::Call { func, args, .. } => foldable(func) && args.iter().all(foldable),
        _ => false,
    }
}

impl Lifter<'_> {
    fn unsupported(&self, what: impl Into<String>) -> Error {
        Error::Unsupported { proto: self.proto_id, pc: self.pc, what: what.into() }
    }

    fn new_local(&mut self, reg: u8, pc: usize) -> u32 {
        let name = self
            .proto
            .locals
            .iter()
            .filter(|l| l.register == reg && l.start_pc <= pc + 1 && pc < l.end_pc.max(l.start_pc + 1))
            .max_by_key(|l| l.start_pc)
            .map(|l| l.name.clone());
        self.names.push(name);
        self.by_value.push(false);
        self.by_ref.push(false);
        self.ever_by_ref.push(false);
        (self.names.len() - 1) as u32
    }

    // debug info (level 2) says exactly where locals live: a local starting here means what
    // its register holds is declared now, and one ending frees its register
    fn scope_changes(&mut self) -> Result<(), Error> {
        let pc = self.pc;
        let starting = self
            .proto
            .locals
            .iter()
            .filter(|l| l.start_pc == pc)
            .filter_map(|l| match self.slots[l.register as usize] {
                Slot::Pending(id) => Some((self.index_of(id), l.register)),
                _ => None,
            })
            .max();
        if let Some((index, reg)) = starting {
            self.flush_keeping(index + 1, &[reg])?;
        }
        // `local b = a` can share a's register when neither is assigned again (an alias),
        // so a local ending only frees its register if no other one still holds it
        for l in &self.proto.locals {
            let reg = l.register;
            let still_held = self.proto.locals.iter().any(|o| o.register == reg && o.start_pc <= pc && pc < o.end_pc);
            if l.end_pc == pc && l.start_pc < pc && !still_held && matches!(self.slots[reg as usize], Slot::Local(_)) {
                self.slots[reg as usize] = Slot::Free;
            }
        }
        Ok(())
    }

    fn starts_local(&self, reg: u8) -> bool {
        self.proto.locals.iter().any(|l| l.register == reg && l.start_pc == self.pc + 1)
    }

    // Whether a write to `reg` is an assignment to the local it holds. In straight-line code
    // a new local is always equivalent unless a closure captured the old one by reference
    // and would see the assignment; debug info says which one the source had.
    // TODO(M2): inside branches a write can only be a new local if no path reads the old
    // variable afterwards.
    fn assigns(&self, reg: u8) -> Option<u32> {
        let Slot::Local(l) = self.slots[reg as usize] else { return None };
        if self.by_value[l as usize] || self.starts_local(reg) {
            return None;
        }
        let continues = self.proto.locals.iter().any(|v| v.register == reg && v.start_pc <= self.pc && self.pc < v.end_pc);
        (self.by_ref[l as usize] || continues).then_some(l)
    }

    // a register about to take a new value: a pending value still in it is never read, but
    // still has to be evaluated, so it becomes a local (with everything older) first
    fn clear(&mut self, reg: u8) -> Result<(), Error> {
        if let Slot::Pending(id) = self.slots[reg as usize] {
            let index = self.index_of(id);
            self.flush_keeping(index + 1, &[reg])?;
        }
        self.release(reg);
        Ok(())
    }

    // a register about to take a new value: free unless the write assigns a local
    fn release(&mut self, reg: u8) {
        if let Slot::Local(_) = self.slots[reg as usize]
            && self.assigns(reg).is_none()
        {
            self.slots[reg as usize] = Slot::Free;
        }
    }

    fn push(&mut self, reg: u8, span: Span, value: Value) -> Result<(), Error> {
        let width = match span {
            Span::One => 1,
            Span::Many(n) => n as usize,
            Span::All => 1,
        };
        for r in reg as usize..reg as usize + width {
            self.clear(r as u8)?;
            if self.slots[r] != Slot::Free {
                return Err(self.unsupported(format!("R{r} is still in use")));
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        for r in reg as usize..reg as usize + width {
            self.slots[r] = Slot::Pending(id);
        }
        if let Value::Method(..) = value {
            self.slots[reg as usize + 1] = Slot::Pending(id);
        }
        self.pending.push(Pending { id, reg, span, value });
        Ok(())
    }

    fn index_of(&self, id: u64) -> usize {
        self.pending.iter().position(|p| p.id == id).expect("slots only name live pending values")
    }

    // every pending value becomes a local, oldest first
    #[track_caller]
    fn flush(&mut self) -> Result<(), Error> {
        self.flush_oldest(self.pending.len())
    }

    // the `count` oldest pending values become locals; evaluating old values earlier never
    // reorders anything, so whatever stays pending can still fold later
    #[track_caller]
    fn flush_oldest(&mut self, count: usize) -> Result<(), Error> {
        self.flush_keeping(count, &[])
    }

    // before a statement: a table under construction that only holds constants can still
    // wait, as nothing can reach it yet and the statement can't change a constant
    #[track_caller]
    fn flush_for_statement(&mut self) -> Result<(), Error> {
        let constant = |f: &Field| {
            let c = |e: &Expr| matches!(e, Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_));
            match f {
                Field::Positional(v) => c(v),
                Field::Keyed(k, v) => c(k) && c(v),
            }
        };
        let keep = self
            .pending
            .iter()
            .filter(|p| matches!(&p.value, Value::Table(fields, _) if fields.iter().all(constant)))
            .map(|p| p.id)
            .collect::<Vec<_>>();
        self.flush_except(self.pending.len(), &keep)
    }

    // Making a table has no effects of its own, so when values have to become locals only to
    // keep their order, a table under construction whose fields so far are pure can wait and
    // stay a constructor (unless it is one of `reading`, the registers being read now).
    #[track_caller]
    fn flush_keeping(&mut self, count: usize, reading: &[u8]) -> Result<(), Error> {
        let keep = if reading.is_empty() {
            Vec::new()
        } else {
            self.pending[..count]
                .iter()
                .filter(|p| !reading.contains(&p.reg))
                .filter(|p| matches!(&p.value, Value::Table(fields, _) if fields.iter().all(|f| self.pure_field(f))))
                .map(|p| p.id)
                .collect()
        };
        self.flush_except(count, &keep)
    }

    #[track_caller]
    fn flush_except(&mut self, count: usize, keep: &[u64]) -> Result<(), Error> {
        if self.trace && count > keep.len() {
            eprintln!("       flush {count} keeping {} from line {}", keep.len(), std::panic::Location::caller().line());
        }
        let rest = self.pending.split_off(count);
        let mut kept = Vec::new();
        for pending in std::mem::replace(&mut self.pending, rest) {
            if keep.contains(&pending.id) {
                kept.push(pending);
                continue;
            }
            let width = match pending.span {
                Span::One => 1,
                Span::Many(n) => n,
                Span::All => return Err(self.unsupported("a value list can't become a local")),
            };
            let value = match pending.value {
                Value::Expr(e) => e,
                Value::Table(fields, _) => table(fields),
                // the object becomes a local; CALL still knows the method from `namecall`
                Value::Method(object, _) => {
                    self.slots[pending.reg as usize + 1] = Slot::Free;
                    object
                }
            };
            let locals = (0..width)
                .map(|i| {
                    let reg = pending.reg + i;
                    let local = self.new_local(reg, self.pc);
                    self.slots[reg as usize] = Slot::Local(local);
                    local
                })
                .collect();
            self.body.push(Stmt::Local { locals, values: vec![value] });
        }
        kept.append(&mut self.pending);
        self.pending = kept;
        Ok(())
    }

    fn pure_field(&self, field: &Field) -> bool {
        let pure = |e: &Expr| match e {
            Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_) => true,
            Expr::Local(l) => !self.by_ref[*l as usize],
            _ => false,
        };
        match field {
            Field::Positional(v) => pure(v),
            Field::Keyed(k, v) => pure(k) && pure(v),
        }
    }

    fn emit(&mut self, stmt: Stmt) -> Result<(), Error> {
        // nothing may follow a return in its block
        if let Stmt::Return(_) = stmt {
            self.flush()?;
        } else {
            self.flush_for_statement()?;
        }
        self.body.push(stmt);
        Ok(())
    }

    // which pending values `regs` read, if they are the newest ones in creation order
    fn foldable(&self, regs: &[u8]) -> Option<Vec<usize>> {
        let mut used = Vec::new();
        let mut skip_to = 0usize;
        for &reg in regs {
            if (reg as usize) < skip_to {
                continue;
            }
            if let Slot::Pending(id) = self.slots[reg as usize] {
                let index = self.index_of(id);
                let p = &self.pending[index];
                if p.reg != reg || matches!(p.span, Span::Many(_)) || used.contains(&index) {
                    return None;
                }
                if let Value::Method(..) = p.value {
                    skip_to = reg as usize + 2;
                }
                used.push(index);
            }
        }
        let first = self.pending.len() - used.len();
        let mut sorted = used.clone();
        sorted.sort_unstable();
        if sorted.iter().enumerate().any(|(i, &index)| index != first + i) {
            return None;
        }
        let in_order = match &self.fastcall {
            Some(f) if f.call_pc == self.pc => {
                let early = used.iter().filter(|&&i| self.pending[i].id < f.first_after).collect::<Vec<_>>();
                early.windows(2).all(|w| w[0] < w[1])
            }
            _ => used.windows(2).all(|w| w[0] < w[1]),
        };
        in_order.then_some(used)
    }

    // the values `regs` hold, in order; a NAMECALL pair counts as one operand
    fn take(&mut self, regs: &[u8]) -> Result<Vec<Operand>, Error> {
        // a value read again later can't be folded into this reader: it becomes a local
        // (a register this instruction overwrites is read later only as its new value)
        let live = self.live[self.at];
        let overwritten = liveness::writes(&self.insns[self.at]);
        let reused = regs
            .iter()
            .filter(|&&r| live.has(r) && !overwritten.has(r))
            .filter_map(|&r| match self.slots[r as usize] {
                Slot::Pending(id) => Some(self.index_of(id)),
                _ => None,
            })
            .max();
        if let Some(index) = reused {
            self.flush_keeping(index + 1, regs)?;
        }
        // Luau reads a local operand where the operation runs, not where it appears, so a copy
        // of a local some closure can assign (captured by reference) can't fold past
        // anything evaluated after it: that might call the closure. It stays a snapshot.
        loop {
            let mut used = regs
                .iter()
                .filter_map(|&r| match self.slots[r as usize] {
                    Slot::Pending(id) => Some(self.index_of(id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            used.sort_unstable();
            used.dedup();
            let newest = used.last().copied();
            let volatile = used.iter().copied().filter(|&i| Some(i) != newest).filter(|&i| {
                matches!(&self.pending[i].value, Value::Expr(Expr::Local(l)) if self.by_ref[*l as usize])
            }).max();
            match volatile {
                Some(index) => {
                    let before = self.pending.len();
                    self.flush_keeping(index + 1, regs)?;
                    if self.pending.len() == before {
                        break;
                    }
                }
                None => break,
            }
        }
        while self.foldable(regs).is_none() {
            let used = regs
                .iter()
                .filter_map(|&r| match self.slots[r as usize] {
                    Slot::Pending(id) => Some(self.index_of(id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            // the oldest operand read out of order becomes a local with everything older;
            // what stays pending keeps its creation order, so it still evaluates in place
            let Some(&oldest) = used.iter().min() else {
                return Err(self.unsupported("operands can't be folded"));
            };
            let before = self.pending.len();
            self.flush_keeping(oldest + 1, regs)?;
            if self.pending.len() == before {
                return Err(self.unsupported("operands can't be folded"));
            }
        }
        let mut out = Vec::new();
        let mut skip_to = 0usize;
        for &reg in regs {
            if (reg as usize) < skip_to {
                continue;
            }
            match self.slots[reg as usize] {
                Slot::Local(l) => out.push(Operand::Expr(Expr::Local(l))),
                Slot::Free => return Err(self.unsupported(format!("R{reg} is read before it is written"))),
                Slot::Pending(id) => {
                    let index = self.index_of(id);
                    let pending = self.pending.remove(index);
                    self.slots[reg as usize] = Slot::Free;
                    out.push(match pending.value {
                        Value::Expr(e) => Operand::Expr(e),
                        Value::Table(fields, _) => Operand::Expr(table(fields)),
                        Value::Method(object, method) => {
                            self.slots[reg as usize + 1] = Slot::Free;
                            skip_to = reg as usize + 2;
                            Operand::Method(object, method)
                        }
                    });
                }
            }
        }
        Ok(out)
    }

    fn take_exprs(&mut self, regs: &[u8]) -> Result<Vec<Expr>, Error> {
        self.take(regs)?.into_iter().map(|o| match o {
            Operand::Expr(e) => Ok(e),
            Operand::Method(..) => Err(self.unsupported("NAMECALL result read as a value")),
        }).collect()
    }

    fn take1(&mut self, reg: u8) -> Result<Expr, Error> {
        Ok(self.take_exprs(&[reg])?.pop().unwrap())
    }

    // the registers from `from` up to the start of the open value list on top
    fn to_top(&self, from: u8) -> Result<Vec<u8>, Error> {
        // after a FASTCALL the callee is loaded below the list, so it needn't be the newest
        match self.pending.iter().rev().find(|p| p.span == Span::All) {
            Some(p) if p.reg >= from => Ok((from..=p.reg).collect()),
            _ => Err(self.unsupported("expected an open value list")),
        }
    }

    fn write(&mut self, reg: u8, expr: Expr) -> Result<(), Error> {
        if let Slot::Pending(id) = self.slots[reg as usize] {
            let index = self.index_of(id);
            self.flush_keeping(index + 1, &[reg])?;
        }
        match self.assigns(reg) {
            Some(l) => self.emit(Stmt::Assign { targets: vec![Expr::Local(l)], values: vec![expr] }),
            None => {
                self.slots[reg as usize] = Slot::Free;
                self.push(reg, Span::One, Value::Expr(expr))
            }
        }
    }

    fn constant(&self, index: u32) -> Result<Expr, Error> {
        match self.proto.constants.get(index as usize) {
            Some(Constant::Nil) => Ok(Expr::Nil),
            Some(Constant::Boolean(b)) => Ok(Expr::Bool(*b)),
            Some(Constant::Number(n)) if n.is_nan() && !matches!(n.to_bits(), 0xfff8_0000_0000_0000 | 0x7ff8_0000_0000_0000) => {
                Err(self.unsupported(format!("NaN constant with payload {:#x}", n.to_bits())))
            }
            Some(Constant::Number(n)) => Ok(Expr::Number(*n)),
            Some(Constant::String(s)) => Ok(Expr::String(s.clone())),
            Some(Constant::Vector(v)) if v[3] == 0.0 => Ok(Expr::Vector([v[0], v[1], v[2]])),
            other => Err(self.unsupported(format!("constant {other:?}"))),
        }
    }

    fn name(&self, index: u32) -> Result<Vec<u8>, Error> {
        match self.proto.string(index) {
            Some(s) if is_identifier(s) => Ok(s.to_vec()),
            Some(s) => Err(self.unsupported(format!("name {:?} is not an identifier", String::from_utf8_lossy(s)))),
            None => Err(self.unsupported(format!("constant {index} is not a string"))),
        }
    }

    // a store to a table under construction becomes one of its fields
    fn store(&mut self, table_reg: u8, key: Option<u8>, key_const: Option<Expr>, value_reg: u8) -> Result<(), Error> {
        if let Slot::Pending(table_id) = self.slots[table_reg as usize] {
            let table_index = self.index_of(table_id);
            let operands = key.into_iter().chain([value_reg]).collect::<Vec<_>>();
            if matches!(self.pending[table_index].value, Value::Table(..))
                && let Some(used) = self.foldable(&operands)
                && table_index < self.pending.len() - used.len()
            {
                // values evaluated before this field but not stored yet (older compilers kept
                // positional values until the end) become locals so their order holds
                let between_end = self.pending.len() - used.len();
                if table_index + 1 < between_end {
                    self.flush_keeping(between_end, &operands)?;
                }
                let mut values = self.take(&operands)?.into_iter();
                let key = match key {
                    Some(_) => values.next().unwrap().expr(),
                    None => key_const.unwrap(),
                };
                let value = values.next().unwrap().expr();
                // taking the operands can turn older values into locals, the table included
                if self.slots[table_reg as usize] == Slot::Pending(table_id)
                    && self.index_of(table_id) + 1 == self.pending.len()
                {
                    let table_index = self.index_of(table_id);
                    if let Value::Table(fields, _) = &mut self.pending[table_index].value {
                        fields.push(Field::Keyed(key, value));
                    }
                    return Ok(());
                }
                let table = self.take1(table_reg)?;
                return self.emit(Stmt::Assign { targets: vec![Expr::Index(Box::new(table), Box::new(key))], values: vec![value] });
            }
        }
        let regs = [Some(table_reg), key, Some(value_reg)].into_iter().flatten().collect::<Vec<_>>();
        let mut values = self.take_exprs(&regs)?.into_iter();
        let table = values.next().unwrap();
        let key = match key {
            Some(_) => values.next().unwrap(),
            None => key_const.unwrap(),
        };
        let value = values.next().unwrap();
        self.emit(Stmt::Assign { targets: vec![Expr::Index(Box::new(table), Box::new(key))], values: vec![value] })
    }

    fn closure(&mut self, insns: &[Insn], at: usize, target: u8, proto: usize) -> Result<usize, Error> {
        let captures = insns[at + 1..].iter().take_while(|i| i.op == Op::CAPTURE).collect::<Vec<_>>();
        let self_capture = captures.iter().any(|c| c.a != 2 && c.b == target) && self.slots[target as usize] == Slot::Free;
        // captured registers have to hold locals: their values (and everything older) become
        // locals, the rest can stay pending
        let captured = captures.iter().filter(|c| c.a != 2 && c.b != target).map(|c| c.b).collect::<Vec<_>>();
        let newest = captured
            .iter()
            .filter_map(|&r| match self.slots[r as usize] {
                Slot::Pending(id) => Some(self.index_of(id)),
                _ => None,
            })
            .max();
        if let Some(index) = newest {
            self.flush_keeping(index + 1, &captured)?;
        }
        let local = if self_capture {
            self.flush_for_statement()?;
            let local = self.new_local(target, self.pc);
            self.slots[target as usize] = Slot::Local(local);
            Some(local)
        } else {
            None
        };
        let mut list = Vec::new();
        for c in &captures {
            list.push(match c.a {
                0 | 1 => match self.slots[c.b as usize] {
                    Slot::Local(l) => {
                        if c.a == 0 {
                            self.by_value[l as usize] = true;
                        } else {
                            self.by_ref[l as usize] = true;
                            self.ever_by_ref[l as usize] = true;
                        }
                        Capture::Local(l)
                    }
                    _ => return Err(self.unsupported(format!("capture of R{} that holds no local", c.b))),
                },
                2 => Capture::Upvalue(c.b),
                t => return Err(self.unsupported(format!("capture type {t}"))),
            });
        }
        let child = &self.chunk.protos[proto];
        if list.len() != child.num_upvalues as usize {
            return Err(self.unsupported("capture count differs from the function's upvalues"));
        }
        let closure = Closure { proto, captures: list };
        match local {
            Some(local) => self.emit(Stmt::LocalFunction { local, closure })?,
            None => self.write(target, Expr::Closure(Box::new(closure)))?,
        }
        Ok(1 + captures.len())
    }

    // handles insns[at]; returns how many instructions it used
    fn step(&mut self, insns: &[Insn], at: usize) -> Result<usize, Error> {
        use Op::*;
        let i = insns[at];
        let (a, b, c) = (i.a, i.b, i.c);
        let binary = |op| match op {
            ADD | ADDK | SUBRK => BinOp::Add,
            SUB | SUBK => BinOp::Sub,
            MUL | MULK => BinOp::Mul,
            DIV | DIVK | DIVRK => BinOp::Div,
            IDIV | IDIVK => BinOp::IDiv,
            MOD | MODK => BinOp::Mod,
            POW | POWK => BinOp::Pow,
            AND | ANDK => BinOp::And,
            _ => BinOp::Or,
        };
        match i.op {
            NOP | COVERAGE | PREPVARARGS => {}
            LOADNIL => self.write(a, Expr::Nil)?,
            LOADB if c == 0 => self.write(a, Expr::Bool(b != 0))?,
            LOADN => self.write(a, Expr::Number(i.d() as f64))?,
            LOADK => self.write(a, self.constant(i.d() as u32)?)?,
            LOADKX => self.write(a, self.constant(i.aux)?)?,
            MOVE => {
                let value = self.take1(b)?;
                self.write(a, value)?
            }
            GETGLOBAL => self.write(a, Expr::Global(self.name(i.aux)?))?,
            SETGLOBAL => {
                let name = self.name(i.aux)?;
                let value = self.take1(a)?;
                self.emit(Stmt::Assign { targets: vec![Expr::Global(name)], values: vec![value] })?
            }
            GETUPVAL => self.write(a, Expr::Upvalue(b))?,
            SETUPVAL => {
                let value = self.take1(a)?;
                self.emit(Stmt::Assign { targets: vec![Expr::Upvalue(b)], values: vec![value] })?
            }
            // closures stop sharing the locals from here up; the registers keep their values
            // (a following `return` still reads them), but a later write no longer reaches a
            // closure, so it needn't be an assignment
            CLOSEUPVALS => {
                for slot in &self.slots[a as usize..] {
                    if let Slot::Local(l) = *slot {
                        self.by_ref[l as usize] = false;
                    }
                }
            }
            GETIMPORT => {
                let Some(Constant::Import(path)) = self.proto.constants.get(i.d() as usize) else {
                    return Err(self.unsupported("GETIMPORT of a non-import constant"));
                };
                let mut expr = Expr::Global(self.name(path[0])?);
                for &part in &path[1..] {
                    expr = Expr::Index(Box::new(expr), Box::new(Expr::String(self.name(part)?)));
                }
                self.write(a, expr)?
            }
            GETTABLE => {
                let mut v = self.take_exprs(&[b, c])?.into_iter();
                let (t, k) = (v.next().unwrap(), v.next().unwrap());
                self.write(a, Expr::Index(Box::new(t), Box::new(k)))?
            }
            GETTABLEKS | GETUDATAKS => {
                let key = self.proto.string(if i.op == GETUDATAKS { i.aux & 0xffff } else { i.aux }).ok_or_else(|| self.unsupported("key is not a string"))?.to_vec();
                let t = self.take1(b)?;
                self.write(a, Expr::Index(Box::new(t), Box::new(Expr::String(key))))?
            }
            GETTABLEN => {
                let t = self.take1(b)?;
                self.write(a, Expr::Index(Box::new(t), Box::new(Expr::Number(c as f64 + 1.0))))?
            }
            SETTABLE => self.store(b, Some(c), None, a)?,
            SETTABLEKS | SETUDATAKS => {
                let key = self.proto.string(if i.op == SETUDATAKS { i.aux & 0xffff } else { i.aux }).ok_or_else(|| self.unsupported("key is not a string"))?.to_vec();
                self.store(b, None, Some(Expr::String(key)), a)?
            }
            SETTABLEN => self.store(b, None, Some(Expr::Number(c as f64 + 1.0)), a)?,
            NEWCLOSURE => {
                let child = *self.proto.children.get(i.d() as usize).ok_or_else(|| self.unsupported("child out of range"))?;
                return self.closure(insns, at, a, child);
            }
            DUPCLOSURE => {
                let Some(Constant::Closure(child)) = self.proto.constants.get(i.d() as usize) else {
                    return Err(self.unsupported("DUPCLOSURE of a non-closure constant"));
                };
                return self.closure(insns, at, a, *child as usize);
            }
            // Luau evaluates the object into the call's base register, then the arguments,
            // and only then looks the method up: the object's pending value turns into the
            // method in place, below the arguments, ready for CALL to fold them together
            NAMECALL | NAMECALLUDATA => {
                let method = self.name(if i.op == NAMECALLUDATA { i.aux & 0xffff } else { i.aux })?;
                self.namecall = Some((a, method.clone()));
                self.clear(a + 1)?;
                if b != a {
                    self.clear(a)?;
                }
                if self.slots[a as usize + 1] != Slot::Free {
                    return Err(self.unsupported("NAMECALL's self register is in use"));
                }
                // an object anywhere but the base is a local; a pending one becomes one first
                if b != a
                    && let Slot::Pending(id) = self.slots[b as usize]
                {
                    let index = self.index_of(id);
                    self.flush_keeping(index + 1, &[b])?;
                }
                match self.slots[b as usize] {
                    Slot::Pending(id) if b == a => {
                        let index = self.index_of(id);
                        let pending = &mut self.pending[index];
                        let Value::Expr(object) = std::mem::replace(&mut pending.value, Value::Expr(Expr::Nil)) else {
                            return Err(self.unsupported("NAMECALL on a value that isn't an expression"));
                        };
                        if pending.span != Span::One {
                            return Err(self.unsupported("NAMECALL on a value list"));
                        }
                        pending.value = Value::Method(object, method);
                        self.slots[a as usize + 1] = Slot::Pending(id);
                    }
                    // a local already sitting in the base register gets overwritten by the method
                    Slot::Local(l) if self.slots[a as usize] == Slot::Free || (b == a && !self.by_ref[l as usize]) => {
                        // just below this call's own arguments (the CALL always comes next)
                        let call = insns.get(at + 1).filter(|c| matches!(c.op, CALL | CALLFB));
                        let Some(call) = call else { return Err(self.unsupported("NAMECALL not followed by CALL")) };
                        let last_arg = if call.b == 0 { 255 } else { a as usize + call.b as usize - 1 };
                        let index = self
                            .pending
                            .iter()
                            .position(|p| p.reg as usize >= a as usize + 2 && p.reg as usize <= last_arg)
                            .unwrap_or(self.pending.len());
                        let id = self.next_id;
                        self.next_id += 1;
                        self.slots[a as usize] = Slot::Pending(id);
                        self.slots[a as usize + 1] = Slot::Pending(id);
                        self.pending.insert(index, Pending { id, reg: a, span: Span::One, value: Value::Method(Expr::Local(l), method) });
                    }
                    _ => return Err(self.unsupported("NAMECALL on an object that is neither a local nor in its base")),
                }
            }
            CALL | CALLFB => {
                let method = self.namecall.take().filter(|(reg, _)| *reg == a).map(|(_, m)| m);
                let all = if b == 0 { self.to_top(a)? } else { (a..a + b).collect() };
                // a method's self register belongs to the method, not the arguments
                let regs = all.iter().copied().filter(|&r| method.is_none() || r != a + 1).collect::<Vec<_>>();
                let mut operands = self.take(&regs)?.into_iter();
                let func = operands.next().unwrap();
                let args = operands.map(Operand::expr).collect::<Vec<_>>();
                let multret = c == 0 || c > 2;
                let call = match (func, method) {
                    (Operand::Method(object, name), _) => Expr::MethodCall { object: Box::new(object), method: name, args, multret },
                    (Operand::Expr(object), Some(name)) => {
                        self.slots[a as usize + 1] = Slot::Free;
                        Expr::MethodCall { object: Box::new(object), method: name, args, multret }
                    }
                    (Operand::Expr(func), None) => Expr::Call { func: Box::new(func), args, multret },
                };
                match c {
                    1 => self.emit(Stmt::Call(call))?,
                    2 => self.write(a, call)?,
                    0 => self.push(a, Span::All, Value::Expr(call))?,
                    n => self.results(a, n - 1, call)?,
                }
            }
            RETURN => {
                let values = match b {
                    0 => {
                        let regs = self.to_top(a)?;
                        self.take_exprs(&regs)?
                    }
                    n => self.take_exprs(&(a..a + n - 1).collect::<Vec<_>>())?,
                };
                self.emit(Stmt::Return(values))?
            }
            ADD | SUB | MUL | DIV | MOD | POW | IDIV | AND | OR => {
                let mut v = self.take_exprs(&[b, c])?.into_iter();
                let (l, r) = (v.next().unwrap(), v.next().unwrap());
                self.write(a, Expr::Binary(binary(i.op), Box::new(l), Box::new(r)))?
            }
            ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK | ANDK | ORK => {
                let l = self.take1(b)?;
                let r = self.constant(c as u32)?;
                self.write(a, Expr::Binary(binary(i.op), Box::new(l), Box::new(r)))?
            }
            SUBRK | DIVRK => {
                let l = self.constant(b as u32)?;
                let r = self.take1(c)?;
                let op = if i.op == SUBRK { BinOp::Sub } else { BinOp::Div };
                self.write(a, Expr::Binary(op, Box::new(l), Box::new(r)))?
            }
            CONCAT => {
                let values = self.take_exprs(&(b..=c).collect::<Vec<_>>())?;
                let expr = values.into_iter().rev().reduce(|right, left| Expr::Binary(BinOp::Concat, Box::new(left), Box::new(right))).unwrap();
                self.write(a, expr)?
            }
            NOT | MINUS | LENGTH => {
                let operand = self.take1(b)?;
                let op = match i.op {
                    NOT => UnOp::Not,
                    MINUS => UnOp::Neg,
                    _ => UnOp::Len,
                };
                self.write(a, Expr::Unary(op, Box::new(operand)))?
            }
            NEWTABLE => self.new_table(a, Vec::new())?,
            DUPTABLE => {
                let Some(Constant::Table(keys)) = self.proto.constants.get(i.d() as usize) else {
                    return Err(self.unsupported("DUPTABLE of a non-table constant"));
                };
                // constants the template already holds come first: they are string keys, so
                // no positional value can land on them
                let mut fields = Vec::new();
                for (key, value) in keys.clone() {
                    if let Some(value) = value {
                        fields.push(Field::Keyed(self.constant(key)?, self.constant(value)?));
                    }
                }
                self.new_table(a, fields)?
            }
            SETLIST => self.set_list(a, b, c, i.aux)?,
            GETVARARGS => match b {
                0 => self.push(a, Span::All, Value::Expr(Expr::VarArg { multret: true }))?,
                2 => self.write(a, Expr::VarArg { multret: false })?,
                n => self.results(a, n - 1, Expr::VarArg { multret: true })?,
            },
            FASTCALL | FASTCALL1 | FASTCALL2 | FASTCALL2K | FASTCALL3 | FASTPCALL => {
                self.fastcall = Some(Fastcall { call_pc: i.pc + 1 + c as usize, first_after: self.next_id });
            }
            op => return Err(self.unsupported(format!("{} (control flow comes in M2)", op.name()))),
        }
        Ok(1)
    }

    fn results(&mut self, reg: u8, count: u8, expr: Expr) -> Result<(), Error> {
        let regs = reg..reg + count;
        for r in regs.clone() {
            self.clear(r)?;
        }
        if regs.clone().all(|r| self.slots[r as usize] == Slot::Free) {
            return self.push(reg, Span::Many(count), Value::Expr(expr));
        }
        if regs.clone().any(|r| matches!(self.slots[r as usize], Slot::Pending(_))) {
            self.flush()?;
        }
        let targets = regs
            .map(|r| match self.slots[r as usize] {
                Slot::Local(l) if !self.by_value[l as usize] => Ok(Expr::Local(l)),
                _ => Err(self.unsupported("results land on a mix of new and existing locals")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.emit(Stmt::Assign { targets, values: vec![expr] })
    }

    fn new_table(&mut self, reg: u8, fields: Vec<Field>) -> Result<(), Error> {
        self.clear(reg)?;
        match self.slots[reg as usize] {
            Slot::Free => self.push(reg, Span::One, Value::Table(fields, 0)),
            _ => self.write(reg, table(fields)),
        }
    }

    fn set_list(&mut self, table_reg: u8, from: u8, count: u8, start: u32) -> Result<(), Error> {
        let regs = if count == 0 { self.to_top(from)? } else { (from..from + count - 1).collect() };
        let constructing = match self.slots[table_reg as usize] {
            Slot::Pending(id) => match &self.pending[self.index_of(id)].value {
                Value::Table(_, positional) if positional + 1 == start => Some(id),
                _ => None,
            },
            _ => None,
        };
        if let Some(table_id) = constructing {
            let values = self.take(&regs)?;
            // anything else above the table (an inlined body's own locals, say) was evaluated
            // before these values were stored, so it becomes locals first
            if self.slots[table_reg as usize] == Slot::Pending(table_id) && self.index_of(table_id) + 1 < self.pending.len() {
                let all = self.pending.len();
                self.flush_keeping(all, &regs)?;
            }
            if self.slots[table_reg as usize] == Slot::Pending(table_id) && self.index_of(table_id) + 1 == self.pending.len() {
                let index = self.index_of(table_id);
                let Value::Table(fields, positional) = &mut self.pending[index].value else { unreachable!() };
                for operand in values {
                    *positional += 1;
                    fields.push(Field::Positional(operand.expr()));
                }
                return Ok(());
            }
            if count == 0 {
                return Err(self.unsupported("open SETLIST into a constructor that had to become a local"));
            }
            let table = self.take1(table_reg)?;
            let values = values.into_iter().map(Operand::expr).collect::<Vec<_>>();
            return self.store_list(table, values, start);
        }
        if count == 0 {
            return Err(self.unsupported("open SETLIST outside a constructor"));
        }
        let table = self.take1(table_reg)?;
        let values = self.take_exprs(&regs)?;
        self.store_list(table, values, start)
    }

    fn store_list(&mut self, table: Expr, values: Vec<Expr>, start: u32) -> Result<(), Error> {
        if !matches!(table, Expr::Local(_)) {
            return Err(self.unsupported("SETLIST into a table that is not a local"));
        }
        for (i, value) in values.into_iter().enumerate() {
            let key = Expr::Number((start as usize + i) as f64);
            self.emit(Stmt::Assign { targets: vec![Expr::Index(Box::new(table.clone()), Box::new(key))], values: vec![value] })?;
        }
        Ok(())
    }
}

fn table(fields: Vec<Field>) -> Expr {
    Expr::Table(fields)
}
