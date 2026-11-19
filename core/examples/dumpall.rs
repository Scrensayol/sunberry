// cargo run --profile harness -p sunberry-core --example dumpall -- <dir> <out dir> [key]
// writes the output of every file the new core decompiles whole
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let key: u8 = args.get(2).map(|k| k.parse().unwrap()).unwrap_or(203);
    std::fs::create_dir_all(&args[1]).unwrap();
    let (mut ok, mut total) = (0, 0);
    for entry in std::fs::read_dir(&args[0]).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "luauc") { continue; }
        total += 1;
        if let Ok(text) = sunberry_core::decompile(&std::fs::read(&path).unwrap(), key) {
            ok += 1;
            std::fs::write(std::path::Path::new(&args[1]).join(path.file_stem().unwrap()).with_extension("lua"), text).unwrap();
        }
    }
    println!("{ok}/{total} written");
}
