// De-inlining. From O2 Luau copies small local functions into their callers: the arguments
// are evaluated as usual, then the callee's body is compiled in place with its parameters
// bound to registers (or folded, when the argument is a constant) and `return` writing the
// call's result registers. Nothing marks the copy, but its instructions keep the callee's
// line numbers, which says where to look.
//
// A candidate region is lifted on its own and compared with the callee's own lifted body.
// `f(args)` replaces it only when the two are the same statements and values, with each
// parameter standing for one register or constant throughout, the callee's upvalues being
// the same variables, and `f` the never-reassigned local holding the callee. Then the call
// computes exactly what the region did, whatever the line numbers suggested.

use std::collections::{HashMap, HashSet};

use crate::{
    ast::{BinOp, Capture, Closure, Expr, Field, Function, Stmt, UnOp},
    bytecode::{Chunk, Op},
};

// diagnostics: runs tried at their start, and calls made back
pub static RUNS_TRIED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static CALLS_MADE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// diagnostics: why runs stayed inlined (the first reason for each run)
pub static REASONS: std::sync::Mutex<Vec<(&'static str, usize)>> = std::sync::Mutex::new(Vec::new());
pub fn reason(why: &'static str) {
    let mut r = REASONS.lock().unwrap();
    match r.iter_mut().find(|(w, _)| *w == why) {
        Some((_, n)) => *n += 1,
        None => r.push((why, 1)),
    }
}

// a function as lifted, before the output-only passes (pinning, `do` scoping)
#[derive(Clone)]
pub struct Raw {
    // false for a function whose first lift failed: nothing is known about its body
    pub ok: bool,
    pub params: Vec<u32>,
    pub is_vararg: bool,
    pub body: Vec<Stmt>,
    // parameters the compiler's type info says are booleans
    pub flags: Vec<bool>,
}

// instructions whose lines belong to other functions' bodies, in runs without control flow
pub struct Run {
    pub start: usize,
    pub end: usize,
}

pub struct Ctx {
    pub raw: Vec<Raw>,
    // where each function is created, when that is exactly one place: an inlined copy of a
    // function creates the same nested functions again, with other captures
    pub site: Vec<Option<usize>>,
    // the variable each function's upvalue captures: (function, local) where it is declared
    pub origin: Vec<Vec<Option<(usize, u32)>>>,
    // the local each function is created into, when it is `local f = function` / `local function f`
    pub holder: Vec<Option<(usize, u32)>>,
    // functions nothing creates: Luau inlined the only use of `(function() ... end)()`, and
    // a copy that matches is called again as that expression, capturing the caller's upvalues
    pub orphan: Vec<bool>,
    pub runs: Vec<Vec<Run>>,
    // for each function, instruction index -> candidate callees there (innermost first)
    pub callees: Vec<HashMap<usize, Vec<usize>>>,
    // for each function, the functions it calls by the local they're kept in (which an
    // inlined copy of it may have had inlined into it too)
    pub calls: Vec<Vec<usize>>,
    // whether it calls one of its parameters (a function passed in can be inlined with it)
    pub calls_param: Vec<bool>,
}

impl Ctx {
    pub fn run_at(&self, proto: usize, at: usize) -> Option<&Run> {
        self.runs[proto].iter().find(|r| r.start <= at && at < r.end)
    }
}

// an inlined body never contains a RETURN of its own (its returns jump to the end), except
// where Luau copied the RETURN a jump went to over the jump: one with a copy further on
fn breaks_run(insns: &[crate::bytecode::Insn], i: usize) -> bool {
    let r = &insns[i];
    r.op == Op::RETURN && (r.b == 0 || !insns[i + 1..].iter().any(|o| o.op == Op::RETURN && o.a == r.a && o.b == r.b))
}

pub fn context(chunk: &Chunk, raw: Vec<Raw>, functions: &[Function]) -> Ctx {
    let n = chunk.protos.len();
    let mut parent = vec![usize::MAX; n];
    for (i, p) in chunk.protos.iter().enumerate() {
        for &c in &p.children {
            parent[c] = i;
        }
    }
    // a function's own lines: from where it starts to its last instruction (lines outside
    // that came from something inlined into it)
    let mut by_line = HashMap::<u32, Vec<usize>>::new();
    let mut span = vec![0u32; n];
    for (q, proto) in chunk.protos.iter().enumerate() {
        let Some(lines) = &proto.lines else { continue };
        if proto.is_vararg || proto.code.is_empty() {
            continue;
        }
        let insns = proto.insns();
        let first = proto.line_defined;
        // (its last instruction can carry an inlined function's line: then its own lines end
        // before the next function defined after it that isn't inside it)
        let mut last = lines[insns.last().unwrap().pc];
        let inside = |mut x: usize| {
            while x != usize::MAX {
                if x == q {
                    return true;
                }
                x = parent[x];
            }
            false
        };
        let next = (0..n).filter(|&o| !inside(o) && chunk.protos[o].line_defined > first).map(|o| chunk.protos[o].line_defined).min().unwrap_or(u32::MAX);
        if last < first {
            last = insns.iter().map(|i| lines[i.pc]).filter(|&l| l >= first && l < next).max().unwrap_or(first);
        }
        span[q] = last.saturating_sub(first);
        let mut own = insns.iter().map(|i| lines[i.pc]).filter(|&l| l >= first && l <= last).collect::<Vec<_>>();
        own.sort_unstable();
        own.dedup();
        for l in own {
            by_line.entry(l).or_default().push(q);
        }
    }
    for list in by_line.values_mut() {
        list.sort_by_key(|&q| (span[q], q));
    }
    let mut runs = Vec::with_capacity(n);
    let mut callees = Vec::with_capacity(n);
    for (c, proto) in chunk.protos.iter().enumerate() {
        let mut list = Vec::new();
        let mut at = HashMap::new();
        if let Some(lines) = &proto.lines {
            let mut ancestors = Vec::new();
            let mut x = parent[c];
            while x != usize::MAX {
                ancestors.push(x);
                x = parent[x];
            }
            let insns = proto.insns();
            let mut i = 0;
            while i < insns.len() {
                let start = i;
                while i < insns.len() && !breaks_run(&insns, i) {
                    let found = by_line
                        .get(&lines[insns[i].pc])
                        .map(|qs| qs.iter().copied().filter(|&q| q != c && !ancestors.contains(&q)).collect::<Vec<_>>())
                        .unwrap_or_default();
                    if found.is_empty() {
                        break;
                    }
                    at.insert(i, found);
                    i += 1;
                }
                if i > start {
                    list.push(Run { start, end: i });
                } else {
                    i += 1;
                }
            }
        }
        runs.push(list);
        callees.push(at);
    }
    // closures as the first lift found them: what each captures and where it is kept
    let mut sites = vec![0usize; n];
    let mut site = vec![None; n];
    let mut captures = vec![None; n];
    let mut holder = vec![None; n];
    for (f, function) in functions.iter().enumerate() {
        visit_closures(&function.body, &mut |closure, kept| {
            sites[closure.proto] += 1;
            site[closure.proto] = Some(f);
            captures[closure.proto] = Some(closure.captures.clone());
            holder[closure.proto] = kept.map(|l| (f, l));
        });
    }
    for x in 0..n {
        if sites[x] != 1 {
            site[x] = None;
            captures[x] = None;
            holder[x] = None;
        }
    }
    // only along functions created in one place: otherwise the variable depends on which
    fn resolve(x: usize, u: usize, site: &[Option<usize>], captures: &[Option<Vec<Capture>>], depth: usize) -> Option<(usize, u32)> {
        if depth > 64 {
            return None;
        }
        let parent = site[x]?;
        match captures[x].as_ref()?.get(u)? {
            Capture::Local(l) => Some((parent, *l)),
            Capture::Upvalue(k) => resolve(parent, *k as usize, site, captures, depth + 1),
        }
    }
    let origin: Vec<Vec<Option<(usize, u32)>>> = chunk
        .protos
        .iter()
        .enumerate()
        .map(|(x, proto)| (0..proto.num_upvalues as usize).map(|u| resolve(x, u, &site, &captures, 0)).collect())
        .collect();
    // what each function calls through an upvalue, as the functions kept there
    let mut calls = Vec::with_capacity(n);
    let mut calls_param = Vec::with_capacity(n);
    for f in 0..n {
        let mut funcs = Vec::new();
        for stmt in &raw[f].body {
            called(stmt, &mut funcs);
        }
        calls.push(
            (0..n)
                .filter(|&q| {
                    holder[q].is_some_and(|h| funcs.iter().any(|e| matches!(e, Expr::Upvalue(u) if origin[f].get(*u as usize) == Some(&Some(h)))))
                })
                .collect(),
        );
        calls_param.push(funcs.iter().any(|e| matches!(e, Expr::Local(l) if raw[f].params.contains(l))));
    }
    let orphan = crate::created(chunk).into_iter().map(|c| !c).collect();
    Ctx { raw, site, origin, holder, orphan, runs, callees, calls, calls_param }
}

// the function expressions a statement calls, nested statements included (not closures')
fn called(stmt: &Stmt, out: &mut Vec<Expr>) {
    fn expr(e: &Expr, out: &mut Vec<Expr>) {
        match e {
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            Expr::Unary(_, a) => expr(a, out),
            Expr::IfElse(a, b, c) => {
                expr(a, out);
                expr(b, out);
                expr(c, out);
            }
            Expr::Call { func, args, .. } => {
                out.push((**func).clone());
                expr(func, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::Table(fields) => fields.iter().for_each(|f| match f {
                Field::Positional(v) => expr(v, out),
                Field::Keyed(k, v) => {
                    expr(k, out);
                    expr(v, out);
                }
            }),
            _ => {}
        }
    }
    match stmt {
        Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, out)),
        Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, out)),
        Stmt::Call(e) => expr(e, out),
        Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, out),
        Stmt::NumericFor { from, to, step, .. } => {
            expr(from, out);
            expr(to, out);
            step.iter().for_each(|s| expr(s, out));
        }
        Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, out)),
        Stmt::LocalFunction { .. } | Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
    }
    for body in stmt.bodies() {
        for s in body {
            called(s, out);
        }
    }
}

fn visit_closures(body: &[Stmt], f: &mut dyn FnMut(&Closure, Option<u32>)) {
    fn expr(e: &Expr, f: &mut dyn FnMut(&Closure, Option<u32>)) {
        match e {
            Expr::Closure(c) => f(c, None),
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, f);
                expr(b, f);
            }
            Expr::Unary(_, a) => expr(a, f),
            Expr::IfElse(a, b, c) => {
                expr(a, f);
                expr(b, f);
                expr(c, f);
            }
            Expr::Call { func, args, .. } => {
                expr(func, f);
                args.iter().for_each(|a| expr(a, f));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, f);
                args.iter().for_each(|a| expr(a, f));
            }
            Expr::Table(fields) => fields.iter().for_each(|field| match field {
                Field::Positional(v) => expr(v, f),
                Field::Keyed(k, v) => {
                    expr(k, f);
                    expr(v, f);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { locals, values } if locals.len() == 1 && values.len() == 1 && matches!(values[0], Expr::Closure(_)) => {
                let Expr::Closure(c) = &values[0] else { unreachable!() };
                f(c, Some(locals[0]));
            }
            Stmt::LocalFunction { local, closure } => f(closure, Some(*local)),
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, f)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, f)),
            Stmt::Call(e) => expr(e, f),
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, f),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, f);
                expr(to, f);
                step.iter().for_each(|s| expr(s, f));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, f)),
            Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for nested in stmt.bodies() {
            visit_closures(nested, f);
        }
    }
}

// what lifting a candidate region on its own gave
pub struct Sub {
    pub stmts: Vec<Stmt>,
    // the result registers' values, one per register, or one value list over all of them
    pub values: Vec<Expr>,
    pub many: bool,
    pub targets: Vec<u8>,
    // locals holding what was in registers when the region started: register, and whether
    // it was a temporary (an argument evaluated for this call) rather than a local
    pub start: HashMap<u32, (u8, bool)>,
    // lifted with jumps to the end as `return`s of the results (see `Lifter::sandbox`)
    pub returns: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Bind {
    Unbound,
    Const(Expr),
    Reg(u8),
}

pub struct Plan {
    pub binds: Vec<Bind>,
}

// how the callee's upvalues look from the caller
pub enum Upvalues<'a> {
    // created in the caller: its closure's captures there
    Here(&'a [Capture]),
    // created in an enclosing function: the caller sees the same variables as its upvalues
    // (its own, then the new ones in `extra`, see `Lifter::extra`)
    Outer { callee: usize, caller: usize, extra: &'a [usize] },
    // created nowhere (see `Ctx::orphan`): each is whichever of the caller's own `count`
    // upvalues it is first matched with, and that one from then on
    Infer { found: &'a std::cell::RefCell<Vec<Option<u8>>>, count: usize },
}

pub fn unify(ctx: &Ctx, callee: usize, upvalues: Upvalues, sub: &Sub) -> Option<Plan> {
    let raw = &ctx.raw[callee];
    if raw.is_vararg || !raw.ok {
        return None;
    }
    let mutated = raw.params.iter().map(|&p| assigned(ctx, &raw.body, &Expr::Local(p))).collect::<Vec<_>>();
    let upvalue = |u: u8, s: &Expr| -> bool {
        match &upvalues {
            Upvalues::Here(caps) => match caps.get(u as usize) {
                Some(Capture::Local(l)) => *s == Expr::Local(*l),
                Some(Capture::Upvalue(k)) => *s == Expr::Upvalue(*k),
                None => false,
            },
            Upvalues::Outer { callee, caller, extra } => {
                let Some(Some(var)) = ctx.origin[*callee].get(u as usize) else { return false };
                let own = &ctx.origin[*caller];
                match *s {
                    Expr::Upvalue(k) if (k as usize) < own.len() => own[k as usize] == Some(*var),
                    // one the caller gets for a function it calls made back (one level down)
                    Expr::Upvalue(k) => extra.get(k as usize - own.len()).is_some_and(|&p| ctx.holder[p] == Some(*var)),
                    // the local itself, where the caller declares it
                    Expr::Local(l) => *var == (*caller, l),
                    _ => false,
                }
            }
            Upvalues::Infer { found, count } => match *s {
                Expr::Upvalue(k) if (k as usize) < *count => {
                    let mut found = found.borrow_mut();
                    match found.get_mut(u as usize) {
                        Some(slot) => *slot.get_or_insert(k) == k,
                        None => false,
                    }
                }
                _ => false,
            },
        }
    };
    let why = std::env::var_os("SUNBERRY_TRACE_INLINE").is_some();
    let matcher = || Matcher {
        ctx,
        params: raw.params.clone(),
        binds: vec![Bind::Unbound; raw.params.len()],
        mutated: mutated.clone(),
        start: &sub.start,
        upvalue: &upvalue,
        map: HashMap::new(),
        taken: HashSet::new(),
        results: None,
        returns: false,
        flags: raw.flags.clone(),
    };
    let body = normalize(&raw.body);
    let n = sub.targets.len();
    let values = &sub.values;
    let mut m = matcher();
    // both as function bodies, laid out alike: falling off the end returns nothing
    if sub.returns {
        let mut body = raw.body.clone();
        if !ends_in_return(&body) {
            body.push(Stmt::Return(Vec::new()));
        }
        m.returns = true;
        if !m.stmts(&sink(normalize(&body)), &sink(normalize(&sub.stmts))) {
            if why {
                eprintln!("   (no match as returns; binds {:?})", m.binds);
            }
            return None;
        }
    }
    let matched = sub.returns || match body.split_last() {
        // straight to one `return` at the end: its values are the result registers'
        Some((Stmt::Return(vals), prefix)) => {
            m.stmts(prefix, &sub.stmts) && {
                if n == 0 {
                    vals.iter().all(|v| m.pure(v))
                } else if sub.many {
                    vals.len() == 1 && vals[0].expands() && m.expr(&vals[0], &values[0])
                } else {
                    m.result_list(vals, values)
                }
            }
        }
        // the returns at the ends of branches: one value built the way the lifter builds it,
        // or each branch assigning the results
        Some((last @ Stmt::If { .. }, prefix)) if returns_everywhere(last) => {
            let value = (n == 1 && !sub.many).then(|| value_of(last)).flatten();
            let as_value = value.is_some_and(|v| m.stmts(prefix, &sub.stmts) && m.expr(&v, &values[0]));
            as_value || {
                m = matcher();
                let mut stmts = sub.stmts.clone();
                let mut results = Vec::new();
                for v in values {
                    let Expr::Local(r) = v else { return None };
                    // the variable the branches assign, declared before them
                    let Some(at) = stmts.iter().position(|s| matches!(s, Stmt::Local { locals, values } if values.is_empty() && locals[..] == [*r])) else {
                        return None;
                    };
                    stmts.remove(at);
                    results.push(v.clone());
                }
                m.results = Some(results);
                !sub.many && m.stmts(&body, &stmts)
            }
        }
        // no `return` at all: the results are nil
        _ => m.stmts(&body, &sub.stmts) && (n == 0 || (!sub.many && values.iter().all(|v| *v == Expr::Nil))),
    };
    if !matched {
        if why {
            eprintln!("   (no match; binds {:?})", m.binds);
        }
        return None;
    }
    // a parameter the callee assigns has a register of its own
    for (i, bind) in m.binds.iter().enumerate() {
        if let Bind::Reg(r) = bind
            && mutated[i]
            && m.binds.iter().enumerate().any(|(j, b)| j != i && *b == Bind::Reg(*r))
        {
            return None;
        }
    }
    Some(Plan { binds: m.binds })
}

// The callee as its inlined copy is laid out: an early `return` jumps to the end, so what
// follows its `if` is the `else` (`if c then return a end; rest` is
// `if c then return a else rest end`); an `if` with only an `else` is the negated `if`.
fn normalize(body: &[Stmt]) -> Vec<Stmt> {
    let mut out = Vec::new();
    for (i, stmt) in body.iter().enumerate() {
        let Stmt::If { cond, then_body, else_body } = stmt else {
            out.push(stmt.clone());
            continue;
        };
        let (t_ends, e_ends) = (ends_in_return(then_body), ends_in_return(else_body));
        if (t_ends || e_ends) && i + 1 < body.len() {
            let rest = &body[i + 1..];
            let mut t = then_body.clone();
            let mut e = else_body.clone();
            if !t_ends {
                t.extend(rest.iter().cloned());
            }
            if !e_ends {
                e.extend(rest.iter().cloned());
            }
            out.push(if_stmt(cond.clone(), normalize(&t), normalize(&e)));
            return out;
        }
        out.push(if_stmt(cond.clone(), normalize(then_body), normalize(else_body)));
    }
    out
}

// A body's last `return` after an `if` goes into the branches that don't end in one
// (`if c then a() end; return` is `if c then a(); return else return end`), all the way
// down: a branch reaching the end returns whether or not the code spells it out.
fn sink(mut body: Vec<Stmt>) -> Vec<Stmt> {
    let n = body.len();
    if n >= 2 && matches!(body[n - 1], Stmt::Return(_)) && matches!(body[n - 2], Stmt::If { .. }) {
        let ret = body.pop().unwrap();
        let Some(Stmt::If { cond, then_body, else_body }) = body.pop() else { unreachable!() };
        let with = |mut b: Vec<Stmt>| {
            if !ends_in_return(&b) {
                b.push(ret.clone());
            }
            sink(b)
        };
        body.push(Stmt::If { cond, then_body: with(then_body), else_body: with(else_body) });
    } else if let Some(Stmt::If { then_body, else_body, .. }) = body.last_mut() {
        *then_body = sink(std::mem::take(then_body));
        *else_body = sink(std::mem::take(else_body));
    }
    body
}

fn if_stmt(cond: Expr, t: Vec<Stmt>, e: Vec<Stmt>) -> Stmt {
    if t.is_empty() && !e.is_empty() {
        Stmt::If { cond: Expr::Unary(UnOp::Not, Box::new(cond)), then_body: e, else_body: t }
    } else {
        Stmt::If { cond, then_body: t, else_body: e }
    }
}

pub fn ends_in_return(body: &[Stmt]) -> bool {
    match body.last() {
        Some(Stmt::Return(_)) => true,
        Some(Stmt::If { then_body, else_body, .. }) => ends_in_return(then_body) && ends_in_return(else_body),
        _ => false,
    }
}

fn returns_everywhere(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::If { then_body, else_body, .. } if ends_in_return(then_body) && ends_in_return(else_body))
}

// an `if` tree whose branches only return one value each, as one value
fn value_of(stmt: &Stmt) -> Option<Expr> {
    fn branch(body: &[Stmt]) -> Option<Expr> {
        match body {
            [Stmt::Return(v)] if v.len() == 1 => Some(v[0].clone().single()),
            [s @ Stmt::If { .. }] => value_of(s),
            _ => None,
        }
    }
    let Stmt::If { cond, then_body, else_body } = stmt else { return None };
    Some(choose(cond, branch(then_body)?, branch(else_body)?))
}

// `if c then t else f`, with c's and/or/not laid out as tests the way Luau compiles them
fn choose(c: &Expr, t: Expr, f: Expr) -> Expr {
    match c {
        Expr::Unary(UnOp::Not, x) => choose(x, f, t),
        Expr::Binary(BinOp::And, a, b) => {
            let inner = choose(b, t, f.clone());
            choose(a, inner, f)
        }
        Expr::Binary(BinOp::Or, a, b) => {
            let inner = choose(b, t.clone(), f);
            choose(a, t, inner)
        }
        Expr::Binary(BinOp::Ne, a, b) => choose(&Expr::Unary(UnOp::Not, Box::new(Expr::Binary(BinOp::Eq, a.clone(), b.clone()))), t, f),
        atom => crate::lift::simplify_choice(atom.clone(), t, f),
    }
}

// a condition as tests: what it says about truthiness, whatever and/or/not spell it
enum CondTree {
    Leaf(bool),
    Test(Expr, Box<CondTree>, Box<CondTree>),
}

fn cond_tree(c: &Expr, t: CondTree, f: CondTree) -> CondTree {
    fn dup(t: &CondTree) -> CondTree {
        match t {
            CondTree::Leaf(b) => CondTree::Leaf(*b),
            CondTree::Test(e, a, b) => CondTree::Test(e.clone(), Box::new(dup(a)), Box::new(dup(b))),
        }
    }
    match c {
        Expr::Unary(UnOp::Not, x) => cond_tree(x, f, t),
        Expr::Binary(BinOp::And, a, b) => {
            let inner = cond_tree(b, t, dup(&f));
            cond_tree(a, inner, f)
        }
        Expr::Binary(BinOp::Or, a, b) => {
            let inner = cond_tree(b, dup(&t), f);
            cond_tree(a, t, inner)
        }
        Expr::Binary(BinOp::Ne, a, b) => cond_tree(&Expr::Binary(BinOp::Eq, a.clone(), b.clone()), f, t),
        e if literal(e) => {
            if truthy(e) {
                t
            } else {
                f
            }
        }
        atom => CondTree::Test(atom.clone(), Box::new(t), Box::new(f)),
    }
}

// whether `target` (a local of this body, or an upvalue) is assigned in it or by a closure
// it creates
fn assigned(ctx: &Ctx, body: &[Stmt], target: &Expr) -> bool {
    fn in_closure(ctx: &Ctx, c: &Closure, target: &Expr) -> bool {
        c.captures.iter().enumerate().any(|(u, cap)| {
            let seen = match cap {
                Capture::Local(l) => Expr::Local(*l),
                Capture::Upvalue(k) => Expr::Upvalue(*k),
            };
            seen == *target && (!ctx.raw[c.proto].ok || assigned(ctx, &ctx.raw[c.proto].body, &Expr::Upvalue(u as u8)))
        })
    }
    let mut found = false;
    visit_closures(body, &mut |c, _| found |= in_closure(ctx, c, target));
    found || assigns_directly(body, target)
}

fn assigns_directly(body: &[Stmt], target: &Expr) -> bool {
    body.iter().any(|s| {
        matches!(s, Stmt::Assign { targets, .. } if targets.contains(target)) || s.bodies().iter().any(|b| assigns_directly(b, target))
    })
}

struct Matcher<'a> {
    ctx: &'a Ctx,
    params: Vec<u32>,
    binds: Vec<Bind>,
    mutated: Vec<bool>,
    start: &'a HashMap<u32, (u8, bool)>,
    upvalue: &'a dyn Fn(u8, &Expr) -> bool,
    // the pattern's locals and the subject's, declared in the same places
    map: HashMap<u32, u32>,
    taken: HashSet<u32>,
    // the variables the region's branches assign the results to (`return` assigns them)
    results: Option<Vec<Expr>>,
    // a `return` against one of the region's returns of its result registers
    returns: bool,
    // parameters typed boolean: one an inlined call left out was false, not nil
    flags: Vec<bool>,
}

fn flipped(s: &Stmt) -> Stmt {
    match s {
        Stmt::If { cond, then_body, else_body } => if_stmt(cond.clone(), then_body.clone(), else_body.clone()),
        s => s.clone(),
    }
}

fn literal(e: &Expr) -> bool {
    matches!(e, Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Vector(_))
}

fn same_literal(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Number(x), Expr::Number(y)) => x.to_bits() == y.to_bits(),
        (Expr::Vector(x), Expr::Vector(y)) => x.iter().zip(y).all(|(p, q)| p.to_bits() == q.to_bits()),
        (a, b) => literal(a) && a == b,
    }
}

fn truthy(e: &Expr) -> bool {
    !matches!(e, Expr::Nil | Expr::Bool(false))
}

impl Matcher<'_> {
    fn declare(&mut self, p: u32, s: u32) -> bool {
        if self.params.contains(&p) || self.start.contains_key(&s) || self.taken.contains(&s) || self.map.contains_key(&p) {
            return false;
        }
        self.map.insert(p, s);
        self.taken.insert(s);
        true
    }

    // an expression the callee evaluates for nothing, that the region can leave out
    fn pure(&self, e: &Expr) -> bool {
        matches!(e, Expr::Local(_) | Expr::Upvalue(_)) || literal(e)
    }

    fn param(&mut self, i: usize, s: &Expr) -> bool {
        match &self.binds[i] {
            Bind::Const(c) => same_literal(c, s),
            Bind::Reg(r) => matches!(s, Expr::Local(l) if self.start.get(l).map(|x| x.0) == Some(*r)),
            Bind::Unbound => {
                if literal(s) && !self.mutated[i] {
                    self.binds[i] = Bind::Const(s.clone());
                    true
                } else if let Expr::Local(l) = s
                    && let Some(&(r, fresh)) = self.start.get(l)
                    && (fresh || !self.mutated[i])
                {
                    self.binds[i] = Bind::Reg(r);
                    true
                } else {
                    false
                }
            }
        }
    }

    // what Luau's constant folding makes of `p` once constant parameters are substituted
    fn eval(&self, p: &Expr) -> Option<Expr> {
        let number = |e: Option<Expr>| match e {
            Some(Expr::Number(n)) => Some(n),
            _ => None,
        };
        Some(match p {
            e if literal(e) => e.clone(),
            Expr::Local(l) => match self.params.iter().position(|x| x == l).map(|i| &self.binds[i]) {
                Some(Bind::Const(c)) => c.clone(),
                _ => return None,
            },
            Expr::Unary(UnOp::Neg, a) => Expr::Number(-number(self.eval(a))?),
            Expr::Unary(UnOp::Not, a) => Expr::Bool(!truthy(&self.eval(a)?)),
            Expr::Unary(UnOp::Len, a) => match self.eval(a)? {
                Expr::String(s) => Expr::Number(s.len() as f64),
                _ => return None,
            },
            Expr::Binary(op @ (BinOp::And | BinOp::Or), a, b) => {
                let a = self.eval(a)?;
                match (op, truthy(&a)) {
                    (BinOp::And, true) | (BinOp::Or, false) => self.eval(b)?,
                    _ => a,
                }
            }
            Expr::Binary(op @ (BinOp::Eq | BinOp::Ne), a, b) => {
                let (a, b) = (self.eval(a)?, self.eval(b)?);
                let eq = match (&a, &b) {
                    (Expr::Number(x), Expr::Number(y)) => x == y,
                    (Expr::Vector(_), _) | (_, Expr::Vector(_)) => return None,
                    _ => a == b,
                };
                Expr::Bool(eq == (*op == BinOp::Eq))
            }
            Expr::Binary(BinOp::Concat, a, b) => match (self.eval(a)?, self.eval(b)?) {
                (Expr::String(mut x), Expr::String(y)) => {
                    x.extend(y);
                    Expr::String(x)
                }
                _ => return None,
            },
            Expr::Binary(op, a, b) => {
                let (x, y) = (number(self.eval(a))?, number(self.eval(b))?);
                match op {
                    BinOp::Add => Expr::Number(x + y),
                    BinOp::Sub => Expr::Number(x - y),
                    BinOp::Mul => Expr::Number(x * y),
                    BinOp::Div => Expr::Number(x / y),
                    BinOp::IDiv => Expr::Number((x / y).floor()),
                    BinOp::Mod => Expr::Number(x - (x / y).floor() * y),
                    BinOp::Pow => Expr::Number(x.powf(y)),
                    BinOp::Lt => Expr::Bool(x < y),
                    BinOp::Le => Expr::Bool(x <= y),
                    BinOp::Gt => Expr::Bool(x > y),
                    BinOp::Ge => Expr::Bool(x >= y),
                    _ => return None,
                }
            }
            Expr::IfElse(c, a, b) => {
                if truthy(&self.eval(c)?) {
                    self.eval(a)?
                } else {
                    self.eval(b)?
                }
            }
            _ => return None,
        })
    }

    fn expr(&mut self, p: &Expr, s: &Expr) -> bool {
        if let Expr::Local(pl) = p
            && let Some(i) = self.params.iter().position(|x| x == pl)
        {
            return self.param(i, s);
        }
        // a part folded to a constant: only a constant pattern can match it
        if literal(s) && !literal(p) {
            return self.eval(p).is_some_and(|v| same_literal(&v, s));
        }
        match (p, s) {
            (Expr::Local(a), Expr::Local(b)) => self.map.get(a) == Some(b),
            (Expr::Upvalue(u), s) => (self.upvalue)(*u, s),
            (Expr::Global(a), Expr::Global(b)) => a == b,
            (Expr::VarArg { multret: a }, Expr::VarArg { multret: b }) => a == b,
            (Expr::Index(a, b), Expr::Index(c, d)) => self.expr(a, c) && self.expr(b, d),
            (Expr::Call { func: f, args: a, multret: m }, Expr::Call { func: g, args: b, multret: n }) => {
                m == n && self.expr(f, g) && self.list(a, b)
            }
            (
                Expr::MethodCall { object: o, method: x, args: a, multret: m },
                Expr::MethodCall { object: p, method: y, args: b, multret: n },
            ) => m == n && x == y && self.expr(o, p) && self.list(a, b),
            (Expr::Binary(o1, a, b), Expr::Binary(o2, c, d)) => o1 == o2 && self.expr(a, c) && self.expr(b, d),
            (Expr::Unary(o1, a), Expr::Unary(o2, b)) => o1 == o2 && self.expr(a, b),
            (Expr::IfElse(a, b, c), Expr::IfElse(d, e, f)) => self.expr(a, d) && self.expr(b, e) && self.expr(c, f),
            (Expr::Table(a), Expr::Table(b)) => {
                a.len() == b.len()
                    && a.iter().zip(b).all(|(x, y)| match (x, y) {
                        (Field::Positional(v), Field::Positional(w)) => self.expr(v, w),
                        (Field::Keyed(k, v), Field::Keyed(j, w)) => self.expr(k, j) && self.expr(v, w),
                        _ => false,
                    })
            }
            (Expr::Closure(a), Expr::Closure(b)) => self.closure(a, b),
            (p, s) => literal(p) && same_literal(p, s),
        }
    }

    // a `return`'s values against the result registers': extra values have to be ones the
    // region could leave out, missing ones are nil
    fn result_list(&mut self, vals: &[Expr], values: &[Expr]) -> bool {
        let (k, n) = (vals.len(), values.len());
        let expands = vals.last().is_some_and(Expr::expands);
        (0..n).all(|i| {
            if i + 1 < k {
                self.expr(&vals[i], &values[i])
            } else if i + 1 == k {
                self.expr(&vals[i].clone().single(), &values[i].clone().single())
            } else {
                !expands && values[i] == Expr::Nil
            }
        }) && vals.iter().skip(n).all(|v| self.pure(v))
    }

    fn cond(&mut self, p: &Expr, s: &Expr) -> bool {
        let (a, b) = (cond_tree(p, CondTree::Leaf(true), CondTree::Leaf(false)), cond_tree(s, CondTree::Leaf(true), CondTree::Leaf(false)));
        self.cond_trees(&a, &b)
    }

    fn cond_trees(&mut self, a: &CondTree, b: &CondTree) -> bool {
        match (a, b) {
            (CondTree::Leaf(x), CondTree::Leaf(y)) => x == y,
            (CondTree::Test(p, t1, f1), CondTree::Test(s, t2, f2)) => self.expr(p, s) && self.cond_trees(t1, t2) && self.cond_trees(f1, f2),
            _ => false,
        }
    }

    fn list(&mut self, a: &[Expr], b: &[Expr]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| self.expr(x, y))
    }

    fn closure(&mut self, a: &Closure, b: &Closure) -> bool {
        let (qa, qb) = (&self.ctx.raw[a.proto], &self.ctx.raw[b.proto]);
        if !qa.ok || !qb.ok || a.captures.len() != b.captures.len() || qa.params.len() != qb.params.len() || qa.is_vararg != qb.is_vararg {
            return false;
        }
        let as_expr = |c: &Capture| match c {
            Capture::Local(l) => Expr::Local(*l),
            Capture::Upvalue(u) => Expr::Upvalue(*u),
        };
        for (x, y) in a.captures.iter().zip(&b.captures) {
            if !self.expr(&as_expr(x), &as_expr(y)) {
                return false;
            }
        }
        // inside, upvalue k is the same capture on both sides
        let same = |u: u8, s: &Expr| *s == Expr::Upvalue(u);
        let empty = HashMap::new();
        let mut inner = Matcher {
            ctx: self.ctx,
            params: Vec::new(),
            binds: Vec::new(),
            mutated: Vec::new(),
            start: &empty,
            upvalue: &same,
            map: HashMap::new(),
            taken: HashSet::new(),
            results: None,
            returns: false,
            flags: Vec::new(),
        };
        qa.params.iter().zip(&qb.params).all(|(x, y)| inner.declare(*x, *y)) && inner.stmts(&qa.body, &qb.body)
    }

    fn stmts(&mut self, a: &[Stmt], b: &[Stmt]) -> bool {
        // a branch's `return` assigns the results (or, with none, is just the end)
        if let Some(results) = self.results.clone()
            && let Some((Stmt::Return(vals), a)) = a.split_last()
        {
            if results.is_empty() {
                return vals.iter().all(|v| self.pure(v)) && self.stmts(a, b);
            }
            let Some((Stmt::Assign { targets, values }, b)) = b.split_last() else { return false };
            return *targets == results && self.stmts(a, b) && self.result_list(vals, values);
        }
        let saved = (self.binds.clone(), self.map.clone(), self.taken.clone());
        if a.len() == b.len() && a.iter().zip(b).all(|(x, y)| self.stmt(x, y)) {
            return true;
        }
        (self.binds, self.map, self.taken) = saved;
        // Luau inlines a call with missing arguments as if they were nil (and constant ones
        // as those constants), and folds the branches that test them: `if dictionary then
        // ... end` is gone from the copy. A parameter only such tests read was nil (false
        // where it's typed boolean) or, if the branch for a truthy one is what's left, true.
        let Some(k) = (0..a.len()).find(|&k| matches!(&a[k], Stmt::If { cond, .. } if self.foldable(cond))) else { return false };
        let Stmt::If { cond, then_body, else_body } = &a[k] else { unreachable!() };
        for truthy_guess in [false, true] {
            let saved = (self.binds.clone(), self.map.clone(), self.taken.clone());
            let mut uses = Vec::new();
            visit_params(cond, &self.params, &mut uses);
            for &i in &uses {
                if self.binds[i] == Bind::Unbound {
                    let falsy = if self.flags.get(i) == Some(&true) { Expr::Bool(false) } else { Expr::Nil };
                    self.binds[i] = Bind::Const(if truthy_guess { Expr::Bool(true) } else { falsy });
                }
            }
            if let Some(v) = self.eval(cond) {
                let branch = if truthy(&v) { then_body } else { else_body };
                let folded = a[..k].iter().chain(branch).chain(&a[k + 1..]).cloned().collect::<Vec<_>>();
                // laid out again the way both sides were (a return the fold brought up)
                let folded = normalize(&folded);
                let folded = if self.returns { sink(folded) } else { folded };
                if self.stmts(&folded, b) {
                    return true;
                }
            }
            (self.binds, self.map, self.taken) = saved;
        }
        false
    }

    // a condition that reads parameters (none of them assigned in the callee) and nothing else
    // that isn't constant
    fn foldable(&mut self, cond: &Expr) -> bool {
        let mut uses = Vec::new();
        visit_params(cond, &self.params, &mut uses);
        if uses.is_empty() || uses.iter().any(|&i| self.mutated[i] || matches!(self.binds[i], Bind::Reg(_))) {
            return false;
        }
        // with the unbound ones as nil, everything else in it has to fold too
        let saved = self.binds.clone();
        for &i in &uses {
            if self.binds[i] == Bind::Unbound {
                self.binds[i] = Bind::Const(Expr::Nil);
            }
        }
        let ok = self.eval(cond).is_some();
        self.binds = saved;
        ok
    }

    fn stmt(&mut self, p: &Stmt, s: &Stmt) -> bool {
        // a `return` inside a loop leaves the loop too: not a plain assignment of the results
        if matches!(p, Stmt::While { .. } | Stmt::Repeat { .. } | Stmt::NumericFor { .. } | Stmt::GenericFor { .. }) {
            let results = self.results.take();
            let ok = self.stmt_inner(p, s);
            self.results = results;
            return ok;
        }
        self.stmt_inner(p, s)
    }

    fn stmt_inner(&mut self, p: &Stmt, s: &Stmt) -> bool {
        match (p, s) {
            (Stmt::Local { locals: a, values: va }, Stmt::Local { locals: b, values: vb }) => {
                a.len() == b.len() && self.list(va, vb) && a.iter().zip(b).all(|(x, y)| self.declare(*x, *y))
            }
            (Stmt::LocalFunction { local: a, closure: ca }, Stmt::LocalFunction { local: b, closure: cb }) => {
                self.declare(*a, *b) && self.closure(ca, cb)
            }
            (Stmt::Assign { targets: ta, values: va }, Stmt::Assign { targets: tb, values: vb }) => {
                self.list(va, vb) && self.list(ta, tb)
            }
            (Stmt::Call(a), Stmt::Call(b)) => self.expr(a, b),
            (Stmt::Return(a), Stmt::Return(b)) if self.returns => self.result_list(a, b),
            (Stmt::Return(a), Stmt::Return(b)) => self.list(a, b),
            (Stmt::If { .. }, s @ Stmt::If { .. }) => {
                let (Stmt::If { cond: c, then_body: t, else_body: e }, Stmt::If { cond: d, then_body: u, else_body: f }) = (p, &flipped(s)) else {
                    unreachable!()
                };
                // either way round: `if c then A else B` is `if not c then B else A`
                let saved = (self.binds.clone(), self.map.clone(), self.taken.clone());
                if self.cond(c, d) && self.stmts(t, u) && self.stmts(e, f) {
                    return true;
                }
                (self.binds, self.map, self.taken) = saved;
                self.cond(&Expr::Unary(UnOp::Not, Box::new(c.clone())), d) && self.stmts(e, u) && self.stmts(t, f)
            }
            (Stmt::While { cond: c, body: a }, Stmt::While { cond: d, body: b }) => self.cond(c, d) && self.stmts(a, b),
            (Stmt::Repeat { body: a, cond: c }, Stmt::Repeat { body: b, cond: d }) => self.stmts(a, b) && self.cond(c, d),
            (
                Stmt::NumericFor { var: v, from: f1, to: t1, step: s1, body: a },
                Stmt::NumericFor { var: w, from: f2, to: t2, step: s2, body: b },
            ) => {
                self.expr(f1, f2)
                    && self.expr(t1, t2)
                    && match (s1, s2) {
                        (Some(x), Some(y)) => self.expr(x, y),
                        (None, None) => true,
                        _ => false,
                    }
                    && self.declare(*v, *w)
                    && self.stmts(a, b)
            }
            (Stmt::GenericFor { vars: v, values: x, body: a }, Stmt::GenericFor { vars: w, values: y, body: b }) => {
                v.len() == w.len() && self.list(x, y) && v.iter().zip(w).all(|(p, q)| self.declare(*p, *q)) && self.stmts(a, b)
            }
            (Stmt::Break, Stmt::Break) | (Stmt::Continue, Stmt::Continue) => true,
            (Stmt::Do(a), Stmt::Do(b)) => self.stmts(a, b),
            _ => false,
        }
    }
}

// Gives functions the upvalues their de-inlined calls use (`Lifted::extra`): the closure
// that creates each one captures the callee's local, or an upvalue for it, which may in turn
// need a new upvalue in the function around it. Returns, for each function, whether all of
// its new upvalues were provided (one that wasn't has to keep its first lift).
pub fn add_captures(chunk: &Chunk, ctx: &Ctx, functions: &mut [Function], extra: &[Vec<usize>]) -> Vec<bool> {
    let n = chunk.protos.len();
    let parent = ctx.site.iter().map(|s| s.unwrap_or(usize::MAX)).collect::<Vec<_>>();
    let mut added = extra.to_vec();
    // (function, capture) for the new upvalues of each function, in order
    let mut captures: Vec<Vec<Capture>> = vec![Vec::new(); n];
    let mut ok = vec![true; n];
    let mut work = (0..n).filter(|&f| !added[f].is_empty()).collect::<Vec<_>>();
    while let Some(e) = work.pop() {
        let par = parent[e];
        while captures[e].len() < added[e].len() {
            let callee = added[e][captures[e].len()];
            let capture = if par == usize::MAX || ctx.holder[callee].is_none() {
                None
            } else if Some(par) == ctx.holder[callee].map(|h| h.0) {
                let mut found = None;
                visit_closures(&functions[par].body, &mut |c, kept| {
                    if c.proto == callee && kept.is_some() {
                        found = kept;
                    }
                });
                found.map(Capture::Local)
            } else if let Some(k) = ctx.holder[callee].and_then(|h| ctx.origin[par].iter().position(|o| *o == Some(h))) {
                Some(Capture::Upvalue(k as u8))
            } else {
                let pos = match added[par].iter().position(|&p| p == callee) {
                    Some(pos) => pos,
                    None => {
                        added[par].push(callee);
                        work.push(par);
                        added[par].len() - 1
                    }
                };
                let k = chunk.protos[par].num_upvalues as usize + pos;
                (k <= 255).then_some(Capture::Upvalue(k as u8))
            };
            match capture {
                Some(c) => captures[e].push(c),
                None => {
                    ok[e] = false;
                    break;
                }
            }
        }
    }
    // a function whose chain broke anywhere keeps its first lift
    for f in 0..n {
        let mut x = f;
        while x != usize::MAX && !added[x].is_empty() {
            if !ok[x] {
                ok[f] = false;
                break;
            }
            x = parent[x];
        }
    }
    if std::env::var_os("SUNBERRY_TRACE").is_some() {
        for e in 0..n {
            if !added[e].is_empty() {
                eprintln!("f{e} new upvalues for {:?}: {:?} ok {}", added[e], captures[e], ok[e]);
            }
        }
    }
    for e in 0..n {
        if captures[e].is_empty() || !ok[e] {
            continue;
        }
        let new = std::mem::take(&mut captures[e]);
        let par = parent[e];
        visit_closures_mut(&mut functions[par].body, &mut |c| {
            if c.proto == e {
                c.captures.extend(new.iter().copied());
            }
        });
    }
    ok
}

fn visit_closures_mut(body: &mut [Stmt], f: &mut dyn FnMut(&mut Closure)) {
    fn expr(e: &mut Expr, f: &mut dyn FnMut(&mut Closure)) {
        match e {
            Expr::Closure(c) => f(c),
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, f);
                expr(b, f);
            }
            Expr::Unary(_, a) => expr(a, f),
            Expr::IfElse(a, b, c) => {
                expr(a, f);
                expr(b, f);
                expr(c, f);
            }
            Expr::Call { func, args, .. } => {
                expr(func, f);
                args.iter_mut().for_each(|a| expr(a, f));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, f);
                args.iter_mut().for_each(|a| expr(a, f));
            }
            Expr::Table(fields) => fields.iter_mut().for_each(|field| match field {
                Field::Positional(v) => expr(v, f),
                Field::Keyed(k, v) => {
                    expr(k, f);
                    expr(v, f);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::LocalFunction { closure, .. } => f(closure),
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter_mut().for_each(|v| expr(v, f)),
            Stmt::Assign { targets, values } => targets.iter_mut().chain(values.iter_mut()).for_each(|v| expr(v, f)),
            Stmt::Call(e) => expr(e, f),
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, f),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, f);
                expr(to, f);
                step.iter_mut().for_each(|s| expr(s, f));
            }
            Stmt::GenericFor { values, .. } => values.iter_mut().for_each(|v| expr(v, f)),
            Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for nested in stmt.bodies_mut() {
            visit_closures_mut(nested, f);
        }
    }
}

// the parameters (by index) `e` reads
fn visit_params(e: &Expr, params: &[u32], out: &mut Vec<usize>) {
    match e {
        Expr::Local(l) => {
            if let Some(i) = params.iter().position(|p| p == l)
                && !out.contains(&i)
            {
                out.push(i);
            }
        }
        Expr::Unary(_, a) => visit_params(a, params, out),
        Expr::Binary(_, a, b) | Expr::Index(a, b) => {
            visit_params(a, params, out);
            visit_params(b, params, out);
        }
        Expr::IfElse(a, b, c) => {
            visit_params(a, params, out);
            visit_params(b, params, out);
            visit_params(c, params, out);
        }
        _ => {}
    }
}
