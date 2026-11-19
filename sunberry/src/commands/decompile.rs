use std::path::Path;

use base64::prelude::*;

pub fn decompile_no_io<T>(bytecode: T, encode_key: u8, lua51: bool, name: Option<&str>) -> String
where
    T: Into<Vec<u8>>,
{
    let mut bytecode = bytecode.into();
    if let Ok(decoded) = BASE64_STANDARD.decode(&bytecode) {
        bytecode = decoded;
    }

    if lua51 {
        lua51_lifter::decompile_bytecode(&bytecode)
    } else {
        luau_lifter::decompile_script(&bytecode, encode_key, name)
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
    core: super::Core,
) -> Result<(), std::io::Error> {
    if core == super::Core::Next && !lua51 {
        let out = match sunberry_core::decompile(&read_bytecode(input)?, encode_key) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("next core: {e}");
                format!("-- next core: {e}\n")
            }
        };
        return std::fs::write(output, out);
    }
    let bytecode = std::fs::read(input)?;
    // scripts are usually dumped as <ScriptName>.luauc
    let name = name.or_else(|| Some(input.file_stem()?.to_string_lossy().into_owned()));
    let out = decompile_no_io(bytecode, encode_key, lua51, name.as_deref());
    std::fs::write(output, out)
}

pub fn disasm(input: &Path, encode_key: u8) -> Result<(), std::io::Error> {
    match sunberry_core::disassemble(&read_bytecode(input)?, encode_key) {
        Ok(text) => print!("{text}"),
        Err(e) => eprintln!("{e}"),
    }
    Ok(())
}
