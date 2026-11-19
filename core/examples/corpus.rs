// cargo run --profile harness -p sunberry-core --example corpus -- <dir> [key]
// lifts every function of every file on its own: counts, refusal reasons, panics
use std::collections::BTreeMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("dir");
    let key: u8 = args.next().map(|k| k.parse().unwrap()).unwrap_or(203);
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "luauc")).collect();
    files.sort();
    std::panic::set_hook(Box::new(|_| {}));
    let (mut functions, mut lifted, mut panics, mut whole) = (0, 0, Vec::new(), 0);
    let mut reasons = BTreeMap::<String, usize>::new();
    for file in &files {
        let data = std::fs::read(file).unwrap();
        match std::panic::catch_unwind(|| sunberry_core::lift_each(&data, key)) {
            Err(e) => panics.push(format!("{}: {}", file.display(), e.downcast_ref::<String>().cloned().or(e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default())),
            Ok(Err(e)) => *reasons.entry(format!("read: {e}")).or_default() += 1,
            Ok(Ok(results)) => {
                if results.iter().all(Result::is_ok) { whole += 1; }
                for r in results {
                    functions += 1;
                    match r {
                        Ok(()) => lifted += 1,
                        Err(e) => {
                            let text = e.to_string();
                            let reason = text.split(": ").skip(1).collect::<Vec<_>>().join(": ");
                            let reason: String = reason.chars().map(|c| if c.is_ascii_digit() { 'N' } else { c }).collect();
                            *reasons.entry(reason).or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    println!("{} files ({whole} whole), {functions} functions, {lifted} lifted ({:.1}%), {} panics", files.len(), lifted as f64 * 100.0 / functions.max(1) as f64, panics.len());
    let mut reasons: Vec<_> = reasons.into_iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(&a.1));
    for (r, n) in reasons.iter().take(12) { println!("  {n:7}  {r}"); }
    for p in panics.iter().take(10) { println!("  PANIC {p}"); }
}
