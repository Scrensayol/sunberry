pub mod ast;
pub mod bytecode;
pub mod disasm;
pub mod inline;
mod lift;
mod liveness;
mod names;
pub mod print;

use rayon::prelude::*;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Read(bytecode::ReadError),
    // bytecode the core can't turn into source yet (or ever); never a wrong guess
    Unsupported { proto: usize, pc: usize, what: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Read(e) => write!(f, "can't read bytecode: {e}"),
            Error::Unsupported { proto, pc, what } => write!(f, "unsupported in function {proto} at pc {pc}: {what}"),
        }
    }
}

impl From<bytecode::ReadError> for Error {
    fn from(e: bytecode::ReadError) -> Error {
        Error::Read(e)
    }
}

pub fn disassemble(data: &[u8], key: u8) -> Result<String, Error> {
    Ok(disasm::disassemble(&bytecode::read(data, key)?))
}

pub fn decompile(data: &[u8], key: u8) -> Result<String, Error> {
    decompile_named(data, key, None)
}

// `script`: the script's name, which the table a module returns is named after
pub fn decompile_named(data: &[u8], key: u8, script: Option<&str>) -> Result<String, Error> {
    let chunk = bytecode::read(data, key)?;
    // functions lift independently; names are only chosen while printing, top-down
    // the first failing function by id, so errors don't depend on thread timing
    // a function that fails on its own can still lift once its inlined copies are calls again
    let lifted = (0..chunk.protos.len()).into_par_iter().map(|id| lift::lift(&chunk, id, None)).collect::<Vec<_>>();
    let mut functions = deinline(&chunk, lifted)?;
    for function in &mut functions {
        tidy(&mut function.body);
    }
    let protos = chunk.protos.iter().map(|p| p.debug_name.clone()).collect::<Vec<_>>();
    names::infer(&mut functions, &protos, chunk.main, script);
    Ok(print::print_chunk(&functions, chunk.main, key != 1))
}

// a second lift of every function with inlined copies in it, which turns those back into
// calls where they match the function exactly; any function that fails keeps its first lift.
// A function that only lifts the second time (its inlined copies were what stopped it) tells
// what it creates and where only then, so the second lifts run again with that known.
fn deinline(chunk: &bytecode::Chunk, lifted: Vec<Result<lift::Lifted, Error>>) -> Result<Vec<ast::Function>, Error> {
    if std::env::var_os("SUNBERRY_NO_DEINLINE").is_some() {
        return lifted.into_iter().map(|l| l.map(|l| l.function)).collect();
    }
    let n = chunk.protos.len();
    let mut errors = Vec::new();
    let (mut functions, mut raw): (Vec<_>, Vec<_>) = lifted
        .into_iter()
        .enumerate()
        .map(|(id, l)| match l {
            Ok(l) => {
                let raw = inline::Raw { ok: true, params: l.function.params.clone(), is_vararg: l.function.is_vararg, flags: flags(&l.function), body: l.raw };
                (l.function, raw)
            }
            Err(e) => {
                errors.push((id, e));
                let proto = &chunk.protos[id];
                let empty = ast::Function { params: Vec::new(), is_vararg: proto.is_vararg, body: Vec::new(), names: Vec::new(), param_types: Vec::new() };
                (empty, inline::Raw { ok: false, params: Vec::new(), is_vararg: proto.is_vararg, body: Vec::new(), flags: Vec::new() })
            }
        })
        .unzip();
    let first = functions.clone();
    let first_failed = errors.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    // the first failure by id that the second lift doesn't make up for
    let first_error = |ok: &dyn Fn(usize) -> bool| errors.iter().find(|(id, _)| !ok(*id)).map(|(_, e)| e.clone());
    let mut second: Vec<Option<lift::Lifted>>;
    let mut ctx;
    let mut round = 0;
    loop {
        ctx = inline::context(chunk, raw.clone(), &functions);
        if let Some(e) = first_error(&|id| !ctx.runs[id].is_empty()) {
            return Err(e);
        }
        second = (0..n)
            .into_par_iter()
            .map(|id| if ctx.runs[id].is_empty() { None } else { lift::lift(chunk, id, Some(&ctx)).ok() })
            .collect();
        round += 1;
        let fixed = first_failed.iter().copied().filter(|&id| !raw[id].ok && second[id].is_some()).collect::<Vec<_>>();
        if fixed.is_empty() || round == 4 {
            break;
        }
        for id in fixed {
            let l = second[id].as_ref().unwrap();
            functions[id] = l.function.clone();
            raw[id] = inline::Raw { ok: true, params: l.function.params.clone(), is_vararg: l.function.is_vararg, flags: flags(&l.function), body: l.raw.clone() };
        }
    }
    if let Some(e) = first_error(&|id| second[id].is_some()) {
        return Err(e);
    }
    let extra = second.iter().map(|l| l.as_ref().map(|l| l.extra.clone()).unwrap_or_default()).collect::<Vec<_>>();
    let mut out = second
        .into_iter()
        .zip(&first)
        .map(|(l, first)| l.map_or_else(|| first.clone(), |l| l.function))
        .collect::<Vec<_>>();
    let ok = inline::add_captures(chunk, &ctx, &mut out, &extra);
    if let Some(e) = first_error(&|id| ok[id]) {
        return Err(e);
    }
    for (id, ok) in ok.into_iter().enumerate() {
        if !ok {
            out[id] = first[id].clone();
        }
    }
    Ok(out)
}

fn flags(function: &ast::Function) -> Vec<bool> {
    function.params.iter().enumerate().map(|(i, _)| function.param_types.get(i).is_some_and(|t| t.as_deref() == Some("boolean"))).collect()
}

// Junk some scripts carry to slow decompilers down (`if v or v or v ... then end`, hundreds of
// times over): in a condition only truthiness counts, so `x or false` is x and an operand
// read twice in a row is read once; an `if` that reads nothing that can run code and does
// nothing either way is no code at all.
fn tidy(body: &mut Vec<ast::Stmt>) {
    use ast::Stmt;
    for stmt in body.iter_mut() {
        for nested in stmt.bodies_mut() {
            tidy(nested);
        }
        match stmt {
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => *cond = tidy_condition(cond.clone()),
            _ => {}
        }
    }
    body.retain(|stmt| !matches!(stmt, Stmt::If { cond, then_body, else_body } if then_body.is_empty() && else_body.is_empty() && inert(cond)));
}

fn tidy_condition(e: ast::Expr) -> ast::Expr {
    use ast::{BinOp, Expr};
    match e {
        Expr::Binary(op @ (BinOp::And | BinOp::Or), a, b) => {
            let (a, b) = (tidy_condition(*a), tidy_condition(*b));
            match (op, a, b) {
                (BinOp::Or, x, Expr::Bool(false)) | (BinOp::Or, Expr::Bool(false), x) | (BinOp::And, x, Expr::Bool(true)) => x,
                // `x and x` / `x or x`, also at the end of a longer chain
                (op, a, b) if inert(&b) && (a == b || matches!(&a, Expr::Binary(o, _, last) if *o == op && **last == b)) => a,
                (op, a, b) => Expr::Binary(op, Box::new(a), Box::new(b)),
            }
        }
        Expr::Unary(ast::UnOp::Not, inner) => Expr::Unary(ast::UnOp::Not, Box::new(tidy_condition(*inner))),
        e => e,
    }
}

// reading it can't run code: no metamethod, no environment lookup
fn inert(e: &ast::Expr) -> bool {
    use ast::Expr;
    match e {
        Expr::Local(_) | Expr::Upvalue(_) | Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) => true,
        Expr::Unary(ast::UnOp::Not, a) => inert(a),
        Expr::Binary(ast::BinOp::And | ast::BinOp::Or, a, b) => inert(a) && inert(b),
        _ => false,
    }
}

// for diagnostics: whether each function lifts on its own
pub fn lift_each(data: &[u8], key: u8) -> Result<Vec<Result<(), Error>>, Error> {
    let chunk = bytecode::read(data, key)?;
    Ok((0..chunk.protos.len()).into_par_iter().map(|id| lift::lift(&chunk, id, None).map(|_| ())).collect())
}
