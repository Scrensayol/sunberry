pub mod ast;
pub mod bytecode;
pub mod disasm;
mod lift;
mod liveness;
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
    let functions = (0..chunk.protos.len())
        .into_par_iter()
        .map(|id| lift::lift(&chunk, id))
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(print::print_chunk(&functions, chunk.main))
}

// for diagnostics: whether each function lifts on its own
pub fn lift_each(data: &[u8], key: u8) -> Result<Vec<Result<(), Error>>, Error> {
    let chunk = bytecode::read(data, key)?;
    Ok((0..chunk.protos.len()).into_par_iter().map(|id| lift::lift(&chunk, id).map(|_| ())).collect())
}
