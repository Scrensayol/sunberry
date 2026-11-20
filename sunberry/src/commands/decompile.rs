use std::path::Path;

use base64::prelude::*;

use super::Core;

pub fn decompile_no_io<T>(bytecode: T, encode_key: u8, lua51: bool, name: Option<&str>, core: Core) -> String
where
    T: Into<Vec<u8>>,
{
    let mut bytecode = bytecode.into();
    if let Ok(decoded) = BASE64_STANDARD.decode(&bytecode) {
        bytecode = decoded;
    }

    if lua51 {
        return lua51_lifter::decompile_bytecode(&bytecode);
    }
    match core {
        Core::Old => luau_lifter::decompile_script(&bytecode, encode_key, name),
        Core::Next => match next(&bytecode, encode_key) {
            Ok(text) => text,
            // the new core refuses what it can't do exactly; the old one still gives something
            // to read, marked so nobody mistakes it for an exact decompile
            Err(why) => {
                let old = luau_lifter::decompile_script(&bytecode, encode_key, name);
                format!("-- sunberry: the new core refused this script ({why})\n-- below is the old core's output, which may not be exact\n\n{old}")
            }
        },
    }
}

fn next(bytecode: &[u8], encode_key: u8) -> Result<String, String> {
    match std::panic::catch_unwind(|| sunberry_core::decompile(bytecode, encode_key)) {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("it panicked".to_string()),
    }
}

fn read_bytecode(input: &Path) -> Result<Vec<u8>, std::io::Error> {
    let bytecode = std::fs::read(input)?;
    Ok(BASE64_STANDARD.decode(&bytecode).unwrap_or(bytecode))
}

pub fn decompile(
    input: &Path,
    output: &Path,
    encode_key: u8,
    lua51: bool,
    name: Option<String>,
    core: Core,
) -> Result<(), std::io::Error> {
    let bytecode = std::fs::read(input)?;
    // scripts are usually dumped as <ScriptName>.luauc
    let name = name.or_else(|| Some(input.file_stem()?.to_string_lossy().into_owned()));
    let out = decompile_no_io(bytecode, encode_key, lua51, name.as_deref(), core);
    std::fs::write(output, out)
}

pub fn disasm(input: &Path, encode_key: u8) -> Result<(), std::io::Error> {
    match sunberry_core::disassemble(&read_bytecode(input)?, encode_key) {
        Ok(text) => print!("{text}"),
        Err(e) => eprintln!("{e}"),
    }
    Ok(())
}
