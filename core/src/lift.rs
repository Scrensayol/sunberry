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

#[derive(Clone, Debug)]
enum Value {
    Expr(Expr),
    // fields in the order their stores ran (Luau flushes positional values before each keyed
    // one, so that is the source order), and how many positional values there are so far
    Table(Vec<Field>, u32),
    // NAMECALL's method and self, waiting for their CALL
    Method(Expr, Vec<u8>),
}

#[derive(Clone, Debug)]
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

#[derive(Clone)]
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
    // registers read again after each instruction, and on entry to it
    live: Vec<Regs>,
    live_in: Vec<Regs>,
    insns: Vec<Insn>,
    // pc -> instruction index (usize::MAX on AUX words); code.len() maps to insns.len()
    index_of: Vec<usize>,
    // registers each enclosing branch writes that are read after it joins: writes to them
    // assign the variable declared before the `if`
    joins: Vec<Vec<u8>>,
    loops: Vec<LoopCtx>,
    trace: bool,
}

// where a loop's far exits land, the pcs jumping there, and the flag (see `far_exits`)
type Far = Option<(Vec<usize>, usize, u32)>;

// the loop being lifted: where a `break` and a `continue` jump to
struct LoopCtx {
    breaks: Vec<usize>,
    cont: usize,
    continued: bool,
    // jumps past the loop and the code right after it (an inlined function's `return`
    // inside a loop): their targets, and the flag that says the loop left that way
    far: Vec<usize>,
    flag: Option<u32>,
}

// jump targets standing for `break` and `continue` once normalized
const BREAK: usize = usize::MAX - 1;
const CONTINUE: usize = usize::MAX - 2;
const FAR: usize = usize::MAX - 3;

// everything a speculative path can change, to put back if it doesn't work out
struct Snapshot {
    slots: Vec<Slot>,
    pending: Vec<Pending>,
    next_id: u64,
    body: usize,
    names: usize,
    by_value: Vec<bool>,
    by_ref: Vec<bool>,
    ever_by_ref: Vec<bool>,
    fastcall: Option<Fastcall>,
    namecall: Option<(u8, Vec<u8>)>,
    // where lifting was: liveness questions are asked about the current instruction
    pc: usize,
    at: usize,
}

struct Test {
    pred: Expr,
    // whether the jump is taken when `pred` is truthy (else when it is falsy)
    jump_when: bool,
    // the pending value a JUMPIF/JUMPIFNOT tested in the region's result register: on the
    // jump path that register still holds it (`x and y`, `x or y`)
    subject: Option<u64>,
}

enum Tree {
    Value(Expr),
    Test { pred: Expr, jump_when: bool, subject: Option<u64>, fall: Box<Tree>, jump: Box<Tree> },
}

#[derive(Clone, Copy, PartialEq)]
enum Succ {
    Node(usize),
    Then,
    Else,
}

struct CondNode {
    // statements that run before `pred` is looked at (an inlined body's, say)
    pre: Vec<Stmt>,
    pred: Expr,
    // where control goes when `pred` is truthy / falsy
    t: Succ,
    f: Succ,
}

fn is_test(op: Op) -> bool {
    use Op::*;
    matches!(
        op,
        JUMPIF | JUMPIFNOT | JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT | JUMPXEQKNIL
            | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS
    )
}

// instructions that only ever compute values into registers
fn is_expression(insn: &Insn) -> bool {
    use Op::*;
    match insn.op {
        CALL | CALLFB => insn.c != 1,
        LOADB => insn.c == 0,
        LOADNIL | LOADN | LOADK | LOADKX | MOVE | GETGLOBAL | GETUPVAL | GETIMPORT | GETTABLE | GETTABLEKS | GETTABLEN
        | GETUDATAKS | NEWCLOSURE | DUPCLOSURE | CAPTURE | NAMECALL | NAMECALLUDATA | ADD | SUB | MUL | DIV | MOD | POW
        | IDIV | ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK | SUBRK | DIVRK | AND | OR | ANDK | ORK | CONCAT | NOT
        | MINUS | LENGTH | NEWTABLE | DUPTABLE | SETLIST | SETTABLE | SETTABLEKS | SETTABLEN | SETUDATAKS | GETVARARGS
        | FASTCALL | FASTCALL1 | FASTCALL2 | FASTCALL2K | FASTCALL3 | FASTPCALL | NOP | COVERAGE => true,
        _ => false,
    }
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
        live_in: Vec::new(),
        insns: Vec::new(),
        index_of: Vec::new(),
        joins: Vec::new(),
        loops: Vec::new(),
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
    let mut index_of = vec![usize::MAX; proto.code.len() + 1];
    for (i, insn) in insns.iter().enumerate() {
        index_of[insn.pc] = i;
    }
    index_of[proto.code.len()] = insns.len();
    let liveness = liveness::analyze(&insns, &index_of);
    lifter.live = liveness.live_after;
    lifter.live_in = liveness.live_in;
    lifter.insns = insns;
    lifter.index_of = index_of;
    lifter.block(0, lifter.insns.len(), &[proto.code.len()])?;
    lifter.flush()?;
    let body = pin_constants(lifter.body, &lifter.ever_by_ref);
    let body = limit_locals(body, proto.num_params as usize);
    Ok(Function { params, is_vararg: proto.is_vararg, body, names: lifter.names })
}

// From O1 up, Luau folds a local that is never assigned and starts as a constant into every
// use, and folds what it can at compile time, where `x ^ 3` is pow() instead of the VM's
// x*x*x. A local the bytecode keeps in a register and reads wasn't folded, so the source
// assigned it somewhere (or was O0, where nothing folds). The output must not be more
// foldable than that: such a local is declared bare and then assigned.
fn pin_constants(body: Vec<Stmt>, by_ref: &[bool]) -> Vec<Stmt> {
    fn mark(body: &[Stmt], assigned: &mut [bool]) {
        for stmt in body {
            if let Stmt::Assign { targets, .. } = stmt {
                for target in targets {
                    if let Expr::Local(l) = target {
                        assigned[*l as usize] = true;
                    }
                }
            }
            for nested in stmt.bodies() {
                mark(nested, assigned);
            }
        }
    }
    let mut assigned = vec![false; by_ref.len()];
    mark(&body, &mut assigned);
    pin(body, &assigned, by_ref)
}

fn pin(body: Vec<Stmt>, assigned: &[bool], by_ref: &[bool]) -> Vec<Stmt> {
    let mut out = Vec::with_capacity(body.len());
    for stmt in body {
        match stmt {
            mut stmt @ (Stmt::If { .. } | Stmt::While { .. } | Stmt::Repeat { .. } | Stmt::NumericFor { .. } | Stmt::GenericFor { .. } | Stmt::Do(_)) => {
                for nested in stmt.bodies_mut() {
                    *nested = pin(std::mem::take(nested), assigned, by_ref);
                }
                out.push(stmt);
            }
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

// Luau allows 200 locals in scope at once. The source kept that by scoping them in blocks,
// which only show in the bytecode as registers being reused; a block declaring many locals
// is cut where everything declared since the last cut is never mentioned again (closures
// mention what they capture), and each run goes in `do ... end`.
const LOCALS_BEFORE_SCOPING: usize = 150;

// `outer`: locals already in scope from the enclosing blocks (registers are per function)
fn limit_locals(body: Vec<Stmt>, outer: usize) -> Vec<Stmt> {
    let declared = body.iter().map(|s| declares(s).len()).sum::<usize>();
    let mut body = if outer + declared <= LOCALS_BEFORE_SCOPING { body } else { scope_runs(body) };
    let mut running = outer;
    for stmt in &mut body {
        // a for loop's own registers: three hidden ones and its variables
        let held = match stmt {
            Stmt::NumericFor { .. } => 4,
            Stmt::GenericFor { vars, .. } => 3 + vars.len(),
            _ => 0,
        };
        for nested in stmt.bodies_mut() {
            *nested = limit_locals(std::mem::take(nested), running + held);
        }
        running += declares(stmt).len();
    }
    body
}

// wraps runs of statements whose locals are done with by the run's end in `do ... end`
fn scope_runs(body: Vec<Stmt>) -> Vec<Stmt> {
    // the last statement mentioning each local
    let mut last = std::collections::HashMap::new();
    for (i, stmt) in body.iter().enumerate() {
        let mut refs = Vec::new();
        stmt_refs(stmt, &mut refs);
        for l in refs {
            last.insert(l, i);
        }
    }
    // how far each statement's own locals reach
    let n = body.len();
    let reach = body
        .iter()
        .enumerate()
        .map(|(i, s)| declares(s).iter().map(|l| last.get(l).copied().unwrap_or(i)).max().unwrap_or(i))
        .collect::<Vec<_>>();
    let mut items = body.into_iter().map(Some).collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        // the shortest run from here that everything declared in it is done with by its end;
        // locals living to the end of the block stay outside
        // a later `local` in the run that outlives it doesn't stretch it: its declaration moves
        // in front of the `do` and the statement becomes an assignment
        let mut end = reach[i];
        let mut k = i + 1;
        while k <= end {
            if !matches!(items[k], Some(Stmt::Local { .. })) {
                end = end.max(reach[k]);
            }
            k += 1;
        }
        let declaring = !declares(items[i].as_ref().unwrap()).is_empty();
        if declaring && end + 1 < n {
            let mut run = Vec::new();
            for k in i..=end {
                match items[k].take().unwrap() {
                    Stmt::Local { locals, values } if k > i && reach[k] > end => {
                        out.push(Stmt::Local { locals: locals.clone(), values: Vec::new() });
                        if !values.is_empty() {
                            run.push(Stmt::Assign { targets: locals.into_iter().map(Expr::Local).collect(), values });
                        }
                    }
                    stmt => run.push(stmt),
                }
            }
            out.push(Stmt::Do(run));
            i = end + 1;
        } else {
            out.push(items[i].take().unwrap());
            i += 1;
        }
    }
    out
}

fn declares(stmt: &Stmt) -> Vec<u32> {
    match stmt {
        Stmt::Local { locals, .. } => locals.clone(),
        Stmt::LocalFunction { local, .. } => vec![*local],
        _ => Vec::new(),
    }
}

fn stmt_refs(stmt: &Stmt, out: &mut Vec<u32>) {
    match stmt {
        Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr_refs(v, out)),
        Stmt::LocalFunction { local, closure } => {
            out.push(*local);
            closure_refs(closure, out);
        }
        Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr_refs(v, out)),
        Stmt::Call(e) | Stmt::If { cond: e, .. } | Stmt::While { cond: e, .. } | Stmt::Repeat { cond: e, .. } => expr_refs(e, out),
        Stmt::NumericFor { from, to, step, .. } => [Some(from), Some(to), step.as_ref()].into_iter().flatten().for_each(|v| expr_refs(v, out)),
        Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr_refs(v, out)),
        Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
    }
    for nested in stmt.bodies() {
        for s in nested {
            stmt_refs(s, out);
        }
    }
}

fn closure_refs(closure: &Closure, out: &mut Vec<u32>) {
    for capture in &closure.captures {
        if let Capture::Local(l) = capture {
            out.push(*l);
        }
    }
}

fn expr_refs(e: &Expr, out: &mut Vec<u32>) {
    match e {
        Expr::Local(l) => out.push(*l),
        Expr::Closure(c) => closure_refs(c, out),
        Expr::Index(a, b) | Expr::Binary(_, a, b) => {
            expr_refs(a, out);
            expr_refs(b, out);
        }
        Expr::Unary(_, a) => expr_refs(a, out),
        Expr::IfElse(a, b, c) => {
            expr_refs(a, out);
            expr_refs(b, out);
            expr_refs(c, out);
        }
        Expr::Call { func, args, .. } => {
            expr_refs(func, out);
            args.iter().for_each(|a| expr_refs(a, out));
        }
        Expr::MethodCall { object, args, .. } => {
            expr_refs(object, out);
            args.iter().for_each(|a| expr_refs(a, out));
        }
        Expr::Table(fields) => fields.iter().for_each(|f| match f {
            Field::Positional(v) => expr_refs(v, out),
            Field::Keyed(k, v) => {
                expr_refs(k, out);
                expr_refs(v, out);
            }
        }),
        _ => {}
    }
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
    fn assigns(&self, reg: u8) -> Option<u32> {
        let Slot::Local(l) = self.slots[reg as usize] else { return None };
        // read after the branch it's in joins: the variable from before the `if`
        if self.joins.iter().any(|j| j.contains(&reg)) {
            return Some(l);
        }
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
            // a constant nobody reads has nothing to evaluate
            if self.pending[index].span == Span::One
                && matches!(&self.pending[index].value, Value::Expr(e) if pure_constant(e))
            {
                self.pending.remove(index);
                self.slots[reg as usize] = Slot::Free;
                return Ok(());
            }
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
        // nothing may follow a return, break or continue in its block
        if stmt.terminates() {
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
        self.take_ignoring(regs, &[])
    }

    // `ignore`: registers whose later reads the caller accounts for itself
    fn take_ignoring(&mut self, regs: &[u8], ignore: &[u8]) -> Result<Vec<Operand>, Error> {
        // a value read again later can't be folded into this reader: it becomes a local
        // (a register this instruction overwrites is read later only as its new value)
        let live = self.live[self.at];
        let overwritten = liveness::writes(&self.insns[self.at]);
        let reused = regs
            .iter()
            .filter(|&&r| live.has(r) && !overwritten.has(r) && !ignore.contains(&r))
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
        if let Slot::Pending(_) = self.slots[reg as usize] {
            self.clear(reg)?;
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

    fn closure(&mut self, at: usize, target: u8, proto: usize) -> Result<usize, Error> {
        let captures = self.insns[at + 1..].iter().take_while(|i| i.op == Op::CAPTURE).copied().collect::<Vec<_>>();
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
    fn step(&mut self, at: usize) -> Result<usize, Error> {
        use Op::*;
        let i = self.insns[at];
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
                return self.closure(at, a, child);
            }
            DUPCLOSURE => {
                let Some(Constant::Closure(child)) = self.proto.constants.get(i.d() as usize) else {
                    return Err(self.unsupported("DUPCLOSURE of a non-closure constant"));
                };
                return self.closure(at, a, *child as usize);
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
                        let call = self.insns.get(at + 1).copied().filter(|c| matches!(c.op, CALL | CALLFB));
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
                let (mut l, mut r) = (v.next().unwrap(), v.next().unwrap());
                // a runtime `x ^ y` is pow(); with y a constant 2, 0.5 or 3 the recompiled code
                // would use POWK's x*x, sqrt or x*x*x instead, so y stays in a variable
                if i.op == POW && power_special(&r) {
                    l = self.pin(l)?;
                    r = self.pin(r)?;
                }
                self.write(a, Expr::Binary(binary(i.op), Box::new(l), Box::new(r)))?
            }
            ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK | ANDK | ORK => {
                let mut l = self.take1(b)?;
                let r = self.constant(c as u32)?;
                // POWK special-cases ^2, ^0.5 and ^3 at runtime, but a constant base would let
                // Luau fold the whole thing with pow() (sqrt(-0) is -0, pow(-0, 0.5) is 0)
                if i.op == POWK && power_special(&r) && foldable(&l) {
                    l = self.pin(l)?;
                }
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
            op => return Err(self.unsupported(format!("{} (loops come in M3)", op.name()))),
        }
        Ok(1)
    }

    // a value Luau can't fold: a local assigned after its declaration
    fn pin(&mut self, expr: Expr) -> Result<Expr, Error> {
        if matches!(expr, Expr::Local(_)) || !foldable(&expr) {
            return Ok(expr);
        }
        let local = self.new_local(u8::MAX, self.pc);
        // literals and operators on them read nothing, so they can go ahead of whatever is
        // still pending; anything else waits its turn
        if !literal(&expr) {
            self.flush_for_statement()?;
        }
        self.body.push(Stmt::Local { locals: vec![local], values: Vec::new() });
        self.body.push(Stmt::Assign { targets: vec![Expr::Local(local)], values: vec![expr] });
        Ok(Expr::Local(local))
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

// Control flow. Luau lays every construct out as a contiguous range of code in source
// order, so structure is recovered by recursive descent over instruction ranges. From O1 up
// Luau threads jumps through unconditional JUMPs, so a region's "end" is a set of pcs: where
// it continues, and the targets of the JUMPs chained from there (`exits`).
impl Lifter<'_> {
    fn idx(&self, pc: usize) -> Result<usize, Error> {
        match self.index_of.get(pc) {
            Some(&i) if i != usize::MAX => Ok(i),
            _ => Err(self.unsupported(format!("jump into the middle of an instruction at {pc}"))),
        }
    }

    // A jump target as this region sees it: the region's end for anything that leads there,
    // and for a RETURN elsewhere the last identical RETURN in the region (Luau copies a
    // RETURN over a jump to it, so the original can sit outside); returning from either
    // returns the same registers.
    fn normalize(&self, target: usize, first: usize, end: usize, exits: &[usize]) -> usize {
        if exits.contains(&target) {
            return exits[0];
        }
        if let Some(lp) = self.loops.last() {
            if lp.breaks.contains(&target) {
                return BREAK;
            }
            if target == lp.cont {
                return CONTINUE;
            }
            if lp.far.contains(&target) {
                return FAR;
            }
        }
        if target > exits[0]
            && let Some(&i) = self.index_of.get(target)
            && let Some(insn) = self.insns.get(i)
            && insn.op == Op::RETURN
            && let Some(copy) = (first + 1..end).rev().find(|&j| self.insns[j].op == Op::RETURN && self.insns[j].a == insn.a && self.insns[j].b == insn.b)
        {
            return self.insns[copy].pc;
        }
        target
    }

    // whether code outside insns[from..to) jumps into it (if not, it's dead)
    fn entered(&self, from: usize, to: usize) -> bool {
        let (lo, hi) = (self.pc_at(from), self.pc_at(to));
        self.insns.iter().enumerate().any(|(k, i)| (k < from || k >= to) && i.target().is_some_and(|t| t >= lo && t < hi))
    }

    fn pc_at(&self, index: usize) -> usize {
        self.insns.get(index).map_or(self.proto.code.len(), |i| i.pc)
    }

    fn exits_from(&self, index: usize) -> Vec<usize> {
        let mut pc = self.pc_at(index);
        let mut out = vec![pc];
        while let Some(&i) = self.index_of.get(pc)
            && let Some(insn) = self.insns.get(i)
            && matches!(insn.op, Op::JUMP | Op::JUMPX)
            && let Some(t) = insn.target()
            && t > pc
        {
            out.push(t);
            pc = t;
        }
        out
    }

    // lifts into a fresh statement list, which it returns; the current one is put back
    // whatever happens, so a failed attempt can be retried another way
    fn sub_body(&mut self, f: impl FnOnce(&mut Self) -> Result<(), Error>) -> Result<Vec<Stmt>, Error> {
        let outer = std::mem::take(&mut self.body);
        let result = f(self);
        let inner = std::mem::replace(&mut self.body, outer);
        result.map(|_| inner)
    }

    fn enter(&mut self, i: usize) -> Result<(), Error> {
        self.pc = self.insns[i].pc;
        self.at = i;
        if self.trace {
            let stack = self.pending.iter().map(|p| format!("R{}{:?}", p.reg, p.span)).collect::<Vec<_>>();
            eprintln!("f{} {:>4} {:<12} pending [{}]", self.proto_id, self.pc, self.insns[i].op.name(), stack.join(" "));
        }
        self.scope_changes()
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            slots: self.slots.clone(),
            pending: self.pending.clone(),
            next_id: self.next_id,
            body: self.body.len(),
            names: self.names.len(),
            by_value: self.by_value.clone(),
            by_ref: self.by_ref.clone(),
            ever_by_ref: self.ever_by_ref.clone(),
            fastcall: self.fastcall.clone(),
            namecall: self.namecall.clone(),
            pc: self.pc,
            at: self.at,
        }
    }

    fn restore(&mut self, s: &Snapshot) {
        self.slots.clone_from(&s.slots);
        self.pending.clone_from(&s.pending);
        self.next_id = s.next_id;
        self.body.truncate(s.body);
        self.names.truncate(s.names);
        self.by_value.clone_from(&s.by_value);
        self.by_ref.clone_from(&s.by_ref);
        self.ever_by_ref.clone_from(&s.ever_by_ref);
        self.fastcall.clone_from(&s.fastcall);
        self.namecall.clone_from(&s.namecall);
        self.pc = s.pc;
        self.at = s.at;
    }

    // lifts insns[start..end); `exits` are the pcs meaning "the end of this region"
    fn block(&mut self, start: usize, end: usize, exits: &[usize]) -> Result<(), Error> {
        let mut i = start;
        while i < end {
            // after a return nothing in the region runs (Luau leaves threaded RETURN copies
            // there), unless something jumps in
            if self.body.last().is_some_and(Stmt::terminates) {
                if self.entered(i, end) {
                    return Err(self.unsupported("code after a return that something jumps to"));
                }
                return Ok(());
            }
            self.enter(i)?;
            let insn = self.insns[i];
            match insn.op {
                Op::FORNPREP => {
                    i = self.numeric_for(i, end, exits)?;
                    continue;
                }
                Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT => {
                    i = self.generic_for(i, end, exits)?;
                    continue;
                }
                _ => {}
            }
            if let Some(back) = self.back_edge(i, end) {
                i = self.jumpback_loop(i, back, end, exits)?;
                continue;
            }
            if is_test(insn.op) {
                if let Some(next) = self.value_region(i, end)? {
                    i = next;
                    continue;
                }
                // a test going to the same place either way (folding leaves these) still has
                // to compare: an empty `if` does exactly that
                let target = insn.target().unwrap();
                if target == self.pc_at(i + 1) || (i + 1 == end && exits.contains(&target)) {
                    let test = self.test(i, None)?;
                    self.emit(Stmt::If { cond: test.pred, then_body: Vec::new(), else_body: Vec::new() })?;
                    i += 1;
                    continue;
                }
                i = self.statement_if(i, end, exits)?;
                continue;
            }
            match insn.op {
                Op::JUMP | Op::JUMPX => {
                    let target = insn.target().unwrap();
                    let loop_target = self.loops.last().map(|l| (l.breaks.contains(&target), l.cont == target));
                    let far = self.loops.last().is_some_and(|l| l.far.contains(&target));
                    if target == insn.pc + 1 || (i + 1 == end && exits.contains(&target)) {
                        i += 1;
                    } else if far {
                        for stmt in self.far_exit_stmts() {
                            self.emit(stmt)?;
                        }
                        i += 1;
                    } else if let Some((true, _)) = loop_target {
                        self.emit(Stmt::Break)?;
                        i += 1;
                    } else if let Some((_, true)) = loop_target {
                        self.loops.last_mut().unwrap().continued = true;
                        self.emit(Stmt::Continue)?;
                        i += 1;
                    } else if exits.contains(&target) && !self.entered(i + 1, end) {
                        // leaving the region early: what follows is dead (constant folding
                        // leaves this behind)
                        return Ok(());
                    } else if target > insn.pc
                        && let Ok(t) = self.idx(target)
                        && t <= end
                        && !self.entered(i + 1, t)
                    {
                        // jumping over code nothing else reaches
                        i = t;
                    } else if target <= insn.pc {
                        return Err(self.unsupported("loops come in M3"));
                    } else {
                        return Err(self.unsupported("a jump out of its region"));
                    }
                }
                _ => i += self.step(i)?,
            }
        }
        Ok(())
    }

    fn test(&mut self, i: usize, result: Option<u8>) -> Result<Test, Error> {
        use Op::*;
        let insn = self.insns[i];
        let (a, aux) = (insn.a, insn.aux);
        Ok(match insn.op {
            JUMPIF | JUMPIFNOT => {
                let pending = match self.slots[a as usize] {
                    Slot::Pending(id) if result == Some(a) => Some(id),
                    _ => None,
                };
                let ignore = pending.map(|_| a).into_iter().collect::<Vec<_>>();
                let pred = self.take_ignoring(&[a], &ignore)?.pop().unwrap().expr();
                let subject = pending.filter(|_| self.slots[a as usize] == Slot::Free);
                Test { pred, jump_when: insn.op == JUMPIF, subject }
            }
            JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT => {
                let b = (aux & 0xff) as u8;
                let (op, jump_when) = match insn.op {
                    JUMPIFEQ => (BinOp::Eq, true),
                    JUMPIFNOTEQ => (BinOp::Eq, false),
                    JUMPIFLT => (BinOp::Lt, true),
                    JUMPIFNOTLT => (BinOp::Lt, false),
                    JUMPIFLE => (BinOp::Le, true),
                    _ => (BinOp::Le, false),
                };
                // `x > y` compiles to y < x with the operands still evaluated x first; keep
                // that order by printing it the way round it was evaluated
                let swapped = self.foldable(&[a, b]).is_none() && self.foldable(&[b, a]).is_some();
                let pred = if swapped {
                    let mut v = self.take_exprs(&[b, a])?.into_iter();
                    let (rb, ra) = (v.next().unwrap(), v.next().unwrap());
                    let flipped = match op {
                        BinOp::Lt => BinOp::Gt,
                        BinOp::Le => BinOp::Ge,
                        other => other,
                    };
                    Expr::Binary(flipped, Box::new(rb), Box::new(ra))
                } else {
                    let mut v = self.take_exprs(&[a, b])?.into_iter();
                    let (ra, rb) = (v.next().unwrap(), v.next().unwrap());
                    Expr::Binary(op, Box::new(ra), Box::new(rb))
                };
                // never `not (x < y)` turned into `x >= y`: NaN makes them differ
                Test { pred, jump_when, subject: None }
            }
            JUMPXEQKNIL | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS => {
                let constant = match insn.op {
                    JUMPXEQKNIL => Expr::Nil,
                    JUMPXEQKB => Expr::Bool(aux & 1 != 0),
                    _ => self.constant(aux & 0xff_ffff)?,
                };
                let value = self.take1(a)?;
                Test { pred: Expr::Binary(BinOp::Eq, Box::new(value), Box::new(constant)), jump_when: aux >> 31 == 0, subject: None }
            }
            op => return Err(self.unsupported(format!("{} is not a test", op.name()))),
        })
    }

    // ---- statement `if` ----

    fn statement_if(&mut self, first: usize, end: usize, exits: &[usize]) -> Result<usize, Error> {
        // the tests that could make up the condition, and the unconditional jumps a constant
        // part of one folds to, across a run of expression code; which of them really are the
        // condition's is checked per candidate
        let mut tests = vec![first];
        let mut j = first + 1;
        while j < end {
            let insn = self.insns[j];
            let forward_jump = matches!(insn.op, Op::JUMP | Op::JUMPX) && insn.target().is_some_and(|t| t > insn.pc);
            // (statements too: an inlined body's can sit between a condition's tests)
            let statement = matches!(insn.op, Op::CALL | Op::CALLFB | Op::SETGLOBAL | Op::SETUPVAL | Op::CLOSEUPVALS);
            if is_test(insn.op) || forward_jump {
                tests.push(j);
            } else if !is_expression(&insn) && insn.op != Op::LOADB && !statement {
                break;
            }
            j += 1;
        }
        let start = self.snapshot();
        let end_pc = exits[0];
        let normalized = tests
            .iter()
            .filter_map(|&t| self.insns[t].target())
            .map(|t| (t, self.normalize(t, first, end, exits)))
            .collect::<Vec<_>>();
        let normalize = move |t: usize| normalized.iter().find(|(from, _)| *from == t).map_or(t, |(_, to)| *to);
        // the longest condition that works: a nested `if` with the same exits is the same code.
        // One whose parts need no statements reads as the source did; parts with statements
        // (through a flag) are the fallback.
        let mut last = None;
        for (statements, n) in [false, true].into_iter().flat_map(|s| (1..=tests.len()).rev().map(move |n| (s, n))) {
            let then_index = tests[n - 1] + 1;
            let then_pc = self.pc_at(then_index);
            let Some(nodes) = self.condition_nodes(&tests[..n], then_pc, &normalize) else { continue };
            let mut else_pc = None;
            let mut valid = true;
            for &t in &nodes {
                let target = normalize(self.insns[t].target().unwrap());
                if target <= then_pc {
                    continue;
                }
                if (target <= end_pc || target >= FAR) && else_pc.is_none_or(|e| e == target) {
                    else_pc = Some(target);
                } else {
                    valid = false;
                }
            }
            let Some(else_pc) = else_pc.filter(|_| valid) else {
                if self.trace {
                    eprintln!("       condition {:?} then {then_pc}: no single else (region ends {end_pc}, exits {exits:?})", nodes.iter().map(|&n| self.insns[n].pc).collect::<Vec<_>>());
                }
                continue;
            };
            // a condition that reads as one can still give branches that don't fit together
            // (a test in the then branch jumping into the else): then a shorter one is tried
            let attempt = self.condition(&nodes, then_pc, else_pc, &normalize, statements).and_then(|cond| self.finish_if(cond, then_index, else_pc, end, exits));
            match attempt {
                Ok(next) => return Ok(next),
                Err(e) => {
                    if self.trace {
                        eprintln!("       condition {:?} then {then_pc} else {else_pc} failed: {e}", nodes.iter().map(|&n| self.insns[n].pc).collect::<Vec<_>>());
                    }
                    last = Some(e);
                    self.restore(&start)
                }
            }
        }
        Err(last.unwrap_or_else(|| self.unsupported("a condition that isn't an expression")))
    }

    // Which tests are the condition's own and which belong to a value inside one of its parts
    // (`x == (a and b)`): a condition test goes to the then branch, the else branch, or the
    // start of another condition test; anything else is part of an expression.
    fn condition_nodes(&self, tests: &[usize], then_pc: usize, normalize: &dyn Fn(usize) -> usize) -> Option<Vec<usize>> {
        let mut nodes = tests.to_vec();
        loop {
            let starts = nodes
                .iter()
                .enumerate()
                .map(|(n, &t)| if n == 0 { self.insns[t].pc } else { self.pc_at(nodes[n - 1] + 1) })
                .collect::<Vec<_>>();
            let first_pc = self.insns[nodes[0]].pc;
            let inner = nodes.iter().copied().find(|&t| {
                let target = normalize(self.insns[t].target().unwrap());
                target > first_pc && target < then_pc && !starts.contains(&target)
            });
            match inner {
                // the last test decides the then branch; the first starts the condition
                Some(t) if t == *nodes.last().unwrap() || t == nodes[0] => return None,
                Some(t) => nodes.retain(|&n| n != t),
                None => return Some(nodes),
            }
        }
    }

    fn condition(
        &mut self,
        tests: &[usize],
        then_pc: usize,
        else_pc: usize,
        normalize: &dyn Fn(usize) -> usize,
        statements: bool,
    ) -> Result<(Vec<Stmt>, Expr), Error> {
        let starts = tests.iter().enumerate().map(|(n, &t)| if n == 0 { self.insns[t].pc } else { self.pc_at(tests[n - 1] + 1) }).collect::<Vec<_>>();
        let mut nodes = Vec::new();
        for (n, &k) in tests.iter().enumerate() {
            let constant = Test { pred: Expr::Bool(true), jump_when: true, subject: None };
            let (pre, test) = if n == 0 {
                let test = if is_test(self.insns[k].op) { self.test(k, None)? } else { constant };
                // what was still pending runs before the `if`
                self.flush()?;
                (Vec::new(), test)
            } else {
                // a later test's own code only runs when the condition gets that far; any
                // statements it needs (an inlined body's) stay with it
                let mut test = None;
                // a value the part computes that is read after the condition (an inlined
                // `a and b or c`'s result) is a variable declared before it; assigning it
                // from inside the flag's `if` is fine
                let mut written = Regs::default();
                for i in tests[n - 1] + 1..k {
                    written = written.union(liveness::writes(&self.insns[i]));
                }
                let after = [then_pc, else_pc]
                    .into_iter()
                    .filter_map(|pc| self.index_of.get(pc).filter(|&&i| i != usize::MAX))
                    .fold(Regs::default(), |acc, &i| acc.union(self.live_in.get(i).copied().unwrap_or_default()));
                let out = written.intersect(after).iter().collect::<Vec<_>>();
                if !out.is_empty() && !statements {
                    return Err(self.unsupported("condition code with statements"));
                }
                for &r in &out {
                    if !matches!(self.slots[r as usize], Slot::Local(l) if !self.by_value[l as usize]) {
                        let local = self.new_local(r, self.pc);
                        self.body.push(Stmt::Local { locals: vec![local], values: Vec::new() });
                        self.slots[r as usize] = Slot::Local(local);
                    }
                }
                self.joins.push(out);
                let pre = self.sub_body(|l| {
                    let exits = [l.insns[k].pc];
                    l.block(tests[n - 1] + 1, k, &exits)?;
                    l.enter(k)?;
                    test = Some(if is_test(l.insns[k].op) { l.test(k, None)? } else { constant });
                    // what it computed and left over still runs in it
                    l.flush()
                });
                self.joins.pop();
                let pre = pre?;
                if !pre.is_empty() {
                    if !statements {
                        return Err(self.unsupported("condition code with statements"));
                    }
                    if pre.iter().any(Stmt::terminates) {
                        return Err(self.unsupported("a condition part that leaves"));
                    }
                    // its statements end up inside the flag's `if`: its own locals are out of
                    // scope from here
                    let mut declared = Vec::new();
                    for stmt in &pre {
                        declared_deep(stmt, &mut declared);
                    }
                    for slot in &mut self.slots {
                        if let Slot::Local(l) = slot
                            && declared.contains(l)
                        {
                            *slot = Slot::Free;
                        }
                    }
                }
                (pre, test.unwrap())
            };
            let succ = |pc: usize| -> Result<Succ, Error> {
                let pc = normalize(pc);
                if pc == then_pc {
                    Ok(Succ::Then)
                } else if pc == else_pc {
                    Ok(Succ::Else)
                } else {
                    match starts.iter().position(|&s| s == pc) {
                        Some(m) if m > n => Ok(Succ::Node(m)),
                        _ => Err(self.unsupported("a condition jump to the middle of a test")),
                    }
                }
            };
            let fall = if n + 1 < tests.len() { Succ::Node(n + 1) } else { Succ::Then };
            let jump = succ(self.insns[k].target().unwrap())?;
            if !is_test(self.insns[k].op) {
                // `x and false` folds to JUMPIFNOT x / JUMP else, and prints back the same way
                // the side it never takes still points on, so the chain stays connected
                let node = match jump {
                    Succ::Then => CondNode { pre, pred: Expr::Bool(true), t: Succ::Then, f: fall },
                    Succ::Else => CondNode { pre, pred: Expr::Bool(false), t: fall, f: Succ::Else },
                    Succ::Node(_) => return Err(self.unsupported("a constant jump inside a condition")),
                };
                nodes.push(Some(node));
                continue;
            }
            let (t, f) = if test.jump_when { (jump, fall) } else { (fall, jump) };
            nodes.push(Some(CondNode { pre, pred: test.pred, t, f }));
        }
        self.reduce_condition(nodes).ok_or_else(|| self.unsupported("a condition with no and/or shape"))
    }

    fn finish_if(&mut self, (pre, cond): (Vec<Stmt>, Expr), then_index: usize, else_pc: usize, end: usize, exits: &[usize]) -> Result<usize, Error> {
        self.body.extend(pre);
        // `if c then break end` compiles to one jump to the loop's end when c holds
        if else_pc >= FAR {
            let stmts = match else_pc {
                BREAK => vec![Stmt::Break],
                CONTINUE => {
                    self.loops.last_mut().unwrap().continued = true;
                    vec![Stmt::Continue]
                }
                _ => self.far_exit_stmts(),
            };
            self.body.push(Stmt::If { cond: tidy_condition(negate(cond)), then_body: stmts, else_body: Vec::new() });
            return Ok(then_index);
        }
        let end_pc = exits[0];
        let first = then_index.saturating_sub(1);
        let normalize = |t: usize| self.normalize(t, first, end, exits);
        // then [then_index, then_end), else [else_range), and where they meet
        let (then_end, else_range, join) = if else_pc == end_pc {
            (end, None, end)
        } else {
            let else_index = self.idx(else_pc)?;
            let last = self.insns[else_index - 1];
            match (last.op, last.target()) {
                // (a `break` or `continue` ending the then branch is no jump over an else)
                (Op::JUMP | Op::JUMPX, Some(t)) if else_index - 1 >= then_index && t > else_pc && normalize(t) < FAR => {
                    let joined = normalize(t);
                    if joined > end_pc {
                        return Err(self.unsupported("an else branch past its region"));
                    }
                    let join = if joined == end_pc { end } else { self.idx(joined)? };
                    (else_index - 1, Some((else_index, join)), join)
                }
                // no jump over an else: a then branch that returns, or no else at all
                _ => (else_index, None, else_index),
            }
        };

        // registers a branch writes and something after the join reads are one variable on
        // both paths, declared before the `if`
        let mut written = Regs::default();
        for i in then_index..then_end {
            written = written.union(liveness::writes(&self.insns[i]));
        }
        if let Some((s, e)) = else_range {
            for i in s..e {
                written = written.union(liveness::writes(&self.insns[i]));
            }
        }
        let live = self.live_in.get(join).copied().unwrap_or_default();
        let joined = written.intersect(live).iter().collect::<Vec<_>>();
        for &r in &joined {
            let assignable = matches!(self.slots[r as usize], Slot::Local(l) if !self.by_value[l as usize]);
            if !assignable {
                let local = self.new_local(r, self.pc);
                self.body.push(Stmt::Local { locals: vec![local], values: Vec::new() });
                self.slots[r as usize] = Slot::Local(local);
            }
        }
        self.joins.push(joined);

        let before = self.slots.clone();
        let then_exits = if then_end == end { exits.to_vec() } else { self.exits_from(then_end) };
        let then_body = self.sub_body(|l| {
            l.block(then_index, then_end, &then_exits)?;
            l.flush_branch()
        });
        let then_body = match then_body {
            Ok(b) => b,
            Err(e) => {
                self.joins.pop();
                return Err(e);
            }
        };
        let then_slots = std::mem::replace(&mut self.slots, before.clone());
        let mut else_body = Vec::new();
        let mut else_slots = before;
        if let Some((s, e)) = else_range {
            let else_exits = if e == end { exits.to_vec() } else { self.exits_from(e) };
            match self.sub_body(|l| {
                l.block(s, e, &else_exits)?;
                l.flush_branch()
            }) {
                Ok(b) => else_body = b,
                Err(e) => {
                    self.joins.pop();
                    return Err(e);
                }
            }
            else_slots = self.slots.clone();
        }
        let (then_returns, else_returns) = (then_body.last().is_some_and(Stmt::terminates), else_body.last().is_some_and(Stmt::terminates));
        for r in 0..self.slots.len() {
            self.slots[r] = if then_returns {
                else_slots[r]
            } else if else_returns || then_slots[r] == else_slots[r] {
                then_slots[r]
            } else {
                Slot::Free
            };
        }
        self.joins.pop();
        self.body.push(Stmt::If { cond: tidy_condition(cond), then_body, else_body });
        Ok(join)
    }

    // the end of a branch: what it left pending runs inside it
    fn flush_branch(&mut self) -> Result<(), Error> {
        let constants = self
            .pending
            .iter()
            .filter(|p| p.span == Span::One && matches!(&p.value, Value::Expr(e) if pure_constant(e)))
            .map(|p| (p.id, p.reg))
            .collect::<Vec<_>>();
        for (id, reg) in constants {
            self.pending.retain(|p| p.id != id);
            self.slots[reg as usize] = Slot::Free;
        }
        self.flush()
    }

    // ---- value regions: `x and y`, `x or y`, `if c then a else b`, `a < b` as a value ----

    // A test whose paths all meet again with one temporary register holding the result, and
    // no statements on the way, is an expression. Each path is lifted in a fork of the state;
    // anything that doesn't fit puts everything back and the caller makes an `if` of it.
    fn value_region(&mut self, first: usize, end: usize) -> Result<Option<usize>, Error> {
        let insn = self.insns[first];
        let Some(mut join) = insn.target() else { return Ok(None) };
        let mut k = first + 1;
        while k < self.insns.len() && self.insns[k].pc < join {
            let i = self.insns[k];
            let target = match i.op {
                Op::JUMP | Op::JUMPX => i.target(),
                Op::LOADB if i.c != 0 => i.target(),
                op if is_test(op) => i.target(),
                _ if is_expression(&i) => None,
                _ => return Ok(None),
            };
            if let Some(t) = target {
                if t <= i.pc {
                    return Ok(None);
                }
                join = join.max(t);
            }
            k += 1;
        }
        let Some(&join_index) = self.index_of.get(join).filter(|&&j| j != usize::MAX && j <= end) else {
            if self.trace {
                eprintln!("       no value region at {}: join {join} past the region", insn.pc);
            }
            return Ok(None);
        };
        let mut written = Regs::default();
        for i in first..join_index {
            written = written.union(liveness::writes(&self.insns[i]));
        }
        // (`x and y` tests the temporary it leaves its result in, and writes y there too)
        let live = self.live_in.get(join_index).copied().unwrap_or_default();
        let mut results = written.intersect(live).iter().collect::<Vec<_>>();
        // a value nothing reads still has to be computed: the register the last instruction
        // before the join writes
        if results.is_empty() && join_index > first + 1 {
            results = liveness::writes(&self.insns[join_index - 1]).iter().collect();
        }
        let [result] = results[..] else {
            if self.trace {
                eprintln!("       no value region at {}: results {results:?}", insn.pc);
            }
            return Ok(None);
        };
        // A region can compute a local's new value (`x = x and y`). While it runs, a pending
        // copy of the local's current value stands in the register; its value is assigned
        // after. Not for a local some closure can assign mid-region.
        let mut target = None;
        if let Slot::Local(l) = self.slots[result as usize] {
            // an open capture would see a write the region makes before running more code; a
            // write that is each path's last step (`x = a < b`) it can't tell apart
            let last_steps = (first..join_index).all(|k| {
                let i = self.insns[k];
                !liveness::writes(&i).has(result)
                    || (i.op == Op::LOADB && i.c != 0 && i.target() == Some(join))
                    || self.pc_at(k + 1) == join
                    || matches!(self.insns.get(k + 1), Some(n) if matches!(n.op, Op::JUMP | Op::JUMPX) && n.target() == Some(join))
            });
            if self.by_ref[l as usize] && !last_steps {
                if self.trace {
                    eprintln!("       no value region at {}: result R{result} is captured", insn.pc);
                }
                return Ok(None);
            }
            if self.assigns(result).is_none() && !self.live_in[first].has(result) {
                self.release(result);
            } else {
                target = Some(l);
            }
        }
        // a value still pending in the result register that the region never reads is
        // settled before it starts (it still has to be evaluated, just not in here)
        if let Slot::Pending(_) = self.slots[result as usize]
            && !self.live_in[first].has(result)
        {
            self.clear(result)?;
        }
        // older values the region reads (besides what the first test consumes) are locals by
        // then: they were evaluated before it, and reading them out of order inside it would
        // take a statement
        let mut written_before = Regs::default();
        let mut from_outside = Regs::default();
        for i in first + 1..join_index {
            from_outside = from_outside.union(liveness::reads(&self.insns[i]).without(written_before));
            written_before = written_before.union(liveness::writes(&self.insns[i]));
        }
        let newest = from_outside
            .iter()
            .filter(|&r| r != result)
            .filter_map(|r| match self.slots[r as usize] {
                Slot::Pending(id) => Some(self.index_of(id)),
                _ => None,
            })
            .max();
        if let Some(index) = newest {
            self.flush_oldest(index + 1)?;
        }

        let start = self.snapshot();
        if let Some(l) = target {
            self.slots[result as usize] = Slot::Free;
            if self.live_in[first].has(result) {
                self.push(result, Span::One, Value::Expr(Expr::Local(l)))?;
            }
        }
        match self.value_paths(first, join, result) {
            Ok(Some(expr)) => {
                match target {
                    Some(l) => {
                        self.slots[result as usize] = Slot::Local(l);
                        self.write(result, expr)?;
                    }
                    None => self.push(result, Span::One, Value::Expr(expr))?,
                }
                Ok(Some(join_index))
            }
            failed => {
                if self.trace {
                    let why = match failed {
                        Err(e) => e.to_string(),
                        _ => "no expression shape".into(),
                    };
                    eprintln!("       value region at {} ({first}..{join}) failed: {why}", insn.pc);
                }
                self.restore(&start);
                Ok(None)
            }
        }
    }

    fn value_paths(&mut self, first: usize, join: usize, result: u8) -> Result<Option<Expr>, Error> {
        let test = self.test(first, Some(result))?;
        // whatever the first test made pending runs before the region, so it may flush; the
        // paths after it may not
        let fork = self.snapshot();
        let base = self.pending.iter().map(|p| p.id).collect::<Vec<_>>();
        let old = match self.slots[result as usize] {
            Slot::Pending(id) => Some(id),
            _ => None,
        };
        let mut budget = 2000;
        let fall = self.value_path(first + 1, join, result, &fork, &base, old, &mut budget)?;
        self.restore(&fork);
        if let Some(id) = test.subject {
            self.push(result, Span::One, Value::Expr(Expr::Tested(id)))?;
        }
        let target = self.idx(self.insns[first].target().unwrap())?;
        let jump = self.value_path(target, join, result, &fork, &base, old, &mut budget)?;
        self.restore(&fork);
        let tree = Tree::Test { pred: test.pred, jump_when: test.jump_when, subject: test.subject, fall: Box::new(fall), jump: Box::new(jump) };
        let Some(expr) = build_value(tree) else { return Ok(None) };
        // the result register's old value, if the region started from one, is part of it now
        if let Some(id) = old {
            self.pending.retain(|p| p.id != id);
            self.slots[result as usize] = Slot::Free;
        }
        Ok(Some(expr))
    }

    #[allow(clippy::too_many_arguments)]
    fn value_path(
        &mut self,
        mut i: usize,
        join: usize,
        result: u8,
        fork: &Snapshot,
        base: &[u64],
        old: Option<u64>,
        budget: &mut usize,
    ) -> Result<Tree, Error> {
        loop {
            *budget = budget.checked_sub(1).ok_or_else(|| self.unsupported("value region too large"))?;
            let insn = *self.insns.get(i).ok_or_else(|| self.unsupported("value region runs off the end"))?;
            if insn.pc == join {
                return self.region_value(result, base, old).map(Tree::Value);
            }
            if insn.pc > join {
                return Err(self.unsupported("value path past its join"));
            }
            self.enter(i)?;
            let clean = |l: &Self| l.body.len() == fork.body && l.names.len() == fork.names;
            // an `x and y` of its own inside this path is just a value here
            if is_test(insn.op) && insn.a != result {
                let join_index = self.idx(join)?;
                if let Some(next) = self.value_region(i, join_index)? {
                    if !clean(self) {
                        return Err(self.unsupported("statement inside an expression"));
                    }
                    i = next;
                    continue;
                }
            }
            if is_test(insn.op) {
                let test = self.test(i, Some(result))?;
                if !clean(self) {
                    return Err(self.unsupported("statement inside an expression"));
                }
                let here = self.snapshot();
                let fall = self.value_path(i + 1, join, result, fork, base, old, budget)?;
                self.restore(&here);
                if let Some(id) = test.subject {
                    self.push(result, Span::One, Value::Expr(Expr::Tested(id)))?;
                }
                let target = self.idx(insn.target().unwrap())?;
                let jump = self.value_path(target, join, result, fork, base, old, budget)?;
                self.restore(&here);
                return Ok(Tree::Test { pred: test.pred, jump_when: test.jump_when, subject: test.subject, fall: Box::new(fall), jump: Box::new(jump) });
            }
            match insn.op {
                Op::JUMP | Op::JUMPX => i = self.idx(insn.target().unwrap())?,
                Op::LOADB if insn.c != 0 => {
                    self.write(insn.a, Expr::Bool(insn.b != 0))?;
                    i = self.idx(insn.target().unwrap())?;
                }
                _ => i += self.step(i)?,
            }
            if !clean(self) {
                return Err(self.unsupported("statement inside an expression"));
            }
        }
    }

    // at the join: the result register holds the newest pending value and everything else
    // is as the region found it
    fn region_value(&mut self, result: u8, base: &[u64], old: Option<u64>) -> Result<Expr, Error> {
        let Slot::Pending(id) = self.slots[result as usize] else { return Err(self.unsupported("no value at the join")) };
        let ids = self.pending.iter().map(|p| p.id).filter(|&p| p != id).collect::<Vec<_>>();
        let expected = base.iter().copied().filter(|&p| Some(p) != old).collect::<Vec<_>>();
        if ids != expected || self.pending.last().map(|p| p.id) != Some(id) {
            return Err(self.unsupported("a value region that leaves other values behind"));
        }
        match &self.pending.last().unwrap() {
            Pending { span: Span::One, value: Value::Expr(e), .. } => Ok(e.clone()),
            Pending { span: Span::One, value: Value::Table(fields, _), .. } => Ok(table(fields.clone())),
            _ => Err(self.unsupported("a value region ending in a value list")),
        }
    }
}


// ---- loops ----
// Each loop is one range closed by its back-edge (FORNLOOP, FORGLOOP or JUMPBACK). `break`
// jumps to just past it (Luau threads that through JUMPs), `continue` to the back-edge.
impl Lifter<'_> {
    // a backward jump to here from inside the region: a while or repeat loop starts here
    fn back_edge(&self, head: usize, end: usize) -> Option<usize> {
        let pc = self.insns[head].pc;
        (head..end).rev().find(|&b| {
            let i = self.insns[b];
            matches!(i.op, Op::JUMPBACK | Op::JUMP | Op::JUMPX) && i.target() == Some(pc) && i.pc >= pc
        })
    }

    // Lifts a loop's body. Registers it writes that the next iteration or the code after the
    // loop reads are one variable around it, declared before it, like a branch's joins.
    fn loop_body(
        &mut self,
        start: usize,
        back: usize,
        exit: usize,
        breaks: Vec<usize>,
        internal: Regs,
        vars: &[(u8, u32)],
        far: &Far,
    ) -> Result<(Vec<Stmt>, bool), Error> {
        self.flush()?;
        let mut written = Regs::default();
        for i in start..back {
            written = written.union(liveness::writes(&self.insns[i]));
        }
        let mut live = self.live_in[start].union(self.live_in.get(exit).copied().unwrap_or_default());
        if let Some((_, landing, _)) = far {
            live = live.union(self.live_in.get(*landing).copied().unwrap_or_default());
        }
        let joined = written.intersect(live).without(internal).iter().collect::<Vec<_>>();
        for &r in &joined {
            let assignable = matches!(self.slots[r as usize], Slot::Local(l) if !self.by_value[l as usize]);
            if !assignable {
                let local = self.new_local(r, self.pc);
                self.body.push(Stmt::Local { locals: vec![local], values: Vec::new() });
                self.slots[r as usize] = Slot::Local(local);
            }
        }
        self.joins.push(joined);
        let (far_pcs, flag) = match far {
            Some((pcs, _, flag)) => (pcs.clone(), Some(*flag)),
            None => (Vec::new(), None),
        };
        self.loops.push(LoopCtx { breaks, cont: self.insns[back].pc, continued: false, far: far_pcs, flag });
        let before = self.slots.clone();
        for &(reg, local) in vars {
            self.slots[reg as usize] = Slot::Local(local);
        }
        let exits = self.exits_from(back);
        let body = self.sub_body(|l| {
            l.block(start, back, &exits)?;
            l.flush_branch()
        });
        let ctx = self.loops.pop().unwrap();
        self.joins.pop();
        let body = body?;
        self.slots = before;
        Ok((body, ctx.continued))
    }

    // Jumps from inside the loop past its end and past the code that follows it: an inlined
    // function `return`ing from inside a loop. One landing place is handled, through a flag
    // declared before the loop: `f = true; break` there, and the skipped code after the loop
    // in `if not f then ... end`.
    fn far_exits(&mut self, start: usize, back: usize, exit: usize, breaks: &[usize], end: usize, exits: &[usize]) -> Result<Far, Error> {
        let exit_pc = self.pc_at(exit);
        let mut pcs = Vec::new();
        let mut landing = None;
        for k in start..back {
            let Some(t) = self.insns[k].target() else { continue };
            if t <= exit_pc || breaks.contains(&t) {
                continue;
            }
            let index = if exits.contains(&t) { end } else { self.idx(t)? };
            if index > end || landing.is_some_and(|l| l != index) {
                return Err(self.unsupported("a loop jumping out past more than its own end"));
            }
            landing = Some(index);
            if !pcs.contains(&t) {
                pcs.push(t);
            }
        }
        let Some(landing) = landing else { return Ok(None) };
        self.flush()?;
        let flag = self.new_local(u8::MAX, self.pc);
        self.body.push(Stmt::Local { locals: vec![flag], values: vec![Expr::Bool(false)] });
        Ok(Some((pcs, landing, flag)))
    }

    fn far_exit_stmts(&self) -> Vec<Stmt> {
        let flag = self.loops.last().and_then(|l| l.flag).expect("far exits only exist with a flag");
        vec![Stmt::Assign { targets: vec![Expr::Local(flag)], values: vec![Expr::Bool(true)] }, Stmt::Break]
    }

    // after a loop with a far exit: the code it skips runs only when the loop ended normally
    fn after_far(&mut self, exit: usize, far: &Far, end: usize, exits: &[usize]) -> Result<usize, Error> {
        let Some((_, landing, flag)) = far else { return Ok(exit) };
        let (landing, flag) = (*landing, *flag);
        if landing > exit {
            let region_exits = if landing == end { exits.to_vec() } else { self.exits_from(landing) };
            let mut written = Regs::default();
            for i in exit..landing {
                written = written.union(liveness::writes(&self.insns[i]));
            }
            let live = self.live_in.get(landing).copied().unwrap_or_default();
            let joined = written.intersect(live).iter().collect::<Vec<_>>();
            for &r in &joined {
                if !matches!(self.slots[r as usize], Slot::Local(l) if !self.by_value[l as usize]) {
                    let local = self.new_local(r, self.pc);
                    self.body.push(Stmt::Local { locals: vec![local], values: Vec::new() });
                    self.slots[r as usize] = Slot::Local(local);
                }
            }
            self.joins.push(joined);
            let before = self.slots.clone();
            let body = self.sub_body(|l| {
                l.block(exit, landing, &region_exits)?;
                l.flush_branch()
            });
            self.joins.pop();
            let body = body?;
            self.slots = before;
            let cond = Expr::Unary(UnOp::Not, Box::new(Expr::Local(flag)));
            self.emit(Stmt::If { cond, then_body: body, else_body: Vec::new() })?;
        }
        Ok(landing)
    }

    fn breaks_from(&self, exit: usize) -> Vec<usize> {
        let mut breaks = self.exits_from(exit);
        // a repeat's condition exits through the CLOSEUPVALS for its body's locals
        if let Some(i) = self.insns.get(exit)
            && i.op == Op::CLOSEUPVALS
        {
            breaks.extend(self.exits_from(exit + 1));
        }
        breaks
    }

    fn numeric_for(&mut self, prep: usize, end: usize, exits: &[usize]) -> Result<usize, Error> {
        let insn = self.insns[prep];
        let a = insn.a;
        // the FORNPREP's skip is threaded past JUMPs after the loop, so find the back-edge by
        // its registers: nested loops always use higher ones
        let Some(back) = (prep + 1..self.insns.len()).find(|&k| self.insns[k].op == Op::FORNLOOP && self.insns[k].a == a) else {
            return Err(self.unsupported("FORNPREP without its FORNLOOP"));
        };
        let exit = back + 1;
        // (the back-edge is threaded too: through JUMPs at the top of the body)
        let top = self.exits_from(prep + 1);
        if self.insns[back].op != Op::FORNLOOP || !self.insns[back].target().is_some_and(|t| top.contains(&t)) {
            return Err(self.unsupported("FORNPREP without its FORNLOOP"));
        }
        // evaluated from, to, step: the order the source has them
        // (the loop instruction reads its registers on every iteration: that's the loop, not
        // a reason to keep the values in locals)
        let header = [a + 2, a, a + 1];
        let mut v = self.take_ignoring(&header, &header)?.into_iter().map(Operand::expr);
        let (from, to, step) = (v.next().unwrap(), v.next().unwrap(), v.next().unwrap());
        let step = (step != Expr::Number(1.0)).then_some(step);
        let var = self.new_local(a + 2, self.insns[prep + 1].pc);
        let mut internal = Regs::default();
        internal.add_range(a as usize, a as usize + 3);
        let breaks = self.breaks_from(exit);
        let far = self.far_exits(prep + 1, back, exit, &breaks, end, exits)?;
        let (body, _) = self.loop_body(prep + 1, back, exit, breaks, internal, &[(a + 2, var)], &far)?;
        self.emit(Stmt::NumericFor { var, from, to, step, body })?;
        self.after_far(exit, &far, end, exits)
    }

    fn generic_for(&mut self, prep: usize, end: usize, exits: &[usize]) -> Result<usize, Error> {
        let insn = self.insns[prep];
        let a = insn.a;
        let back = self.idx(insn.target().unwrap())?;
        let looping = self.insns[back];
        let top = self.exits_from(prep + 1);
        if looping.op != Op::FORGLOOP || !looping.target().is_some_and(|t| top.contains(&t)) {
            return Err(self.unsupported("FORGPREP without its FORGLOOP"));
        }
        let exit = back + 1;
        // the generator, state and control: one call's three results, or values one by one
        let values = match self.slots[a as usize] {
            Slot::Pending(id) if matches!(self.pending[self.index_of(id)], Pending { reg, span: Span::Many(3), .. } if reg == a) => {
                let index = self.index_of(id);
                if index + 1 != self.pending.len() {
                    self.flush_oldest(index)?;
                }
                let pending = self.pending.pop().unwrap();
                for r in a..a + 3 {
                    self.slots[r as usize] = Slot::Free;
                }
                let Value::Expr(e) = pending.value else { return Err(self.unsupported("generic for over a value list")) };
                vec![e]
            }
            _ => {
                let header = [a, a + 1, a + 2];
                let mut values = self.take_ignoring(&header, &header)?.into_iter().map(Operand::expr).collect::<Vec<_>>();
                while values.len() > 1 && values.last() == Some(&Expr::Nil) {
                    values.pop();
                }
                values
            }
        };
        let count = (looping.aux & 0xff) as u8;
        let vars = (0..count).map(|k| (a + 3 + k, self.new_local(a + 3 + k, self.insns[prep + 1].pc))).collect::<Vec<_>>();
        let mut internal = Regs::default();
        internal.add_range(a as usize, a as usize + 3 + (count as usize).max(2));
        let breaks = self.breaks_from(exit);
        let far = self.far_exits(prep + 1, back, exit, &breaks, end, exits)?;
        let (body, _) = self.loop_body(prep + 1, back, exit, breaks, internal, &vars, &far)?;
        self.emit(Stmt::GenericFor { vars: vars.iter().map(|v| v.1).collect(), values, body })?;
        self.after_far(exit, &far, end, exits)
    }

    // `while` and `repeat` are both one range closed by a JUMPBACK. Lifted as `while true`,
    // a leading `if c then break end` makes it `while not c`, and a trailing one (with no
    // `continue`, which would skip it) `repeat ... until c`.
    fn jumpback_loop(&mut self, head: usize, back: usize, end: usize, exits: &[usize]) -> Result<usize, Error> {
        let exit = back + 1;
        let breaks = self.breaks_from(exit);
        let far = self.far_exits(head, back, exit, &breaks, end, exits)?;
        let (mut body, continued) = self.loop_body(head, back, exit, breaks, Regs::default(), &[], &far)?;
        let breaks_if = |s: Option<&Stmt>| matches!(s, Some(Stmt::If { then_body, else_body, .. }) if matches!(then_body[..], [Stmt::Break]) && else_body.is_empty());
        let stmt = if breaks_if(body.first()) {
            let Stmt::If { cond, .. } = body.remove(0) else { unreachable!() };
            Stmt::While { cond: tidy_condition(negate(cond)), body }
        } else if !continued && breaks_if(body.last()) {
            let Some(Stmt::If { cond, .. }) = body.pop() else { unreachable!() };
            Stmt::Repeat { body, cond }
        } else {
            Stmt::While { cond: Expr::Bool(true), body }
        };
        self.emit(stmt)?;
        self.after_far(exit, &far, end, exits)
    }
}

fn negate(e: Expr) -> Expr {
    match e {
        Expr::Unary(UnOp::Not, inner) => *inner,
        e => Expr::Unary(UnOp::Not, Box::new(e)),
    }
}

fn count_nots(e: &Expr) -> usize {
    match e {
        Expr::Unary(UnOp::Not, a) => 1 + count_nots(a),
        Expr::Binary(BinOp::And | BinOp::Or, a, b) => count_nots(a) + count_nots(b),
        _ => 0,
    }
}

// A condition with fewer `not`s. Only truthiness matters to a condition, so `not not x` is
// `x` here; De Morgan holds anywhere, as `not` always gives a boolean.
fn tidy_condition(e: Expr) -> Expr {
    match e {
        Expr::Unary(UnOp::Not, inner) => match *inner {
            Expr::Unary(UnOp::Not, x) => tidy_condition(*x),
            Expr::Binary(op @ (BinOp::And | BinOp::Or), a, b) => {
                let flipped = if op == BinOp::And { BinOp::Or } else { BinOp::And };
                let pushed = Expr::Binary(flipped, Box::new(tidy_condition(negate(*a.clone()))), Box::new(tidy_condition(negate(*b.clone()))));
                let kept = Expr::Unary(UnOp::Not, Box::new(Expr::Binary(op, Box::new(tidy_condition(*a)), Box::new(tidy_condition(*b)))));
                if count_nots(&pushed) < count_nots(&kept) { pushed } else { kept }
            }
            other => Expr::Unary(UnOp::Not, Box::new(tidy_condition(other))),
        },
        Expr::Binary(op @ (BinOp::And | BinOp::Or), a, b) => Expr::Binary(op, Box::new(tidy_condition(*a)), Box::new(tidy_condition(*b))),
        e => e,
    }
}

// Each test sends control to two places: another test, the then branch, or the else branch.
// Luau's conditions are and/or trees, so a test whose successor has no other predecessor
// folds into it, until one test with then/else is left. No code is duplicated. When the
// folded-in test needs statements first (an inlined body), the pair goes through a flag:
// `a and b` is `local f = a; if f then <b's statements> f = b end`, in the same order.
impl Lifter<'_> {
    fn reduce_condition(&mut self, mut nodes: Vec<Option<CondNode>>) -> Option<(Vec<Stmt>, Expr)> {
        let not = |e: Expr| Expr::Unary(UnOp::Not, Box::new(e));
        let mut scoped = Vec::new();
        loop {
            let mut changed = false;
            for a in 0..nodes.len() {
                let Some(node) = &nodes[a] else { continue };
                let preds = |b: usize, nodes: &[Option<CondNode>]| {
                    nodes.iter().flatten().filter(|n| n.t == Succ::Node(b) || n.f == Succ::Node(b)).count()
                };
                let (t, f) = (node.t, node.f);
                let merge = match (t, f) {
                    (Succ::Node(b), _) if preds(b, &nodes) == 1 => {
                        let nb = nodes[b].as_ref().unwrap();
                        if f == nb.f {
                            Some((b, true, false))
                        } else if f == nb.t {
                            Some((b, true, true))
                        } else {
                            None
                        }
                    }
                    (_, Succ::Node(b)) if preds(b, &nodes) == 1 => {
                        let nb = nodes[b].as_ref().unwrap();
                        if t == nb.t {
                            Some((b, false, false))
                        } else if t == nb.f {
                            Some((b, false, true))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let Some((b, through_true, negated)) = merge else { continue };
                let na = nodes[a].take().unwrap();
                let nb = nodes[b].take().unwrap();
                // (first, and_like): the combined truthiness is `first and b` or `first or b`
                let (first, and_like) = match (through_true, negated) {
                    // a truthy goes on to b, falsy agrees with b falsy: a and b
                    (true, false) => (na.pred, true),
                    // a falsy lands where b truthy does: (not a) or b
                    (true, true) => (not(na.pred), false),
                    // a truthy agrees with b truthy, falsy goes on to b: a or b
                    (false, false) => (na.pred, false),
                    // a truthy lands where b falsy does: (not a) and b
                    (false, true) => (not(na.pred), true),
                };
                let op = if and_like { BinOp::And } else { BinOp::Or };
                let (pre, pred) = if nb.pre.is_empty() {
                    (na.pre, Expr::Binary(op, Box::new(first), Box::new(nb.pred)))
                } else {
                    let flag = self.new_local(u8::MAX, self.pc);
                    for stmt in &nb.pre {
                        declared_deep(stmt, &mut scoped);
                    }
                    let mut pre = na.pre;
                    pre.push(Stmt::Local { locals: vec![flag], values: vec![first] });
                    let mut inner = nb.pre;
                    inner.push(Stmt::Assign { targets: vec![Expr::Local(flag)], values: vec![nb.pred] });
                    let guard = if and_like { Expr::Local(flag) } else { not(Expr::Local(flag)) };
                    pre.push(Stmt::If { cond: guard, then_body: inner, else_body: Vec::new() });
                    (pre, Expr::Local(flag))
                };
                nodes[a] = Some(CondNode { pre, pred, t: nb.t, f: nb.f });
                changed = true;
            }
            if !changed {
                break;
            }
        }
        // locals declared inside a flag's `if` are out of scope after it
        for slot in &mut self.slots {
            if let Slot::Local(l) = slot
                && scoped.contains(l)
            {
                *slot = Slot::Free;
            }
        }
        let mut alive = nodes.into_iter().flatten();
        let node = alive.next()?;
        if alive.next().is_some() {
            return None;
        }
        match (node.t, node.f) {
            (Succ::Then, Succ::Else) => Some((node.pre, node.pred)),
            (Succ::Else, Succ::Then) => Some((node.pre, not(node.pred))),
            _ => None,
        }
    }
}

fn declared_deep(stmt: &Stmt, out: &mut Vec<u32>) {
    out.extend(declares(stmt));
    for nested in stmt.bodies() {
        for s in nested {
            declared_deep(s, out);
        }
    }
}

fn build_value(tree: Tree) -> Option<Expr> {
    match tree {
        Tree::Value(e) => Some(e),
        Tree::Test { pred, jump_when, subject, fall, jump } => {
            let (t, f) = if jump_when { (jump, fall) } else { (fall, jump) };
            let (t, f) = (build_value(*t)?, build_value(*f)?);
            if let Some(id) = subject {
                if t == Expr::Tested(id) && !f.contains_tested(id) {
                    return Some(Expr::Binary(BinOp::Or, Box::new(pred), Box::new(f)));
                }
                if f == Expr::Tested(id) && !t.contains_tested(id) {
                    return Some(Expr::Binary(BinOp::And, Box::new(pred), Box::new(t)));
                }
                if t.contains_tested(id) || f.contains_tested(id) {
                    return None;
                }
            }
            Some(simplify_choice(pred, t, f))
        }
    }
}

// `if p then t else f` in the plainest form that means exactly the same
fn simplify_choice(p: Expr, t: Expr, f: Expr) -> Expr {
    let boolean = p.is_boolean();
    match (t, f) {
        (Expr::Bool(true), Expr::Bool(false)) if boolean => p,
        (Expr::Bool(true), Expr::Bool(false)) => Expr::Unary(UnOp::Not, Box::new(Expr::Unary(UnOp::Not, Box::new(p)))),
        (Expr::Bool(false), Expr::Bool(true)) => Expr::Unary(UnOp::Not, Box::new(p)),
        (t, Expr::Bool(false)) if boolean => Expr::Binary(BinOp::And, Box::new(p), Box::new(t)),
        (Expr::Bool(true), f) if boolean => Expr::Binary(BinOp::Or, Box::new(p), Box::new(f)),
        // reading a local twice is reading it once
        (t, f) if f == p && matches!(p, Expr::Local(_)) => Expr::Binary(BinOp::And, Box::new(p), Box::new(t)),
        (t, f) if t == p && matches!(p, Expr::Local(_)) => Expr::Binary(BinOp::Or, Box::new(p), Box::new(f)),
        // `if p then x or f else f` is `p and x or f`: `a and b or c` compiles to this
        (Expr::Binary(BinOp::Or, x, y), f) if *y == f => {
            Expr::Binary(BinOp::Or, Box::new(Expr::Binary(BinOp::And, Box::new(p), x)), y)
        }
        (t, f) => Expr::IfElse(Box::new(p), Box::new(t), Box::new(f)),
    }
}

fn literal(e: &Expr) -> bool {
    match e {
        Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_) => true,
        Expr::Unary(_, a) => literal(a),
        Expr::Binary(_, a, b) => literal(a) && literal(b),
        _ => false,
    }
}

// exponents POWK computes differently from pow() (or that might fold to one)
fn power_special(e: &Expr) -> bool {
    match literal_value(e) {
        Some(n) => n == 2.0 || n == 0.5 || n == 3.0,
        None => foldable(e),
    }
}

// the value of plain arithmetic on number literals, as Luau would fold it
fn literal_value(e: &Expr) -> Option<f64> {
    Some(match e {
        Expr::Number(n) => *n,
        Expr::Unary(UnOp::Neg, a) => -literal_value(a)?,
        Expr::Binary(op, a, b) => {
            let (a, b) = (literal_value(a)?, literal_value(b)?);
            match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => a / b,
                _ => return None,
            }
        }
        _ => return None,
    })
}

fn pure_constant(e: &Expr) -> bool {
    matches!(e, Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_) | Expr::Tested(_))
}

fn table(fields: Vec<Field>) -> Expr {
    Expr::Table(fields)
}
