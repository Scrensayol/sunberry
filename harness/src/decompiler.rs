use std::{io::Read, panic, process::Command, time::Duration};

use wait_timeout::ChildExt;

pub trait Decompiler: Sync {
    fn name(&self) -> &str;
    fn decompile(&self, bytecode: &[u8]) -> Result<String, String>;
}

pub struct Sunberry;

impl Decompiler for Sunberry {
    fn name(&self) -> &str {
        "sunberry"
    }

    fn decompile(&self, bytecode: &[u8]) -> Result<String, String> {
        panic::catch_unwind(|| luau_lifter::decompile_bytecode(bytecode, 1)).map_err(|e| {
            let message = e
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic");
            format!("panic: {message}")
        })
    }
}

// Any other decompiler, run as `name=program args...` where `{}` is replaced by the path
// of a raw bytecode file (appended when absent). The decompiled source is read from stdout.
pub struct External {
    name: String,
    argv: Vec<String>,
}

impl External {
    pub fn parse(spec: &str) -> Result<External, String> {
        let (name, command) = spec.split_once('=').ok_or("expected name=command")?;
        let mut argv = command.split_whitespace().map(str::to_string).collect::<Vec<_>>();
        if argv.is_empty() {
            return Err("empty command".into());
        }
        if !argv.iter().any(|a| a.contains("{}")) {
            argv.push("{}".into());
        }
        Ok(External { name: name.to_string(), argv })
    }
}

impl Decompiler for External {
    fn name(&self) -> &str {
        &self.name
    }

    fn decompile(&self, bytecode: &[u8]) -> Result<String, String> {
        let file = tempfile::Builder::new().suffix(".luauc").tempfile().map_err(|e| e.to_string())?;
        std::fs::write(file.path(), bytecode).map_err(|e| e.to_string())?;
        let path = file.path().to_string_lossy();
        let argv = self.argv.iter().map(|a| a.replace("{}", &path)).collect::<Vec<_>>();

        let mut child = Command::new(&argv[0])
            .args(&argv[1..])
            .env("RAYON_NUM_THREADS", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", argv[0]))?;
        let mut stdout = child.stdout.take().unwrap();
        let reader = std::thread::spawn(move || {
            let mut text = Vec::new();
            let _ = stdout.read_to_end(&mut text);
            text
        });
        match child.wait_timeout(Duration::from_secs(30)).map_err(|e| e.to_string())? {
            Some(status) => {
                let text = String::from_utf8_lossy(&reader.join().unwrap()).into_owned();
                if status.success() {
                    Ok(text)
                } else {
                    let mut error = String::new();
                    let _ = child.stderr.take().unwrap().read_to_string(&mut error);
                    Err(format!("exit {status}: {}", error.lines().next().unwrap_or_default()))
                }
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                Err("timeout".into())
            }
        }
    }
}
