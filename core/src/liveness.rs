// Which registers an instruction reads and writes, and which registers are read again
// later. Straight-line code only for now: control flow will run the same transfer over a
// graph until it settles.

use crate::bytecode::{Insn, Op};

#[derive(Clone, Copy, Default)]
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

    fn without(self, other: Regs) -> Regs {
        Regs([0, 1, 2, 3].map(|i| self.0[i] & !other.0[i]))
    }

    fn union(self, other: Regs) -> Regs {
        Regs([0, 1, 2, 3].map(|i| self.0[i] | other.0[i]))
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

// live[i]: registers read by some instruction after insns[i] before being written again
pub fn live_after(insns: &[Insn]) -> Vec<Regs> {
    let mut live = vec![Regs::default(); insns.len()];
    let mut current = Regs::default();
    for (index, insn) in insns.iter().enumerate().rev() {
        live[index] = current;
        current = reads(insn).union(current.without(writes(insn)));
    }
    live
}
