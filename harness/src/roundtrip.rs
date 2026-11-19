use serde::Serialize;

use crate::{
    decompiler::Decompiler,
    luau::{self, Ending, Observation, Profile},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Pass,
    // the input itself didn't compile or ran out of steps, so it says nothing about the decompiler
    Unusable,
    DecompileFailed,
    RecompileFailed,
    Mismatch,
}

impl Kind {
    pub fn is_failure(self) -> bool {
        matches!(self, Kind::DecompileFailed | Kind::RecompileFailed | Kind::Mismatch)
    }
}

pub struct Verdict {
    pub kind: Kind,
    pub detail: String,
    pub output: Option<String>,
    pub want: Option<Observation>,
    pub got: Option<Observation>,
}

impl Verdict {
    fn new(kind: Kind, detail: impl Into<String>) -> Verdict {
        Verdict { kind, detail: detail.into(), output: None, want: None, got: None }
    }
}

// The reference is the original bytecode's behavior, never the source text: the output
// has to recompile, at the same profile, into something that prints the same thing.
pub fn check(source: &str, profile: Profile, decompiler: &dyn Decompiler) -> Verdict {
    let bytecode = match luau::compile(source, profile) {
        Ok(b) => b,
        Err(e) => return Verdict::new(Kind::Unusable, format!("input doesn't compile: {e}")),
    };
    let want = luau::run(&bytecode);
    if want.ending == Ending::TimedOut {
        return Verdict::new(Kind::Unusable, "input exhausts the step budget");
    }
    // where an allocation finally fails depends on temporaries no decompiler has to preserve
    if want.ending == Ending::Errored(luau::OUT_OF_MEMORY.into()) {
        return Verdict::new(Kind::Unusable, "input exhausts the memory limit");
    }

    let output = match decompiler.decompile(&bytecode) {
        Ok(o) => o,
        Err(e) => return Verdict { want: Some(want), ..Verdict::new(Kind::DecompileFailed, e) },
    };
    let recompiled = match luau::compile(&output, profile) {
        Ok(b) => b,
        Err(e) => {
            return Verdict { output: Some(output), want: Some(want), ..Verdict::new(Kind::RecompileFailed, e) };
        }
    };
    let got = luau::run(&recompiled);
    let (kind, detail) = if got == want { (Kind::Pass, String::new()) } else { (Kind::Mismatch, first_difference(&want, &got)) };
    Verdict { kind, detail, output: Some(output), want: Some(want), got: Some(got) }
}

fn first_difference(want: &Observation, got: &Observation) -> String {
    for (i, (w, g)) in want.log.iter().zip(&got.log).enumerate() {
        if w != g {
            return format!("line {}: want `{w}` got `{g}`", i + 1);
        }
    }
    if want.log.len() != got.log.len() {
        return format!("printed {} lines, want {}", got.log.len(), want.log.len());
    }
    format!("ended {:?}, want {:?}", got.ending, want.ending)
}
