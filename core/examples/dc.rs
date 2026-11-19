// cargo run -p sunberry-core --example dc -- <file.luauc> [key] [--disasm]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let data = std::fs::read(&args[0]).unwrap();
    let key = args.get(1).and_then(|k| k.parse().ok()).unwrap_or(1);
    if args.iter().any(|a| a == "--disasm") {
        print!("{}", sunberry_core::disassemble(&data, key).unwrap());
    }
    match sunberry_core::decompile(&data, key) {
        Ok(text) => print!("{text}"),
        Err(e) => println!("ERROR: {e}"),
    }
}
