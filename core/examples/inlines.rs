// diagnostics: instruction runs whose line numbers belong to another function's body,
// i.e. what O2 inlining left behind
use std::collections::HashMap;

use sunberry_core::bytecode;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let dir = std::path::Path::new(&args[1]);
    let key = args.get(2).map_or(203, |k| k.parse().unwrap());
    let show = args.get(3).map_or(0, |k| k.parse().unwrap());
    let mut paths = if dir.is_dir() {
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect::<Vec<_>>()
    } else {
        vec![dir.to_path_buf()]
    };
    paths.sort();
    let (mut runs, mut files_with, mut insns_in, mut insns_all) = (0, 0, 0, 0);
    let mut by_len = HashMap::<usize, usize>::new();
    let mut shown = 0;
    for path in paths {
        let Ok(data) = std::fs::read(&path) else { continue };
        let Ok(chunk) = bytecode::read(&data, key) else { continue };
        let mut parent = vec![usize::MAX; chunk.protos.len()];
        for (i, p) in chunk.protos.iter().enumerate() {
            for &c in &p.children {
                parent[c] = i;
            }
        }
        let lines_of = |p: &bytecode::Proto| -> Vec<u32> {
            let mut l = p.lines.clone().unwrap_or_default();
            l.sort();
            l.dedup();
            l
        };
        let sets = chunk.protos.iter().map(lines_of).collect::<Vec<_>>();
        let mut any = false;
        for (c, proto) in chunk.protos.iter().enumerate() {
            let Some(lines) = &proto.lines else { continue };
            let ancestors = {
                let mut a = Vec::new();
                let mut x = parent[c];
                while x != usize::MAX {
                    a.push(x);
                    x = parent[x];
                }
                a
            };
            let insns = proto.insns();
            insns_all += insns.len();
            let callee = |line: u32| -> Option<usize> {
                (0..chunk.protos.len())
                    .filter(|&p| p != c && !ancestors.contains(&p) && !chunk.protos[p].is_vararg)
                    .filter(|&p| sets[p].binary_search(&line).is_ok())
                    .min_by_key(|&p| sets[p].last().unwrap() - sets[p][0])
            };
            let own = sets[c].clone();
            let _ = own;
            let mut i = 0;
            while i < insns.len() {
                let line = lines[insns[i].pc];
                // instructions on a line C's own code also uses can't be told apart; skip
                // closure creation (the definition itself)
                let Some(p) = callee(line).filter(|_| !matches!(insns[i].op, bytecode::Op::NEWCLOSURE | bytecode::Op::DUPCLOSURE)) else {
                    i += 1;
                    continue;
                };
                let start = i;
                while i < insns.len() && callee(lines[insns[i].pc]) == Some(p) {
                    i += 1;
                }
                runs += 1;
                any = true;
                insns_in += i - start;
                *by_len.entry((i - start).min(20)).or_default() += 1;
                if shown < show {
                    shown += 1;
                    println!("== {} f{c} pcs {}..{} from f{p} (line {})", path.display(), insns[start].pc, insns[i - 1].pc, line);
                }
            }
        }
        files_with += any as usize;
    }
    println!("{runs} runs in {files_with} files, {insns_in}/{insns_all} instructions");
    let mut v = by_len.into_iter().collect::<Vec<_>>();
    v.sort();
    println!("{v:?}");
}
