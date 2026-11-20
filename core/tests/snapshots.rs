// Decompiles every tests/cases/*.luauc (vanilla Luau bytecode, encode key 1, written by
// tests/compile.sh) and compares the output with the .lua next to it. When an output change
// is intended, rerun with UPDATE_SNAPSHOTS=1 to rewrite the expected files, then review the
// diff. Whether the output behaves like the source is the harness's job:
// `sunberry-harness check --decompiler next core/tests/cases`.

use std::{fs, path::Path};

fn decompile_with_threads(bytecode: &[u8], threads: usize) -> String {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| sunberry_core::decompile(bytecode, 1))
        .unwrap_or_else(|e| format!("-- refused: {e}\n"))
}

#[test]
fn snapshots() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();

    let mut cases = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "luauc"))
        .collect::<Vec<_>>();
    cases.sort();
    assert!(!cases.is_empty(), "no cases in {}", dir.display());

    let mut changed = Vec::new();
    let mut refused = Vec::new();
    for case in cases {
        let name = case.file_stem().unwrap().to_string_lossy().into_owned();
        let bytecode = fs::read(&case).unwrap();
        let output = decompile_with_threads(&bytecode, 1);
        assert_eq!(output, decompile_with_threads(&bytecode, 4), "{name} decompiles differently with 4 threads than with 1");
        if output.starts_with("-- refused") {
            refused.push(name.clone());
        }

        let expected_path = case.with_extension("lua");
        if update {
            fs::write(&expected_path, &output).unwrap();
        } else if fs::read_to_string(&expected_path).ok().as_deref() != Some(output.as_str()) {
            changed.push(name);
        }
    }

    assert!(refused.is_empty(), "the new core refused {}", refused.join(", "));
    assert!(
        changed.is_empty(),
        "output changed for {}. if that's intended, rerun with UPDATE_SNAPSHOTS=1 and review the diff",
        changed.join(", ")
    );
}
