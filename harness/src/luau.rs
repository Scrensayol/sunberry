use std::{cell::Cell, cell::RefCell, fmt, rc::Rc};

use mlua::{
    Lua, MultiValue, VmState,
    chunk::{ChunkMode, Compiler},
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct Profile {
    pub opt: u8,
    pub debug: u8,
}

impl Profile {
    pub const ALL: [Profile; 6] = [
        Profile { opt: 0, debug: 1 },
        Profile { opt: 1, debug: 1 },
        Profile { opt: 2, debug: 1 },
        Profile { opt: 0, debug: 2 },
        Profile { opt: 1, debug: 2 },
        Profile { opt: 2, debug: 2 },
    ];

    pub fn parse(text: &str) -> Option<Profile> {
        let rest = text.strip_prefix('O')?;
        let (opt, debug) = rest.split_once('g')?;
        let profile = Profile { opt: opt.parse().ok()?, debug: debug.parse().ok()? };
        (profile.opt <= 2 && profile.debug <= 2).then_some(profile)
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "O{}g{}", self.opt, self.debug)
    }
}

pub fn compile(source: &str, profile: Profile) -> Result<Vec<u8>, String> {
    let bytecode = Compiler::new()
        .set_optimization_level(profile.opt)
        .set_debug_level(profile.debug)
        .compile(source)
        .map_err(|e| e.to_string())?;
    // luau_compile reports a syntax error as a chunk whose first byte is 0
    match bytecode.split_first() {
        Some((0, message)) => Err(String::from_utf8_lossy(message).into_owned()),
        _ => Ok(bytecode),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Ending {
    Returned,
    Errored(String),
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub log: Vec<String>,
    pub ending: Ending,
}

impl Observation {
    pub fn render(&self) -> String {
        let mut text = self.log.join("\n");
        text.push_str(&format!("\n== {:?}\n", self.ending));
        text
    }
}

pub const STEP_BUDGET: u64 = 2_000_000;
pub const OUT_OF_MEMORY: &str = "memory limit";

// Runs bytecode in a fresh VM. print is captured, and anything that differs between two
// faithful compilations of the same program (line numbers in error positions, heap
// addresses) is normalized away.
pub fn run(bytecode: &[u8]) -> Observation {
    let lua = Lua::new();
    let _ = lua.set_memory_limit(64 << 20);

    let log = Rc::new(RefCell::new(Vec::new()));
    let steps = Rc::new(Cell::new(0u64));

    let print = {
        let log = log.clone();
        lua.create_function(move |_, args: MultiValue| {
            let parts = args.iter().map(|v| v.to_string().unwrap_or_else(|e| format!("<{e}>"))).collect::<Vec<_>>();
            log.borrow_mut().push(parts.join("\t"));
            Ok(())
        })
        .unwrap()
    };
    lua.globals().set("print", print).unwrap();

    {
        let steps = steps.clone();
        lua.set_interrupt(move |_| {
            let n = steps.get() + 1;
            steps.set(n);
            if n > STEP_BUDGET {
                Err(mlua::Error::runtime("harness: step budget exhausted"))
            } else {
                Ok(VmState::Continue)
            }
        });
    }

    let result = lua.load(bytecode).set_mode(ChunkMode::Binary).set_name("=prog").exec();
    let ending = if steps.get() > STEP_BUDGET {
        Ending::TimedOut
    } else {
        match result {
            Ok(()) => Ending::Returned,
            Err(e) => Ending::Errored(error_text(&e)),
        }
    };

    let mut addresses = Vec::new();
    let log = log.borrow().iter().map(|line| normalize(line, &mut addresses)).collect();
    let ending = match ending {
        Ending::Errored(text) => Ending::Errored(normalize(&text, &mut addresses)),
        other => other,
    };
    Observation { log, ending }
}

fn error_text(error: &mlua::Error) -> String {
    match error {
        mlua::Error::RuntimeError(text) => text.lines().next().unwrap_or_default().to_string(),
        mlua::Error::CallbackError { cause, .. } => error_text(cause),
        mlua::Error::MemoryError(_) => OUT_OF_MEMORY.to_string(),
        other => other.to_string().lines().next().unwrap_or_default().to_string(),
    }
}

// "prog:12: msg" -> "prog:?: msg" and "0x00007f..." -> "0x#1" numbered by first appearance
pub fn normalize(text: &str, addresses: &mut Vec<String>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if text[i..].starts_with("prog:") {
            let digits = bytes[i + 5..].iter().take_while(|b| b.is_ascii_digit()).count();
            if digits > 0 && bytes.get(i + 5 + digits) == Some(&b':') {
                out.push_str("prog:?:");
                i += 6 + digits;
                continue;
            }
        }
        if text[i..].starts_with("0x") {
            let hex = bytes[i + 2..].iter().take_while(|b| b.is_ascii_hexdigit()).count();
            if hex >= 6 {
                let address = &text[i..i + 2 + hex];
                let index = match addresses.iter().position(|a| a == address) {
                    Some(index) => index,
                    None => {
                        addresses.push(address.to_string());
                        addresses.len() - 1
                    }
                };
                out.push_str(&format!("0x#{}", index + 1));
                i += 2 + hex;
                continue;
            }
        }
        let c = text[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}
