// Undoes rewrites that keep what a function does but not the shapes Luau's compiler writes
// (Rosetta's hard tier, and obfuscators doing the same): code nothing reaches, no-ops and jumps
// to the next instruction, branches that always go one way, instructions swapped for others that
// do the same, and registers moved away from where the compiler puts them. The lifter replays
// the compiler, so it needs those shapes back. Only used when a script doesn't lift as it is.

use crate::bytecode::{Constant, Insn, Op, Proto};

#[derive(Clone, Copy)]
struct N {
    op: Op,
    a: u8,
    b: u8,
    c: u8,
    d: i32,
    aux: u32,
    // the instruction it jumps to, by index
    target: Option<usize>,
    line: u32,
    old_pc: usize,
    gone: bool,
    // for a FORNLOOP nothing reaches, kept as where its loop ends: its FORNPREP's index
    paired: Option<usize>,
}

// D shares its bits with B and C
fn set_d(n: &mut N, d: i32) {
    n.d = d;
    n.b = d as u16 as u8;
    n.c = (d as u16 >> 8) as u8;
}

fn word_of(n: &N) -> u32 {
    n.op as u32 | (n.a as u32) << 8 | (n.b as u32) << 16 | (n.c as u32) << 24
}

// never falls through to the next instruction
fn ends(n: &N) -> bool {
    matches!(n.op, Op::JUMP | Op::JUMPBACK | Op::JUMPX | Op::RETURN | Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT)
        || n.op == Op::LOADB && n.c != 0
}

// where an instruction's jump sits: D, E (JUMPX) or C (a skip, LOADB and the fast calls)
fn jumps_in_c(op: Op) -> bool {
    matches!(op, Op::LOADB | Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL)
}

fn decode(p: &Proto) -> Vec<N> {
    let insns = p.insns();
    let mut index_of = vec![usize::MAX; p.code.len() + 1];
    for (k, i) in insns.iter().enumerate() {
        index_of[i.pc] = k;
    }
    index_of[p.code.len()] = insns.len();
    let target = |i: &Insn| -> Option<usize> {
        let t = if jumps_in_c(i.op) && i.op != Op::LOADB { Some(i.pc + 1 + i.c as usize) } else { i.target() };
        t.and_then(|t| index_of.get(t).copied()).filter(|&k| k != usize::MAX && k < insns.len())
    };
    insns
        .iter()
        .map(|i| N {
            op: i.op,
            a: i.a,
            b: i.b,
            c: i.c,
            d: i.d(),
            aux: i.aux,
            target: target(i),
            line: p.lines.as_ref().and_then(|l| l.get(i.pc).copied()).unwrap_or(0),
            old_pc: i.pc,
            gone: false,
            paired: None,
        })
        .collect()
}

// the surviving instruction control reaches from k on: k itself, or the next one that isn't gone
fn live_from(ns: &[N], mut k: usize) -> usize {
    while k < ns.len() && ns[k].gone {
        k += 1;
    }
    k
}

// the registers an instruction names: bit offsets in its word and its AUX word, and the run of
// registers it reaches from its first one (first, how many; -1 for "to the top")
fn reg_fields(n: &N) -> (Vec<u32>, Vec<u32>, Option<(u8, i32)>) {
    use Op::*;
    let (a, b, c) = (8, 16, 24);
    let mut main = Vec::new();
    let mut aux = Vec::new();
    let a_is_reg = !matches!(
        n.op,
        NOP | BREAK | JUMP | JUMPBACK | PREPVARARGS | JUMPX | COVERAGE | CAPTURE | FASTCALL | FASTCALL1 | FASTCALL2 | FASTCALL2K | FASTCALL3 | FASTPCALL
    );
    if a_is_reg {
        main.push(a);
    }
    match n.op {
        MOVE | GETTABLE | SETTABLE | GETTABLEKS | SETTABLEKS | GETTABLEN | SETTABLEN | NAMECALL | ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK
        | ANDK | ORK | NOT | MINUS | LENGTH | SETLIST => main.push(b),
        ADD | SUB | MUL | DIV | MOD | POW | IDIV | AND | OR | CONCAT => main.extend([b, c]),
        SUBRK | DIVRK => main.push(c),
        FASTCALL1 | FASTCALL2 | FASTCALL2K | FASTCALL3 => main.push(b),
        CAPTURE if n.a <= 1 => main.push(b),
        _ => {}
    }
    if matches!(n.op, GETTABLE | SETTABLE) {
        main.push(c);
    }
    match n.op {
        JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT | FASTCALL2 => aux.push(0),
        FASTCALL3 => aux.extend([0, 8]),
        _ => {}
    }
    let range = match n.op {
        CALL => {
            let (b, c) = (n.b as i32, n.c as i32);
            if b == 0 || c == 0 { Some((n.a, -1)) } else { Some((n.a, (b - 1).max(c - 1) + 1)) }
        }
        RETURN | GETVARARGS => Some((n.a, if n.b == 0 { -1 } else { n.b as i32 - 1 })),
        SETLIST => Some((n.b, if n.c == 0 { -1 } else { n.c as i32 - 1 })),
        CONCAT => Some((n.b, n.c as i32 - n.b as i32 + 1)),
        NAMECALL => Some((n.a, 2)),
        FORNPREP | FORNLOOP => Some((n.a, 3)),
        FORGLOOP => Some((n.a, 3 + (n.aux & 0xff) as i32)),
        FORGPREP | FORGPREP_INEXT | FORGPREP_NEXT => Some((n.a, 5)),
        LOADNIL => Some((n.a, 1)),
        _ => None,
    };
    (main, aux, range)
}

fn regs_of(n: &N) -> Vec<u8> {
    let (main, aux, range) = reg_fields(n);
    let w = word_of(n);
    let mut out = main.iter().map(|&s| (w >> s) as u8).chain(aux.iter().map(|&s| (n.aux >> s) as u8)).collect::<Vec<_>>();
    if let Some((s, k)) = range
        && k > 0
    {
        out.extend((s as u32..(s as u32 + k as u32).min(256)).map(|r| r as u8));
    }
    out
}

fn remap(n: &mut N, map: &[u8; 256]) {
    let (main, aux, _) = reg_fields(n);
    let mut w = word_of(n);
    for s in main {
        let r = (w >> s) as u8;
        w = (w & !(0xff << s)) | (map[r as usize] as u32) << s;
    }
    n.a = (w >> 8) as u8;
    n.b = (w >> 16) as u8;
    n.c = (w >> 24) as u8;
    for s in aux {
        let r = (n.aux >> s) as u8;
        n.aux = (n.aux & !(0xff << s)) | (map[r as usize] as u32) << s;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Known {
    Nil,
    Bool(bool),
    Num(f64),
}

impl Known {
    fn truthy(self) -> bool {
        !matches!(self, Known::Nil | Known::Bool(false))
    }
}

// Branches whose operands are constants worked out in the same straight run (a scratch register
// loaded with a boolean and tested, b <= a * a) always go one way: they become a jump, or
// nothing. Returns whether any did.
fn fold_branches(ns: &mut [N], p: &Proto) -> bool {
    let mut leader = vec![false; ns.len() + 1];
    for n in ns.iter() {
        if let Some(t) = n.target {
            leader[t] = true;
        }
    }
    let mut known: [Option<Known>; 256] = [None; 256];
    let mut changed = false;
    for k in 0..ns.len() {
        if leader[k] || (k > 0 && ends(&ns[k - 1])) {
            known = [None; 256];
        }
        let n = ns[k];
        if n.gone {
            continue;
        }
        let num = |r: u8, known: &[Option<Known>; 256]| match known[r as usize] {
            Some(Known::Num(x)) => Some(x),
            _ => None,
        };
        // Some(true): always jumps, Some(false): never
        let decided = match n.op {
            Op::JUMPIF => known[n.a as usize].map(Known::truthy),
            Op::JUMPIFNOT => known[n.a as usize].map(|v| !v.truthy()),
            Op::JUMPIFLE | Op::JUMPIFLT | Op::JUMPIFNOTLE | Op::JUMPIFNOTLT => match (num(n.a, &known), num(n.aux as u8, &known)) {
                (Some(x), Some(y)) => Some(match n.op {
                    Op::JUMPIFLE => x <= y,
                    Op::JUMPIFLT => x < y,
                    Op::JUMPIFNOTLE => !(x <= y),
                    _ => !(x < y),
                }),
                _ => None,
            },
            _ => None,
        };
        if let Some(jumps) = decided {
            if jumps {
                ns[k].op = Op::JUMP;
                ns[k].a = 0;
            } else {
                ns[k].gone = true;
            }
            changed = true;
            known = [None; 256];
            continue;
        }
        // what this instruction leaves in registers
        let value = match n.op {
            Op::LOADNIL => Some(Known::Nil),
            Op::LOADB if n.c == 0 => Some(Known::Bool(n.b != 0)),
            Op::LOADN => Some(Known::Num(n.d as f64)),
            Op::LOADK => match p.constants.get(n.d as u16 as usize) {
                Some(Constant::Nil) => Some(Known::Nil),
                Some(Constant::Boolean(b)) => Some(Known::Bool(*b)),
                Some(Constant::Number(x)) => Some(Known::Num(*x)),
                _ => None,
            },
            Op::MUL | Op::ADD | Op::SUB => match (num(n.b, &known), num(n.c, &known)) {
                (Some(x), Some(y)) => Some(Known::Num(match n.op {
                    Op::MUL => x * y,
                    Op::ADD => x + y,
                    _ => x - y,
                })),
                _ => None,
            },
            _ => None,
        };
        // (anything else may write registers it names, or past them: forget what it touches)
        let regs = regs_of(&n);
        match n.op {
            Op::CALL | Op::GETVARARGS if regs.is_empty() || n.c == 0 || n.b == 0 => known = [None; 256],
            Op::CALL | Op::NAMECALL | Op::FORGLOOP | Op::FORNLOOP | Op::FORNPREP | Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT | Op::CONCAT => {
                for r in n.a as usize..256 {
                    known[r] = None;
                }
            }
            _ => {
                for r in regs {
                    known[r as usize] = None;
                }
            }
        }
        if let Some(v) = value {
            known[n.a as usize] = Some(v);
        }
    }
    changed
}

fn falls(n: &N) -> bool {
    !ends(n)
}

// marks everything control never reaches as gone
fn drop_unreachable(ns: &mut [N]) {
    let mut seen = vec![false; ns.len()];
    let mut work = vec![live_from(ns, 0)];
    while let Some(k) = work.pop() {
        if k >= ns.len() || seen[k] {
            continue;
        }
        seen[k] = true;
        if let Some(t) = ns[k].target {
            work.push(live_from(ns, t));
        }
        if falls(&ns[k]) {
            work.push(live_from(ns, k + 1));
        }
    }
    // (but a loop's FORNLOOP, which the compiler leaves after a body that always breaks: it's
    // where the loop ends, and FORNLOOP only ever means that)
    for k in 0..ns.len() {
        if seen[k] && ns[k].op == Op::FORNPREP {
            // (paired by register and the `for` line both carry: the compiler may thread
            // FORNLOOP's jump back through a body that's only a jump)
            let body = live_from(ns, k + 1);
            let pair = |l: usize| ns[l].op == Op::FORNLOOP && ns[l].a == ns[k].a;
            let found = (0..ns.len())
                .find(|&l| pair(l) && ns[l].target.map(|t| live_from(ns, t)) == Some(body))
                .or_else(|| (0..ns.len()).find(|&l| pair(l) && !seen[l] && ns[l].line == ns[k].line));
            if let Some(l) = found
                && !seen[l]
            {
                seen[l] = true;
                ns[l].paired = Some(k);
            }
        }
    }
    for (k, n) in ns.iter_mut().enumerate() {
        if !seen[k] {
            n.gone = true;
        }
    }
}

fn read_anywhere(ns: &[N], r: u8, except: &[usize]) -> bool {
    ns.iter().enumerate().any(|(k, n)| !n.gone && !except.contains(&k) && reads(n).contains(&r))
}

// the registers an instruction reads (a rough superset is fine: it only keeps code)
fn reads(n: &N) -> Vec<u8> {
    let mut out = regs_of(n);
    // (open arguments, a return list or a SETLIST read everything from their first register up,
    // see reads_at; a call's open results, and GETVARARGS's, are only written)
    match n.op {
        Op::CALL => {
            out = vec![n.a];
            if n.b == 0 {
                out.extend(n.a..=255);
            } else {
                out.extend((n.a as u32..n.a as u32 + n.b as u32).map(|r| r as u8));
            }
        }
        Op::RETURN | Op::SETLIST => {
            if let (_, _, Some((s, -1))) = reg_fields(n) {
                out.extend(s..=255);
            }
        }
        _ => {}
    }
    let writes_only = matches!(n.op, Op::LOADNIL | Op::LOADB | Op::LOADN | Op::LOADK | Op::LOADKX | Op::GETGLOBAL | Op::GETUPVAL | Op::GETIMPORT | Op::NEWTABLE | Op::DUPTABLE | Op::NEWCLOSURE | Op::DUPCLOSURE | Op::GETVARARGS);
    if writes_only {
        out.clear();
    } else if matches!(n.op, Op::MOVE | Op::GETTABLE | Op::GETTABLEKS | Op::GETTABLEN | Op::ADD | Op::SUB | Op::MUL | Op::DIV | Op::MOD | Op::POW | Op::IDIV | Op::ADDK | Op::SUBK | Op::MULK | Op::DIVK | Op::MODK | Op::POWK | Op::IDIVK | Op::AND | Op::OR | Op::ANDK | Op::ORK | Op::NOT | Op::MINUS | Op::LENGTH | Op::SUBRK | Op::DIVRK)
        && !out.iter().skip(1).any(|&r| r == n.a)
    {
        out.remove(0);
    }
    out
}

// Loads nothing ever reads: the scratch registers an always-one-way branch tested, once the
// branch is folded. A register qualifies when everything writing it is a constant load or
// arithmetic on numbers, and everything reading it is that arithmetic for another such register.
// what instruction k reads: a run to the top only up to where the open results it takes start,
// which the instruction just before it (a call or GETVARARGS) wrote itself
fn reads_at(ns: &[N], k: usize) -> Vec<u8> {
    let n = &ns[k];
    if let (_, _, Some((s, -1))) = reg_fields(n)
        && matches!(n.op, Op::CALL | Op::RETURN | Op::SETLIST)
        && let Some(prev) = (0..k).rev().find(|&j| !ns[j].gone)
        && let producer = ns[prev]
        && (producer.op == Op::CALL && producer.c == 0 || producer.op == Op::GETVARARGS && producer.b == 0)
        && producer.a >= s
    {
        let mut out = reads(n).into_iter().filter(|&r| r < s).collect::<Vec<_>>();
        out.extend(s..producer.a);
        return out;
    }
    reads(n)
}

fn drop_dead_loads(ns: &mut [N], p: &Proto) {
    use crate::liveness::Regs;
    let numeric_load = |n: &N| n.op == Op::LOADN || (n.op == Op::LOADK && matches!(p.constants.get(n.d as u16 as usize), Some(Constant::Number(_))));
    let mut leader = vec![false; ns.len() + 1];
    for n in ns.iter().filter(|n| !n.gone) {
        if let Some(t) = n.target {
            leader[live_from(ns, t)] = true;
        }
    }
    // whether register r holds a number when instruction k reads it: the last write to it in
    // the same straight run is a numeric load or arithmetic that itself read numbers
    fn numeric_at(ns: &[N], leader: &[bool], k: usize, r: u8, numeric_load: &dyn Fn(&N) -> bool, depth: u32) -> bool {
        let mut j = k;
        while j > 0 && !leader[j] {
            j -= 1;
            let m = &ns[j];
            if m.gone {
                continue;
            }
            if m.a == r && !matches!(m.op, Op::JUMP | Op::JUMPX | Op::NOP) {
                return numeric_load(m)
                    || (depth < 8 && matches!(m.op, Op::MUL | Op::ADD | Op::SUB) && numeric_at(ns, leader, j, m.b, numeric_load, depth + 1) && numeric_at(ns, leader, j, m.c, numeric_load, depth + 1));
            }
            if ends(m) {
                return false;
            }
        }
        false
    }
    loop {
        // what's read later, along every path from each instruction
        let n = ns.len();
        let succ = |k: usize| {
            let mut v = Vec::new();
            if let Some(t) = ns[k].target {
                v.push(live_from(ns, t));
            }
            if falls(&ns[k]) {
                v.push(live_from(ns, k + 1));
            }
            v
        };
        let reads_of = |k: usize| {
            let mut r = Regs::default();
            for x in reads_at(ns, k) {
                r.add(x as usize);
            }
            r
        };
        let writes_of = |k: usize| {
            let m = &ns[k];
            crate::liveness::writes(&crate::bytecode::Insn::from_parts(m.op, m.a, m.b, m.c, m.aux, word_of(m)))
        };
        let mut live_in = vec![Regs::default(); n + 1];
        loop {
            let mut changed = false;
            for k in (0..n).rev() {
                if ns[k].gone {
                    continue;
                }
                let out = succ(k).into_iter().filter(|&s| s < n).fold(Regs::default(), |a, s| a.union(live_in[s]));
                let inn = reads_of(k).union(out.without(writes_of(k)));
                if inn != live_in[k] {
                    live_in[k] = inn;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut drop = Vec::new();
        for k in 0..n {
            let m = ns[k];
            if m.gone {
                continue;
            }
            let pure = match m.op {
                Op::LOADN | Op::LOADNIL | Op::LOADK => true,
                Op::LOADB => m.c == 0,
                // (arithmetic can call a metamethod or fail: only on numbers)
                Op::MUL | Op::ADD | Op::SUB => numeric_at(ns, &leader, k, m.b, &numeric_load, 0) && numeric_at(ns, &leader, k, m.c, &numeric_load, 0),
                _ => false,
            };
            let out = succ(k).into_iter().filter(|&s| s < n).fold(Regs::default(), |a, s| a.union(live_in[s]));
            if pure && !out.has(m.a) {
                drop.push(k);
            }
        }
        if drop.is_empty() {
            break;
        }
        for k in drop {
            ns[k].gone = true;
        }
    }
}

// Instructions swapped for others that do the same, put back the way the compiler writes them
fn unsubstitute(ns: &mut [N], p: &Proto) {
    // a constant loaded into a scratch register just for arithmetic with it: the K form
    for k in 0..ns.len() {
        if ns[k].gone || ns[k].op != Op::LOADK {
            continue;
        }
        let s = ns[k].a;
        let kidx = ns[k].d as u16 as usize;
        if !matches!(p.constants.get(kidx), Some(Constant::Number(_))) || kidx > 255 {
            continue;
        }
        let Some(j) = (k + 1..ns.len()).find(|&j| !ns[j].gone) else { continue };
        let u = ns[j];
        let (op, b, c) = match u.op {
            Op::ADD | Op::SUB | Op::MUL | Op::DIV | Op::MOD if u.c == s && u.b != s => (Op::from_byte(u.op as u8 + 6).unwrap(), u.b, kidx as u8),
            Op::IDIV if u.c == s && u.b != s => (Op::IDIVK, u.b, kidx as u8),
            Op::SUB if u.b == s && u.c != s => (Op::SUBRK, kidx as u8, u.c),
            Op::DIV if u.b == s && u.c != s => (Op::DIVRK, kidx as u8, u.c),
            _ => continue,
        };
        // (the scratch register is read nowhere else, and nothing jumps between the two)
        if (u.a != s && read_anywhere(ns, s, &[j])) || ns.iter().any(|n| !n.gone && n.target == Some(j)) {
            continue;
        }
        ns[j].op = op;
        ns[j].b = b;
        ns[j].c = c;
        ns[k].gone = true;
    }
    for k in 0..ns.len() {
        if ns[k].gone {
            continue;
        }
        let n = ns[k];
        match n.op {
            // AND or OR of a register with itself is a copy
            Op::AND | Op::OR if n.b == n.c => {
                ns[k].op = Op::MOVE;
                ns[k].c = 0;
            }
            Op::LOADK => match p.constants.get(n.d as u16 as usize) {
                Some(Constant::Nil) => {
                    ns[k].op = Op::LOADNIL;
                    set_d(&mut ns[k], 0);
                }
                Some(Constant::Boolean(b)) => {
                    ns[k].op = Op::LOADB;
                    ns[k].b = *b as u8;
                    ns[k].c = 0;
                    ns[k].target = None;
                }
                Some(Constant::Number(x)) if x.fract() == 0.0 && (-32768.0..=32767.0).contains(x) && !(*x == 0.0 && x.is_sign_negative()) => {
                    ns[k].op = Op::LOADN;
                    set_d(&mut ns[k], *x as i32);
                }
                _ => {}
            },
            _ => {}
        }
    }

}

// Renumbers registers past the parameters so they sit where the compiler puts them: in the
// order the code first writes them, from right after the parameters. What has to stay together
// does: a run of registers an instruction reaches moves as one, nothing moves across the first
// register of a call or CLOSEUPVALS, and from where a list runs to the top, nothing moves.
fn renumber(ns: &mut [N], p: &mut Proto, sort: bool) {
    let np = p.num_params as usize;
    let live = ns.iter().filter(|n| !n.gone).copied().collect::<Vec<_>>();
    let mut used = [false; 256];
    let mut cuts = vec![np, 256];
    let mut runs = Vec::new();
    let mut fixed_from = 256;
    for n in &live {
        for r in regs_of(n) {
            used[r as usize] = true;
        }
        if matches!(n.op, Op::CLOSEUPVALS | Op::CALL | Op::FORGLOOP | Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT) {
            cuts.push(n.a as usize);
        }
        if n.op == Op::CONCAT {
            cuts.push(n.b as usize);
        }
        if let (_, _, Some((s, count))) = reg_fields(n) {
            if count < 0 {
                fixed_from = fixed_from.min(s as usize);
            } else if count > 1 {
                runs.push((s as usize, (s as usize + count as usize).min(256)));
            }
        }
    }
    // (a run among the parameters can't move, and neither can one across a cut)
    if !sort || runs.iter().any(|&(s, e)| s < np && e > np) || runs.iter().any(|&(s, e)| cuts.iter().any(|&c| c > s && c < e)) {
        // only close the gap past the parameters: shift everything down as one
        let first = (np..256).find(|&r| used[r]).unwrap_or(np);
        if first > np && first <= fixed_from {
            let mut map = [0u8; 256];
            for (r, m) in map.iter_mut().enumerate() {
                *m = if r < np { r as u8 } else { r.saturating_sub(first - np) as u8 };
            }
            apply(ns, p, &map);
        }
        return;
    }
    cuts.push(fixed_from);
    cuts.retain(|&c| c >= np && c <= 256);
    cuts.sort_unstable();
    cuts.dedup();
    let mut first_write = [usize::MAX; 256];
    for (k, n) in live.iter().enumerate() {
        for r in regs_of(n) {
            first_write[r as usize] = first_write[r as usize].min(k);
        }
    }
    let mut map = [0u8; 256];
    for (r, m) in map.iter_mut().enumerate() {
        *m = r as u8;
    }
    let mut next = np;
    for w in cuts.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        if lo >= fixed_from {
            // from where a list runs to the top, everything moves as one: open results fill
            // registers no instruction names
            let base = next;
            for r in lo..256 {
                map[r] = (base + (r - lo)).min(255) as u8;
            }
            break;
        }
        let mut spans = runs.iter().copied().filter(|&(s, _)| s >= lo && s < hi).collect::<Vec<_>>();
        spans.sort_unstable();
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (s, e) in spans {
            match merged.last_mut() {
                Some(last) if s < last.1 => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }
        let mut units: Vec<Vec<usize>> = Vec::new();
        let mut r = lo;
        let mut m = merged.iter().peekable();
        while r < hi {
            match m.peek() {
                Some(&&(s, e)) if s == r => {
                    units.push((s..e.min(hi)).collect());
                    r = e.min(hi);
                    m.next();
                }
                _ => {
                    if used[r] {
                        units.push(vec![r]);
                    }
                    r += 1;
                }
            }
        }
        units.sort_by_key(|u| u.iter().map(|&r| first_write[r]).min().unwrap_or(usize::MAX));
        for u in &units {
            for &r in u {
                map[r] = next.min(255) as u8;
                next += 1;
            }
        }
    }
    apply(ns, p, &map);
}

fn apply(ns: &mut [N], p: &mut Proto, map: &[u8; 256]) {
    for n in ns.iter_mut() {
        remap(n, map);
    }
    for l in &mut p.locals {
        l.register = map[l.register as usize];
    }
    let top = ns.iter().filter(|n| !n.gone).flat_map(regs_of).max().map_or(0, |r| r as usize + 1);
    p.max_stack = p.max_stack.min(255).max(top as u8);
}

// Lays the surviving instructions back out as code, with jumps' offsets worked out again.
// Returns None when a jump no longer fits its field.
fn encode(ns: &[N], p: &mut Proto) -> Option<()> {
    let order = (0..ns.len()).filter(|&k| !ns[k].gone).collect::<Vec<_>>();
    let mut pc_of = vec![0usize; ns.len() + 1];
    let mut pc = 0;
    for &k in &order {
        pc_of[k] = pc;
        pc += if ns[k].op.has_aux() { 2 } else { 1 };
    }
    let end = pc;
    // a jump to something gone lands where control goes on from it
    let land = |t: usize| {
        let t = live_from(ns, t);
        if t >= ns.len() { end } else { pc_of[t] }
    };
    let mut code = Vec::with_capacity(end);
    let mut lines = Vec::with_capacity(end);
    for &k in &order {
        let mut n = ns[k];
        let at = pc_of[k];
        let word = if let Some(t) = n.target {
            let off = land(t) as i64 - (at as i64 + 1);
            if jumps_in_c(n.op) {
                if !(0..=255).contains(&off) {
                    return None;
                }
                n.c = off as u8;
                word_of(&n)
            } else if matches!(n.op, Op::JUMP | Op::JUMPBACK | Op::JUMPX) {
                if (-32768..=32767).contains(&off) {
                    n.op = if off < 0 { Op::JUMPBACK } else { Op::JUMP };
                    n.op as u32 | (off as i16 as u16 as u32) << 16
                } else {
                    Op::JUMPX as u32 | ((off as i32 as u32) & 0xff_ffff) << 8
                }
            } else {
                if !(-32768..=32767).contains(&off) {
                    return None;
                }
                n.op as u32 | (n.a as u32) << 8 | (off as i16 as u16 as u32) << 16
            }
        } else {
            word_of(&n)
        };
        code.push(word);
        lines.push(n.line);
        if n.op.has_aux() {
            code.push(n.aux);
            lines.push(n.line);
        }
    }
    // debug ranges follow their instructions; one at something gone moves to what comes next
    let mut new_pc = vec![end; p.code.len() + 1];
    let mut next = end;
    for k in (0..ns.len()).rev() {
        if !ns[k].gone {
            next = pc_of[k];
        }
        new_pc[ns[k].old_pc] = next;
    }
    for l in &mut p.locals {
        l.start_pc = new_pc.get(l.start_pc).copied().unwrap_or(end);
        l.end_pc = new_pc.get(l.end_pc).copied().unwrap_or(end);
    }
    p.locals.retain(|l| l.start_pc < l.end_pc);
    if p.lines.is_some() {
        p.lines = Some(lines);
    }
    p.code = code;
    Some(())
}

// the same test with the opposite outcome, for the conditional jumps that have one
fn invert(n: &mut N) -> bool {
    use Op::*;
    n.op = match n.op {
        JUMPIF => JUMPIFNOT,
        JUMPIFNOT => JUMPIF,
        JUMPIFEQ => JUMPIFNOTEQ,
        JUMPIFNOTEQ => JUMPIFEQ,
        JUMPIFLE => JUMPIFNOTLE,
        JUMPIFNOTLE => JUMPIFLE,
        JUMPIFLT => JUMPIFNOTLT,
        JUMPIFNOTLT => JUMPIFLT,
        JUMPXEQKNIL | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS => {
            n.aux ^= 1 << 31;
            n.op
        }
        _ => return false,
    };
    true
}

// Lays the blocks out the way the compiler emits them, when they've been shuffled and glued
// with jumps: a depth-first order over the control flow where a block comes after every block
// that leads into it (but through a loop's back edge), successors from later source lines
// placed after earlier ones, and a loop's own instructions kept with the blocks they fall into.
// Conditions flip where that lets one side fall through; anything else that can't is a jump.
fn relayout(ns: &[N]) -> Option<Vec<N>> {
    let live = (0..ns.len()).filter(|&k| !ns[k].gone).collect::<Vec<_>>();
    if live.is_empty() {
        return bail(1);
    }
    let mut pos = vec![usize::MAX; ns.len() + 1];
    for (i, &k) in live.iter().enumerate() {
        pos[k] = i;
    }
    let at = |t: usize| pos[live_from(ns, t).min(ns.len())];
    let n_live = live.len();
    // leaders: where a jump lands (fast calls skip inside a block), and after anything that
    // jumps or ends
    let mut leader = vec![false; n_live + 1];
    leader[0] = true;
    for (i, &k) in live.iter().enumerate() {
        let n = &ns[k];
        if let Some(t) = n.target
            && !matches!(n.op, Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL)
        {
            let ti = at(t);
            if ti == usize::MAX {
                return bail(2);
            }
            leader[ti] = true;
            leader[i + 1] = true;
        }
        if ends(n) {
            leader[i + 1] = true;
        }
    }
    let starts = (0..n_live).filter(|&i| leader[i]).collect::<Vec<_>>();
    let mut block_of = vec![0usize; n_live + 1];
    for (b, &s) in starts.iter().enumerate() {
        let e = starts.get(b + 1).copied().unwrap_or(n_live);
        for i in s..e {
            block_of[i] = b;
        }
    }
    block_of[n_live] = usize::MAX;
    let nb = starts.len();
    let end_of = |b: usize| starts.get(b + 1).copied().unwrap_or(n_live);
    let last = |b: usize| ns[live[end_of(b) - 1]];
    // a block that's only a jump goes where it jumps (followed to the end of such chains)
    let resolve = |mut b: usize| {
        for _ in 0..nb {
            let l = last(b);
            if end_of(b) - starts[b] == 1 && matches!(l.op, Op::JUMP | Op::JUMPX) {
                let t = block_of[at(l.target.unwrap())];
                if t == usize::MAX || t == b {
                    return b;
                }
                b = t;
            } else {
                return b;
            }
        }
        b
    };
    // each block's successors: (where it falls, where it jumps)
    let mut fall = vec![None; nb];
    let mut jump = vec![None; nb];
    for b in 0..nb {
        let l = last(b);
        // (a loop's body stays its own block even when it's only a jump: `for ... do break end`)
        let body_fall = l.op == Op::FORNPREP;
        let body_jump = matches!(l.op, Op::FORNLOOP | Op::FORGLOOP);
        if !ends(&l) {
            let f = block_of[end_of(b)];
            // (a FORNLOOP nothing reaches may be last; where it falls is worked out below)
            if f == usize::MAX && l.op != Op::FORNLOOP {
                return bail(3);
            }
            if f != usize::MAX {
                // (an empty body is only a jump to the FORNLOOP: that one does go)
                let r = resolve(f);
                fall[b] = Some(if body_fall && last(r).op != Op::FORNLOOP { f } else { r });
            }
        }
        if let Some(t) = l.target
            && !matches!(l.op, Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL)
        {
            let j = block_of[at(t)];
            jump[b] = Some(if body_jump { j } else { resolve(j) });
        }
    }
    // a FORNLOOP nothing reaches falls where its own FORNPREP jumps (the loop's exit), not
    // into whatever the shuffle put after it: it never runs, so that only says where it goes
    for l in 0..nb {
        if let Some(prep) = last(l).paired {
            let pb = block_of[pos[prep]];
            fall[l] = if pb == usize::MAX { None } else { jump[pb] };
        }
    }
    // blocks a loop instruction has to sit right before: FORNPREP's body, FORNLOOP's and
    // FORGLOOP's exit, the body FORGPREP's loop runs (where FORGLOOP jumps back to)
    let mut glued = vec![None; nb];
    for b in 0..nb {
        let l = last(b);
        match l.op {
            Op::FORNPREP | Op::FORNLOOP | Op::FORGLOOP => glued[b] = fall[b],
            Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT => {
                let g = jump[b].or_else(|| bail(6))?;
                glued[b] = jump[g].filter(|_| matches!(last(g).op, Op::FORGLOOP));
            }
            _ => {}
        }
    }
    // a loop's back edge comes from the block that ends it (the one jumping to a block that
    // dominates it): that block goes after everything else in the loop, whatever its line
    // (for order, FORGPREP leads into the body it jumps around: the compiler writes the body
    // between it and FORGLOOP, which it enters at the bottom)
    let forgprep = |b: usize| matches!(last(b).op, Op::FORGPREP | Op::FORGPREP_INEXT | Op::FORGPREP_NEXT);
    let edges = |b: usize| if forgprep(b) { glued[b].into_iter().collect() } else { fall[b].into_iter().chain(jump[b]).collect::<Vec<_>>() };
    let idom = dominators(nb, &edges);
    let dominates = |a: usize, mut b: usize| loop {
        if a == b {
            return true;
        }
        match idom[b] {
            Some(d) if d != b => b = d,
            _ => return false,
        }
    };
    let latch = (0..nb).map(|b| edges(b).iter().any(|&s| dominates(s, b))).collect::<Vec<_>>();
    let line = |b: usize| ns[live[starts[b]]].line;
    // each loop's blocks (its header, and what reaches a back edge into it without passing the
    // header): a successor outside a loop its block is in is that loop's exit
    let mut preds = vec![Vec::new(); nb];
    for b in 0..nb {
        for s in edges(b) {
            preds[s].push(b);
        }
    }
    let mut loops: Vec<Vec<bool>> = Vec::new();
    for b in 0..nb {
        for h in edges(b) {
            if !dominates(h, b) {
                continue;
            }
            let mut inside = vec![false; nb];
            inside[h] = true;
            let mut work = vec![b];
            while let Some(x) = work.pop() {
                if !inside[x] {
                    inside[x] = true;
                    work.extend(preds[x].iter().copied());
                }
            }
            // What follows the loop is where its header or the block closing it goes out to.
            // Other ways out (a break, a return in the loop) are the body's own: the compiler
            // writes them inside it, so they count as in it
            let exits_of = |inside: &Vec<bool>, x: usize| edges(x).into_iter().filter(|&s| !inside[s]).collect::<Vec<_>>();
            let mut follow = exits_of(&inside, h);
            follow.extend(exits_of(&inside, b));
            // (and where the blocks leading into the closing one go out: a repeat's until test
            // before a JUMPBACK of its own)
            for &x in &preds[b] {
                if inside[x] && x != h {
                    follow.extend(exits_of(&inside, x));
                }
            }
            if follow.is_empty() {
                // (`while true` left only by breaks: what the breaks go to)
                let first = (0..nb).filter(|&x| inside[x]).flat_map(|x| exits_of(&inside, x)).collect::<Vec<_>>();
                follow = first.iter().flat_map(|&e| edges(e)).filter(|s| first.contains(s) || !inside[*s]).collect();
                if follow.is_empty() {
                    follow = first;
                }
            }
            let mut work = (0..nb).filter(|&x| inside[x]).flat_map(|x| exits_of(&inside, x)).collect::<Vec<_>>();
            while let Some(x) = work.pop() {
                if inside[x] || follow.contains(&x) || !dominates(h, x) {
                    continue;
                }
                inside[x] = true;
                work.extend(edges(x));
            }
            loops.push(inside);
        }
    }
    let leaves = |b: usize, s: usize| loops.iter().any(|l| l[b] && !l[s]);
    // depth first from the entry: successors from later lines first, so they're placed later;
    // on a tie the one it falls into last, so it can still fall into it
    let mut post = Vec::with_capacity(nb);
    let mut seen = vec![false; nb];
    let mut stack = vec![(0usize, 0usize)];
    seen[0] = true;
    let succs = |b: usize| {
        if forgprep(b) {
            return glued[b].into_iter().collect::<Vec<_>>();
        }
        // (rank: a loop's exit first, so it's placed after the loop; then the block that closes
        // the loop, so it's placed after the rest of it; then later lines first; the glued one
        // last, so it ends up right after)
        let rank = |s: usize, kind: u8| -> (u8, std::cmp::Reverse<u32>, u8) {
            if glued[b] == Some(s) {
                (3, std::cmp::Reverse(0), 0)
            } else if leaves(b, s) {
                (0, std::cmp::Reverse(line(s)), kind)
            } else if latch[s] {
                (1, std::cmp::Reverse(line(s)), kind)
            } else {
                (2, std::cmp::Reverse(line(s)), kind)
            }
        };
        let mut v = Vec::new();
        if let Some(j) = jump[b] {
            v.push((rank(j, 0), j));
        }
        if let Some(f) = fall[b]
            && Some(f) != jump[b]
        {
            v.push((rank(f, 1), f));
        }
        v.sort_by_key(|x| x.0);
        v.into_iter().map(|x| x.1).collect::<Vec<_>>()
    };
    while let Some(&mut (b, ref mut i)) = stack.last_mut() {
        let next = succs(b);
        if *i < next.len() {
            let s = next[*i];
            *i += 1;
            if !seen[s] {
                seen[s] = true;
                stack.push((s, 0));
            }
        } else {
            post.push(b);
            stack.pop();
        }
    }
    let mut order = post.into_iter().rev().collect::<Vec<_>>();
    // a FORNLOOP nothing reaches goes right before the exit it falls into
    for b in 0..nb {
        if !seen[b]
            && last(b).op == Op::FORNLOOP
            && let Some(exit) = fall[b]
            && let Some(at) = order.iter().position(|&x| x == exit)
        {
            order.insert(at, b);
            seen[b] = true;
        }
    }
    if std::env::var_os("SUNBERRY_TRACE_LAYOUT").is_some() {
        for b in 0..nb {
            eprintln!("  block {b} at {} line {} last {:?} fall {:?} jump {:?} glued {:?} succs {:?}", ns[live[starts[b]]].old_pc, line(b), last(b).op, fall[b], jump[b], glued[b], succs(b));
        }
        eprintln!("  order {order:?}");
    }
    // a glued block goes right after its loop instruction even where depth first put it elsewhere
    for b in 0..nb {
        if let Some(g) = glued[b]
            && seen[b]
        {
            let gi = order.iter().position(|&x| x == g).or_else(|| bail(7))?;
            let bi = order.iter().position(|&x| x == b).or_else(|| bail(8))?;
            if gi != bi + 1 {
                order.remove(gi);
                let bi = order.iter().position(|&x| x == b).or_else(|| bail(9))?;
                order.insert(bi + 1, g);
            }
        }
    }
    // lay it out: each block's instructions, then a jump where what it falls into isn't next
    let mut out: Vec<N> = Vec::with_capacity(n_live + nb);
    let mut start_of = vec![usize::MAX; nb];
    // (targets are block numbers until every block has a place)
    let mut fix: Vec<(usize, usize)> = Vec::new();
    for (oi, &b) in order.iter().enumerate() {
        start_of[b] = out.len();
        let next = order.get(oi + 1).copied();
        for i in starts[b]..end_of(b) {
            let mut n = ns[live[i]];
            n.target = None;
            let k = live[i];
            let is_last = i + 1 == end_of(b);
            if let Some(t) = ns[k].target {
                if matches!(n.op, Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL) {
                    // (a skip within the block, or to the block that has to follow)
                    let ti = at(t);
                    if block_of[ti] != b && glued_after(&order, oi, block_of[ti]) {
                        fix.push((out.len(), block_of[ti]));
                    } else if block_of[ti] == b {
                        n.target = Some(usize::MAX - (ti - starts[b]));
                    } else {
                        return bail(4);
                    }
                } else if is_last {
                    fix.push((out.len(), jump[b].unwrap()));
                }
            }
            if is_last && n.target.is_none() && !fix.last().is_some_and(|f| f.0 == out.len()) && ns[k].target.is_some() {
                // (handled above)
            }
            out.push(n);
        }
        let l = last(b);
        let (f, j) = (fall[b], jump[b]);
        if let Some(f) = f {
            let cond = l.target.is_some() && !matches!(l.op, Op::FASTCALL | Op::FASTCALL1 | Op::FASTCALL2 | Op::FASTCALL2K | Op::FASTCALL3 | Op::FASTPCALL);
            if next == Some(f) {
            } else if cond && next == j && invert(out.last_mut().unwrap()) {
                // flip it: it now jumps where it fell, and falls where it jumped
                let lastk = out.len() - 1;
                for x in fix.iter_mut().rev() {
                    if x.0 == lastk {
                        x.1 = f;
                        break;
                    }
                }
            } else {
                let mut glue = l;
                glue.op = Op::JUMP;
                glue.a = 0;
                glue.aux = 0;
                glue.gone = false;
                fix.push((out.len(), f));
                out.push(glue);
            }
        }
    }
    for (k, b) in fix {
        let s = start_of[b];
        if s == usize::MAX {
            return bail(5);
        }
        out[k].target = Some(s);
    }
    // in-block fast call skips, by offset from the block's start
    for k in 0..out.len() {
        if let Some(t) = out[k].target
            && t > out.len()
        {
            let off = usize::MAX - t;
            let bstart = (0..=k).rev().find(|&s| start_of.contains(&s)).unwrap();
            out[k].target = Some(bstart + off);
        }
    }
    Some(out)
}

// each block's immediate dominator (None where unreachable; the entry is its own), Cooper,
// Harvey and Kennedy's iteration over a reverse postorder
fn dominators(nb: usize, edges: &dyn Fn(usize) -> Vec<usize>) -> Vec<Option<usize>> {
    let mut post = Vec::with_capacity(nb);
    let mut seen = vec![false; nb];
    let mut stack = vec![(0usize, 0usize)];
    seen[0] = true;
    while let Some(&mut (b, ref mut i)) = stack.last_mut() {
        let next = edges(b);
        if *i < next.len() {
            let s = next[*i];
            *i += 1;
            if !seen[s] {
                seen[s] = true;
                stack.push((s, 0));
            }
        } else {
            post.push(b);
            stack.pop();
        }
    }
    let mut rank = vec![usize::MAX; nb];
    for (i, &b) in post.iter().enumerate() {
        rank[b] = i;
    }
    let mut preds = vec![Vec::new(); nb];
    for b in 0..nb {
        if seen[b] {
            for s in edges(b) {
                preds[s].push(b);
            }
        }
    }
    let mut idom: Vec<Option<usize>> = vec![None; nb];
    idom[0] = Some(0);
    let rpo = post.iter().rev().copied().collect::<Vec<_>>();
    let mut changed = true;
    while changed {
        changed = false;
        for &b in rpo.iter().skip(1) {
            let mut new: Option<usize> = None;
            for &p in &preds[b] {
                if idom[p].is_none() {
                    continue;
                }
                new = Some(match new {
                    None => p,
                    Some(mut x) => {
                        let mut y = p;
                        while x != y {
                            while rank[x] < rank[y] {
                                x = idom[x].unwrap();
                            }
                            while rank[y] < rank[x] {
                                y = idom[y].unwrap();
                            }
                        }
                        x
                    }
                });
            }
            if new.is_some() && idom[b] != new {
                idom[b] = new;
                changed = true;
            }
        }
    }
    idom
}

// for diagnostics: where relayout gave up
fn bail<T>(at: u32) -> Option<T> {
    if std::env::var_os("SUNBERRY_TRACE_LAYOUT").is_some() {
        eprintln!("relayout gave up at {at}");
    }
    None
}

// whether block t comes right after position oi in order
fn glued_after(order: &[usize], oi: usize, t: usize) -> bool {
    order.get(oi + 1) == Some(&t)
}

// a jump to where control goes anyway
fn drop_jumps_to_next(ns: &mut [N]) {
    for k in (0..ns.len()).rev() {
        if !ns[k].gone
            && matches!(ns[k].op, Op::JUMP | Op::JUMPX)
            && let Some(t) = ns[k].target
            && live_from(ns, t) == live_from(ns, k + 1)
        {
            ns[k].gone = true;
        }
    }
}

// The compiler reads a global, and a field of one by name, as an import (GETIMPORT); spelled
// out as GETGLOBAL and GETTABLEKS into the same register, it's that import again. With
// Rosetta's settings the compiler never writes GETGLOBAL itself.
fn restore_imports(ns: &mut [N], p: &mut Proto) {
    for k in 0..ns.len() {
        if ns[k].gone || ns[k].op != Op::GETGLOBAL {
            continue;
        }
        let r = ns[k].a;
        let mut names = vec![ns[k].aux];
        let mut chain = vec![k];
        let mut j = k + 1;
        while names.len() < 3 {
            j = live_from(ns, j);
            if j >= ns.len() || ns[j].op != Op::GETTABLEKS || ns[j].a != r || ns[j].b != r || ns.iter().any(|n| !n.gone && n.target == Some(j)) {
                break;
            }
            names.push(ns[j].aux);
            chain.push(j);
            j += 1;
        }
        if names.iter().any(|&n| n >= 1024 || !matches!(p.constants.get(n as usize), Some(Constant::String(_)))) {
            continue;
        }
        let path = names.clone();
        let index = match p.constants.iter().position(|c| *c == Constant::Import(path.clone())) {
            Some(i) => i,
            None => {
                p.constants.push(Constant::Import(path));
                p.constants.len() - 1
            }
        };
        if index > 32767 {
            continue;
        }
        let id = (names.len() as u32) << 30 | names.iter().enumerate().map(|(i, &n)| n << (20 - 10 * i)).fold(0, |a, b| a | b);
        ns[k].op = Op::GETIMPORT;
        set_d(&mut ns[k], index as i32);
        ns[k].aux = id;
        for &c in &chain[1..] {
            ns[c].gone = true;
        }
    }
}

// Rewrites p's code the way the compiler would have written it; false if it couldn't, and p is
// left as it was
// `sort_registers`: also put registers in the order the code first writes them (undoing a
// shuffle), not only close the gap past the parameters
pub fn normalize(p: &mut Proto, sort_registers: bool) -> bool {
    let mut ns = decode(p);
    let before = p.clone();
    fold_branches(&mut ns, p);
    for n in ns.iter_mut() {
        if n.op == Op::NOP || n.op == Op::COVERAGE {
            n.gone = true;
        }
    }
    drop_unreachable(&mut ns);
    unsubstitute(&mut ns, p);
    drop_dead_loads(&mut ns, p);
    drop_jumps_to_next(&mut ns);
    if sort_registers {
        match relayout(&ns) {
            Some(out) => ns = out,
            None => {
                *p = before;
                return false;
            }
        }
        unsubstitute(&mut ns, p);
        drop_jumps_to_next(&mut ns);
        restore_imports(&mut ns, p);
    }
    renumber(&mut ns, p, sort_registers);
    if encode(&ns, p).is_none() {
        *p = before;
        return false;
    }
    true
}
