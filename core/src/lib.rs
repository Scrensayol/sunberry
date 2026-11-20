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
    let chunk = bytecode::read(data, key)?;
    // functions lift independently; names are only chosen while printing, top-down
    // the first failing function by id, so errors don't depend on thread timing
    let lifted = (0..chunk.protos.len())
        .into_par_iter()
        .map(|id| lift::lift(&chunk, id, None))
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut functions = deinline(&chunk, lifted);
    let protos = chunk.protos.iter().map(|p| p.debug_name.clone()).collect::<Vec<_>>();
    for function in &mut functions {
        names::infer(function, &protos);
    }
    Ok(print::print_chunk(&functions, chunk.main))
}

// a second lift of every function with inlined copies in it, which turns those back into
// calls where they match the function exactly; any function that fails keeps its first lift
fn deinline(chunk: &bytecode::Chunk, lifted: Vec<lift::Lifted>) -> Vec<ast::Function> {
    if std::env::var_os("SUNBERRY_NO_DEINLINE").is_some() {
        return lifted.into_iter().map(|l| l.function).collect();
    }
    let (functions, raw): (Vec<_>, Vec<_>) = lifted
        .into_iter()
        .map(|l| {
            let raw = inline::Raw { params: l.function.params.clone(), is_vararg: l.function.is_vararg, body: l.raw };
            (l.function, raw)
        })
        .unzip();
    let ctx = inline::context(chunk, raw, &functions);
    let second = (0..functions.len())
        .into_par_iter()
        .map(|id| if ctx.runs[id].is_empty() { None } else { lift::lift(chunk, id, Some(&ctx)).ok() })
        .collect::<Vec<_>>();
    let extra = second.iter().map(|l| l.as_ref().map(|l| l.extra.clone()).unwrap_or_default()).collect::<Vec<_>>();
    let mut out = second
        .into_iter()
        .zip(&functions)
        .map(|(l, first)| l.map_or_else(|| first.clone(), |l| l.function))
        .collect::<Vec<_>>();
    let ok = inline::add_captures(chunk, &ctx, &mut out, &extra);
    for (id, ok) in ok.into_iter().enumerate() {
        if !ok {
            out[id] = functions[id].clone();
        }
    }
    out
}

// for diagnostics: whether each function lifts on its own
pub fn lift_each(data: &[u8], key: u8) -> Result<Vec<Result<(), Error>>, Error> {
    let chunk = bytecode::read(data, key)?;
    Ok((0..chunk.protos.len()).into_par_iter().map(|id| lift::lift(&chunk, id, None).map(|_| ())).collect())
}
