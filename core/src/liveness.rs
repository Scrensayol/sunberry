// Which registers an instruction reads and writes, and which registers are read again
// later along some path. Runs over the control flow graph until nothing changes, so loops
// work the same as branches.

use crate::bytecode::{Insn, Op};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Regs([u64; 4]);

impl Regs {
    pub fn add(&mut self, r: usize) {
        if r < 256 {
            self.0[r / 64] |= 1 << (r % 64);
        }
    }

    pub fn add_range(&mut self, from: usize, to: usize) {
        for r in from..to.min(256) {
            self.add(r);
        }
    }

    pub fn has(&self, r: u8) -> bool {
        self.0[r as usize / 64] & (1 << (r % 64)) != 0
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..=255u8).filter(|&r| self.has(r))
    }

    pub fn without(self, other: Regs) -> Regs {
        Regs([0, 1, 2, 3].map(|i| self.0[i] & !other.0[i]))
    }

    pub fn union(self, other: Regs) -> Regs {
        Regs([0, 1, 2, 3].map(|i| self.0[i] | other.0[i]))
    }

    pub fn intersect(self, other: Regs) -> Regs {
        Regs([0, 1, 2, 3].map(|i| self.0[i] & other.0[i]))
    }
}

pub fn reads(i: &Insn) -> Regs {
    use Op::*;
    let (a, b, c) = (i.a as usize, i.b as usize, i.c as usize);
    let mut r = Regs::default();
    match i.op {
        MOVE | GETTABLEKS | GETTABLEN | GETUDATAKS | NAMECALL | NAMECALLUDATA | NOT | MINUS | LENGTH | ADDK | SUBK
        | MULK | DIVK | MODK | POWK | IDIVK | ANDK | ORK => r.add(b),
        SUBRK | DIVRK => r.add(c),
        SETGLOBAL | SETUPVAL | JUMPIF | JUMPIFNOT | JUMPXEQKNIL | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS | CMPPROTO => r.add(a),
        GETTABLE | ADD | SUB | MUL | DIV | MOD | POW | IDIV | AND | OR => {
            r.add(b);
            r.add(c);
        }
        SETTABLE => {
            r.add(a);
            r.add(b);
            r.add(c);
        }
        SETTABLEKS | SETTABLEN | SETUDATAKS => {
            r.add(a);
            r.add(b);
        }
        JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT => {
            r.add(a);
            r.add(i.aux as usize & 0xff);
        }
        CONCAT => r.add_range(b, c + 1),
        // an open count reads up to the top, which only ever holds the values being passed
        CALL | CALLFB => r.add_range(a, if b == 0 { 256 } else { a + b }),
        RETURN => r.add_range(a, if b == 0 { 256 } else { a + b - 1 }),
        SETLIST => {
            r.add(a);
            r.add_range(b, if c == 0 { 256 } else { b + c - 1 });
        }
        FORNPREP | FORNLOOP | FORGLOOP | FORGPREP | FORGPREP_INEXT | FORGPREP_NEXT => r.add_range(a, a + 3),
        CAPTURE if a != 2 => r.add(b),
        // FASTCALL's arguments are read again by the CALL it guards, which is where they count
        _ => {}
    }
    r
}

pub fn writes(i: &Insn) -> Regs {
    use Op::*;
    let (a, b, c) = (i.a as usize, i.b as usize, i.c as usize);
    let mut r = Regs::default();
    match i.op {
        LOADNIL | LOADB | LOADN | LOADK | LOADKX | MOVE | GETGLOBAL | GETUPVAL | GETIMPORT | GETTABLE | GETTABLEKS
        | GETTABLEN | GETUDATAKS | NEWCLOSURE | DUPCLOSURE | NEWTABLE | DUPTABLE | ADD | SUB | MUL | DIV | MOD | POW
        | IDIV | ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK | SUBRK | DIVRK | AND | OR | ANDK | ORK | CONCAT
        | NOT | MINUS | LENGTH => r.add(a),
        NAMECALL | NAMECALLUDATA => r.add_range(a, a + 2),
        // an open result list replaces everything from its base up
        CALL | CALLFB => r.add_range(a, if c == 0 { 256 } else { a + c - 1 }),
        GETVARARGS => r.add_range(a, if b == 0 { 256 } else { a + b - 1 }),
        _ => {}
    }
    r
}

// where control can go after insns[index]
pub fn successors(insns: &[Insn], index_of: &[usize], index: usize) -> Vec<usize> {
    use Op::*;
    let insn = &insns[index];
    let next = (index + 1 < insns.len()).then_some(index + 1);
    let target = insn.target().and_then(|t| index_of.get(t).copied()).filter(|&t| t != usize::MAX);
    match insn.op {
        RETURN => Vec::new(),
        JUMP | JUMPBACK | JUMPX | FORGPREP | FORGPREP_INEXT | FORGPREP_NEXT => target.into_iter().collect(),
        LOADB if insn.c != 0 => target.into_iter().collect(),
        _ => next.into_iter().chain(target).collect(),
    }
}

pub struct Liveness {
    // registers read along some path from here before being written
    pub live_in: Vec<Regs>,
    // the same, from just after the instruction
    pub live_after: Vec<Regs>,
}

pub fn analyze(insns: &[Insn], index_of: &[usize]) -> Liveness {
    let successors = (0..insns.len()).map(|i| successors(insns, index_of, i)).collect::<Vec<_>>();
    let effects = insns.iter().map(|i| (reads(i), writes(i))).collect::<Vec<_>>();
    let mut live_in = vec![Regs::default(); insns.len()];
    loop {
        let mut changed = false;
        for i in (0..insns.len()).rev() {
            let out = successors[i].iter().fold(Regs::default(), |acc, &s| acc.union(live_in[s]));
            let (reads, writes) = effects[i];
            let inn = reads.union(out.without(writes));
            if inn != live_in[i] {
                live_in[i] = inn;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let live_after = successors.iter().map(|s| s.iter().fold(Regs::default(), |acc, &s| acc.union(live_in[s]))).collect();
    Liveness { live_in, live_after }
}
