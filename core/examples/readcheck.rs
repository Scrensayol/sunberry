// cargo run --profile harness -p sunberry-core --example readcheck -- <dir> [key]
// reads every file under <dir>, checking the invariants the lifter relies on
use std::collections::BTreeMap;

use sunberry_core::bytecode::{self, Op};

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("dir");
    let key: u8 = args.next().map(|k| k.parse().unwrap()).unwrap_or(203);
    let mut files = Vec::new();
    walk(std::path::Path::new(&dir), &mut files);
    let (mut ok, mut errors, mut versions, mut problems) = (0, BTreeMap::<String, usize>::new(), BTreeMap::new(), Vec::new());
    for file in &files {
        let data = std::fs::read(file).unwrap();
        match bytecode::read(&data, key) {
            Err(e) => *errors.entry(e.0.split(" at").next().unwrap().to_string()).or_default() += 1,
            Ok(chunk) => {
                ok += 1;
                *versions.entry((chunk.version, chunk.trailing)).or_insert(0) += 1;
                for (id, proto) in chunk.protos.iter().enumerate() {
                    let insns = proto.insns();
                    let starts: std::collections::HashSet<usize> = insns.iter().map(|i| i.pc).collect();
                    for insn in &insns {
                        if let Some(t) = insn.target() && (!starts.contains(&t) && t != proto.code.len()) {
                            problems.push(format!("{} f{id} pc{} {} -> {t} not an instruction", file.display(), insn.pc, insn.op.name()));
                        }
                        if matches!(insn.op, Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL) {
                            let call = insn.pc + 1 + insn.c as usize;
                            let lands = insns.iter().find(|i| i.pc == call).map(|i| i.op);
                            if !matches!(lands, Some(Op::CALL | Op::CALLFB)) {
                                problems.push(format!("{} f{id} pc{} fastcall skip lands on {lands:?}", file.display(), insn.pc));
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{} files, {ok} read, (version, trailing bytes): {versions:?}", files.len());
    for (e, n) in errors { println!("  {n} x {e}"); }
    println!("{} invariant problems", problems.len());
    for p in problems.iter().take(10) { println!("  {p}"); }
}

fn walk(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path).unwrap().flatten().map(|e| e.path()).collect();
        entries.sort();
        for e in entries { walk(&e, out); }
    } else if path.extension().is_some_and(|e| e == "luauc") {
        out.push(path.to_path_buf());
    }
}
