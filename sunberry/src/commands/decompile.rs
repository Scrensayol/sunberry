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

pub fn decompile(
    input: &Path,
    output: &Path,
    encode_key: u8,
    lua51: bool,
    name: Option<String>,
) -> Result<(), std::io::Error> {
    let bytecode = std::fs::read(input)?;
    // scripts are usually dumped as <ScriptName>.luauc
    let name = name.or_else(|| Some(input.file_stem()?.to_string_lossy().into_owned()));
    let out = decompile_no_io(bytecode, encode_key, lua51, name.as_deref());
    std::fs::write(output, out)
}
