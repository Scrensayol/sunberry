use std::fmt::Write;

use crate::bytecode::{Chunk, Constant, Insn, Op, Proto};

pub fn disassemble(chunk: &Chunk) -> String {
    let mut out = String::new();
    if !chunk.userdata_types.is_empty() {
        let names = chunk.userdata_types.iter().map(|(i, n)| format!("{i}={}", String::from_utf8_lossy(n))).collect::<Vec<_>>();
        let _ = writeln!(out, "userdata types (version {}): {}\n", chunk.types_version, names.join(" "));
    }
    for (id, proto) in chunk.protos.iter().enumerate() {
        let name = proto.debug_name.as_deref().map(String::from_utf8_lossy).unwrap_or_default();
        let main = if id == chunk.main { " (main)" } else { "" };
        let _ = writeln!(
            out,
            "function {id}{main} {name} params={} upvalues={} vararg={} stack={} line={}",
            proto.num_params, proto.num_upvalues, proto.is_vararg, proto.max_stack, proto.line_defined
        );
        if !proto.type_info.is_empty() {
            let bytes = proto.type_info.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
            let _ = writeln!(out, "  types {bytes}");
        }
        for (i, k) in proto.constants.iter().enumerate() {
            let _ = writeln!(out, "  K{i} = {}", constant(proto, k));
        }
        for (i, child) in proto.children.iter().enumerate() {
            let _ = writeln!(out, "  P{i} = function {child}");
        }
        for local in &proto.locals {
            let _ = writeln!(
                out,
                "  local R{} {} pc {}..{}",
                local.register,
                String::from_utf8_lossy(&local.name),
                local.start_pc,
                local.end_pc
            );
        }
        for insn in proto.insns() {
            let line = proto.lines.as_ref().map(|l| format!("{:>5} ", l[insn.pc])).unwrap_or_default();
            let _ = writeln!(out, "  {line}{:>5}  {}", insn.pc, instruction(proto, &insn));
        }
        out.push('\n');
    }
    out
}

fn constant(proto: &Proto, k: &Constant) -> String {
    match k {
        Constant::Nil => "nil".into(),
        Constant::Boolean(b) => b.to_string(),
        Constant::Number(n) => format!("{n:?}"),
        Constant::Integer(n) => format!("{n}i"),
        Constant::Vector(v) => format!("vector({}, {}, {}, {})", v[0], v[1], v[2], v[3]),
        Constant::String(s) => format!("{:?}", String::from_utf8_lossy(s)),
        Constant::Import(path) => path
            .iter()
            .map(|&i| proto.string(i).map(|s| String::from_utf8_lossy(s).into_owned()).unwrap_or("?".into()))
            .collect::<Vec<_>>()
            .join("."),
        Constant::Table(keys) => {
            let keys = keys
                .iter()
                .map(|(key, value)| match value {
                    Some(v) => format!("K{key}=K{v}"),
                    None => format!("K{key}"),
                })
                .collect::<Vec<_>>();
            format!("table {{{}}}", keys.join(", "))
        }
        Constant::Closure(f) => format!("closure function {f}"),
        Constant::ClassShape => "class shape".into(),
    }
}

fn instruction(proto: &Proto, i: &Insn) -> String {
    use Op::*;
    let (a, b, c) = (i.a, i.b, i.c);
    let k = |index: u32| proto.constants.get(index as usize).map(|k| constant(proto, k)).unwrap_or("?".into());
    let to = || i.target().map(|t| format!(" -> {t}")).unwrap_or_default();
    let body = match i.op {
        LOADNIL | CLOSEUPVALS | PREPVARARGS => format!("R{a}"),
        LOADB => format!("R{a} {}{}", b != 0, to()),
        LOADN => format!("R{a} {}", i.d()),
        LOADK => format!("R{a} K{} {}", i.d(), k(i.d() as u32)),
        LOADKX => format!("R{a} K{} {}", i.aux, k(i.aux)),
        MOVE | NOT | MINUS | LENGTH => format!("R{a} R{b}"),
        GETGLOBAL | SETGLOBAL => format!("R{a} {}", k(i.aux)),
        GETUPVAL | SETUPVAL => format!("R{a} U{b}"),
        GETIMPORT => format!("R{a} {}", k(i.d() as u32)),
        GETTABLE | SETTABLE => format!("R{a} R{b}[R{c}]"),
        GETTABLEKS | SETTABLEKS | GETUDATAKS | SETUDATAKS => format!("R{a} R{b}.{}", k(i.aux)),
        GETTABLEN | SETTABLEN => format!("R{a} R{b}[{}]", c as u32 + 1),
        NEWCLOSURE => format!("R{a} P{}", i.d()),
        DUPCLOSURE | DUPTABLE => format!("R{a} K{} {}", i.d(), k(i.d() as u32)),
        NAMECALL | NAMECALLUDATA => format!("R{a} R{b}:{}", k(i.aux)),
        CALL | CALLFB => format!("R{a} args={} results={}", count(b), count(c)),
        RETURN => format!("R{a} values={}", count(b)),
        GETVARARGS => format!("R{a} values={}", count(b)),
        JUMP | JUMPBACK | JUMPX => to().trim_start().to_string(),
        JUMPIF | JUMPIFNOT => format!("R{a}{}", to()),
        JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT => format!("R{a} R{}{}", i.aux, to()),
        JUMPXEQKNIL => format!("R{a} not={}{}", i.aux >> 31, to()),
        JUMPXEQKB => format!("R{a} {} not={}{}", i.aux & 1 != 0, i.aux >> 31, to()),
        JUMPXEQKN | JUMPXEQKS => format!("R{a} {} not={}{}", k(i.aux & 0xffffff), i.aux >> 31, to()),
        ADD | SUB | MUL | DIV | MOD | POW | IDIV | AND | OR => format!("R{a} R{b} R{c}"),
        ADDK | SUBK | MULK | DIVK | MODK | POWK | IDIVK | ANDK | ORK => format!("R{a} R{b} {}", k(c as u32)),
        SUBRK | DIVRK => format!("R{a} {} R{c}", k(b as u32)),
        CONCAT => format!("R{a} R{b}..R{c}"),
        NEWTABLE => format!("R{a} hash={b} array={}", i.aux),
        SETLIST => format!("R{a} R{b} count={} at={}", count(c), i.aux),
        FORNPREP | FORNLOOP | FORGPREP | FORGPREP_INEXT | FORGPREP_NEXT => format!("R{a}{}", to()),
        FORGLOOP => format!("R{a} vars={}{}", i.aux & 0xff, to()),
        CAPTURE => format!("{} {b}", ["val R", "ref R", "upval U"].get(a as usize).copied().unwrap_or("?")),
        FASTCALL => format!("builtin {a} skip={c}"),
        FASTCALL1 => format!("builtin {a} R{b} skip={c}"),
        FASTCALL2 => format!("builtin {a} R{b} R{} skip={c}", i.aux & 0xff),
        FASTCALL2K => format!("builtin {a} R{b} {} skip={c}", k(i.aux)),
        FASTCALL3 => format!("builtin {a} R{b} R{} R{} skip={c}", i.aux & 0xff, (i.aux >> 8) & 0xff),
        FASTPCALL => format!("{} args={b} skip={c}", if a == 0 { "pcall" } else { "xpcall" }),
        CMPPROTO => format!("R{a} proto={}{}", i.aux, to()),
        _ => format!("{a} {b} {c} aux={}", i.aux),
    };
    format!("{:<12} {body}", i.op.name())
}

fn count(field: u8) -> String {
    if field == 0 { "multret".into() } else { (field - 1).to_string() }
}
