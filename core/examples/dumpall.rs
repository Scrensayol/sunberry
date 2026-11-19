// cargo run --profile harness -p sunberry-core --example dumpall -- <dir> <out dir> [key]
// writes the output of every file the new core decompiles whole
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let args: Vec<String> = std::env::args().skip(1).collect();
    let key: u8 = args.get(2).map(|k| k.parse().unwrap()).unwrap_or(203);
    std::fs::create_dir_all(&args[1]).unwrap();
    let (mut ok, mut total) = (0, 0);
    for entry in std::fs::read_dir(&args[0]).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "luauc") { continue; }
        total += 1;
        let data = std::fs::read(&path).unwrap();
        let result = std::panic::catch_unwind(|| sunberry_core::decompile(&data, key));
        let Ok(result) = result else {
            println!("PANIC {}", path.display());
            continue;
        };
        if let Ok(text) = result {
            ok += 1;
            std::fs::write(std::path::Path::new(&args[1]).join(path.file_stem().unwrap()).with_extension("lua"), text).unwrap();
        }
    }
    println!("{ok}/{total} written");
    let load = |a: &std::sync::atomic::AtomicUsize| a.load(std::sync::atomic::Ordering::Relaxed);
    let mut reasons = sunberry_core::inline::REASONS.lock().unwrap().clone();
    reasons.sort_by_key(|r| std::cmp::Reverse(r.1));
    for (w, n) in reasons { println!("  {n:7}  {w}"); }
    println!("inlined runs tried {}, calls made back {}", load(&sunberry_core::inline::RUNS_TRIED), load(&sunberry_core::inline::CALLS_MADE));
}
