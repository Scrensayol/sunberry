// Luau bytecode reader, following VM/src/lvmload.cpp (versions 3..=14). Instructions keep
// their word positions: jump offsets count AUX words, so everything downstream addresses
// code by word index.

use std::fmt;

macro_rules! opcodes {
    ($($name:ident),* $(,)?) => {
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum Op { $($name),* }

        const OPS: &[Op] = &[$(Op::$name),*];

        impl Op {
            pub fn name(self) -> &'static str {
                match self { $(Op::$name => stringify!($name)),* }
            }
        }
    };
}

// order is LuauOpcode's in Common/include/Luau/Bytecode.h
opcodes!(
    NOP, BREAK, LOADNIL, LOADB, LOADN, LOADK, MOVE, GETGLOBAL, SETGLOBAL, GETUPVAL, SETUPVAL,
    CLOSEUPVALS, GETIMPORT, GETTABLE, SETTABLE, GETTABLEKS, SETTABLEKS, GETTABLEN, SETTABLEN,
    NEWCLOSURE, NAMECALL, CALL, RETURN, JUMP, JUMPBACK, JUMPIF, JUMPIFNOT, JUMPIFEQ, JUMPIFLE,
    JUMPIFLT, JUMPIFNOTEQ, JUMPIFNOTLE, JUMPIFNOTLT, ADD, SUB, MUL, DIV, MOD, POW, ADDK, SUBK,
    MULK, DIVK, MODK, POWK, AND, OR, ANDK, ORK, CONCAT, NOT, MINUS, LENGTH, NEWTABLE, DUPTABLE,
    SETLIST, FORNPREP, FORNLOOP, FORGLOOP, FORGPREP_INEXT, FASTCALL3, FORGPREP_NEXT, NATIVECALL,
    GETVARARGS, DUPCLOSURE, PREPVARARGS, LOADKX, JUMPX, FASTCALL, COVERAGE, CAPTURE, SUBRK,
    DIVRK, FASTCALL1, FASTCALL2, FASTCALL2K, FORGPREP, JUMPXEQKNIL, JUMPXEQKB, JUMPXEQKN,
    JUMPXEQKS, IDIV, IDIVK, GETUDATAKS, SETUDATAKS, NAMECALLUDATA, NEWCLASSMEMBER, CALLFB,
    CMPPROTO, FASTPCALL, NEWCLASS,
);

impl Op {
    pub fn from_byte(byte: u8) -> Option<Op> {
        OPS.get(byte as usize).copied()
    }

    // getOpLength in Common/include/Luau/BytecodeUtils.h
    pub fn has_aux(self) -> bool {
        use Op::*;
        matches!(
            self,
            GETGLOBAL | SETGLOBAL | GETIMPORT | GETTABLEKS | SETTABLEKS | NAMECALL | JUMPIFEQ | JUMPIFLE
                | JUMPIFLT | JUMPIFNOTEQ | JUMPIFNOTLE | JUMPIFNOTLT | NEWTABLE | SETLIST | FORGLOOP | LOADKX
                | FASTCALL2 | FASTCALL2K | FASTCALL3 | JUMPXEQKNIL | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS
                | GETUDATAKS | SETUDATAKS | NAMECALLUDATA | NEWCLASSMEMBER | CALLFB | CMPPROTO | NEWCLASS
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Insn {
    pub pc: usize,
    pub op: Op,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub aux: u32,
    word: u32,
}

impl Insn {
    // (for rewriting code: an instruction from its fields, pc 0)
    pub fn from_parts(op: Op, a: u8, b: u8, c: u8, aux: u32, word: u32) -> Insn {
        Insn { pc: 0, op, a, b, c, aux, word }
    }

    pub fn d(&self) -> i32 {
        (self.word as i32) >> 16
    }

    pub fn e(&self) -> i32 {
        (self.word as i32) >> 8
    }

    pub fn len(&self) -> usize {
        if self.op.has_aux() { 2 } else { 1 }
    }

    // where a jump lands, for every instruction that has one
    pub fn target(&self) -> Option<usize> {
        use Op::*;
        let offset = match self.op {
            JUMP | JUMPBACK | JUMPIF | JUMPIFNOT | JUMPIFEQ | JUMPIFLE | JUMPIFLT | JUMPIFNOTEQ
            | JUMPIFNOTLE | JUMPIFNOTLT | FORNPREP | FORNLOOP | FORGLOOP | FORGPREP | FORGPREP_INEXT
            | FORGPREP_NEXT | JUMPXEQKNIL | JUMPXEQKB | JUMPXEQKN | JUMPXEQKS | CMPPROTO => self.d(),
            JUMPX => self.e(),
            LOADB if self.c != 0 => self.c as i32,
            _ => return None,
        };
        Some((self.pc as i64 + 1 + offset as i64) as usize)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    Vector([f64; 4]),
    String(Vec<u8>),
    // up to three constant indices naming a global path
    Import(Vec<u32>),
    // DUPTABLE templates: key constant, and the constant it starts with (if any)
    Table(Vec<(u32, Option<u32>)>),
    Closure(u32),
    ClassShape,
}

#[derive(Clone, Debug)]
pub struct LocalVar {
    pub name: Vec<u8>,
    pub start_pc: usize,
    pub end_pc: usize,
    pub register: u8,
}

#[derive(Clone, Debug)]
pub struct Proto {
    pub max_stack: u8,
    pub num_params: u8,
    pub num_upvalues: u8,
    pub is_vararg: bool,
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub children: Vec<usize>,
    pub line_defined: u32,
    pub debug_name: Option<Vec<u8>>,
    pub lines: Option<Vec<u32>>,
    pub locals: Vec<LocalVar>,
    pub upvalue_names: Vec<Vec<u8>>,
    // the compiler's type info for the function, as stored (see `param_types`)
    pub type_info: Vec<u8>,
}

impl Proto {
    // Parameter types from the compiler's type info, as annotation text: only those an
    // annotation can say exactly (`number`, `string?`, a userdata type the host named), not
    // `any` or the ones that need a shape (tables, functions)
    pub fn param_types(&self, types_version: u8, userdata: &[(u8, Vec<u8>)]) -> Vec<Option<String>> {
        let info = &self.type_info;
        let signature = if types_version == 1 {
            Some(&info[..])
        } else {
            let varint = |at: &mut usize| {
                let mut value = 0usize;
                for shift in (0..35).step_by(7) {
                    let byte = *info.get(*at)?;
                    *at += 1;
                    value |= ((byte & 0x7f) as usize) << shift;
                    if byte & 0x80 == 0 {
                        return Some(value);
                    }
                }
                None
            };
            let mut at = 0;
            (|| {
                let size = varint(&mut at)?;
                varint(&mut at)?;
                varint(&mut at)?;
                info.get(at..at + size)
            })()
        };
        let Some(&[5, count, ref types @ ..]) = signature else { return vec![None; self.num_params as usize] };
        (0..self.num_params as usize)
            .map(|i| {
                let t = *types.get(i).filter(|_| i < count as usize)?;
                let base = match t & 0x7f {
                    1 => "boolean".to_string(),
                    2 => "number".to_string(),
                    3 => "string".to_string(),
                    6 => "thread".to_string(),
                    9 => "buffer".to_string(),
                    tag @ 64..=95 => {
                        let name = &userdata.iter().find(|(index, _)| *index == tag - 63)?.1;
                        let name = std::str::from_utf8(name).ok()?;
                        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_').then(|| name.to_string())?
                    }
                    _ => return None,
                };
                Some(if t & 0x80 != 0 { format!("{base}?") } else { base })
            })
            .collect()
    }

    pub fn insn(&self, pc: usize) -> Insn {
        let word = self.code[pc];
        let op = Op::from_byte(word as u8).expect("opcodes are checked when reading");
        let aux = if op.has_aux() { self.code.get(pc + 1).copied().unwrap_or(0) } else { 0 };
        Insn { pc, op, a: (word >> 8) as u8, b: (word >> 16) as u8, c: (word >> 24) as u8, aux, word }
    }

    pub fn insns(&self) -> Vec<Insn> {
        let mut out = Vec::new();
        let mut pc = 0;
        while pc < self.code.len() {
            let insn = self.insn(pc);
            pc += insn.len();
            out.push(insn);
        }
        out
    }

    pub fn string(&self, index: u32) -> Option<&[u8]> {
        match self.constants.get(index as usize)? {
            Constant::String(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub version: u8,
    // bytes after the main proto index; Roblox appends 24 (a hash or signature)
    pub trailing: usize,
    pub protos: Vec<Proto>,
    pub main: usize,
    pub types_version: u8,
    // userdata type names the compiler was given, by index from 1 (types version 3)
    pub userdata_types: Vec<(u8, Vec<u8>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadError(pub String);

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> Result<u8, ReadError> {
        let byte = *self.data.get(self.at).ok_or_else(|| ReadError(format!("unexpected end at byte {}", self.at)))?;
        self.at += 1;
        Ok(byte)
    }

    fn bytes(&mut self, n: usize) -> Result<&'a [u8], ReadError> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.data.len());
        let end = end.ok_or_else(|| ReadError(format!("unexpected end reading {n} bytes at {}", self.at)))?;
        let out = &self.data[self.at..end];
        self.at = end;
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn f64(&mut self) -> Result<f64, ReadError> {
        Ok(f64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }

    fn varint(&mut self) -> Result<u64, ReadError> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = self.u8()?;
            value |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(ReadError("varint too long".into()))
    }

    fn count(&mut self) -> Result<usize, ReadError> {
        let n = self.varint()? as usize;
        // every counted item takes at least a byte, so a count past the end is corrupt
        if n > self.data.len() - self.at.min(self.data.len()) {
            return Err(ReadError(format!("count {n} at byte {} runs past the end", self.at)));
        }
        Ok(n)
    }

    fn string_ref(&mut self, strings: &[Vec<u8>]) -> Result<Option<Vec<u8>>, ReadError> {
        match self.varint()? as usize {
            0 => Ok(None),
            i => strings.get(i - 1).cloned().map(Some).ok_or_else(|| ReadError(format!("string {i} out of range"))),
        }
    }
}

// `key` undoes Roblox's opcode encoding: op = byte * key % 256 (1 for plain Luau, 203 for Roblox)
pub fn read(data: &[u8], key: u8) -> Result<Chunk, ReadError> {
    let mut r = Reader { data, at: 0 };
    let version = r.u8()?;
    if version == 0 {
        return Err(ReadError(format!("compile error: {}", String::from_utf8_lossy(&data[1..]))));
    }
    if !(3..=14).contains(&version) {
        return Err(ReadError(format!("unsupported bytecode version {version}")));
    }
    let types_version = if version >= 4 { r.u8()? } else { 0 };
    if types_version > 3 {
        return Err(ReadError(format!("unsupported type info version {types_version}")));
    }

    let strings = (0..r.count()?)
        .map(|_| {
            let len = r.varint()? as usize;
            Ok(r.bytes(len)?.to_vec())
        })
        .collect::<Result<Vec<_>, ReadError>>()?;

    // userdata types the host named (Vector3, CFrame...), by index from 1
    let mut userdata_types = Vec::new();
    if types_version == 3 {
        loop {
            let index = r.u8()?;
            if index == 0 {
                break;
            }
            userdata_types.push((index, r.string_ref(&strings)?.unwrap_or_default()));
        }
    }

    let proto_count = r.count()?;
    let mut protos = Vec::with_capacity(proto_count);
    for _ in 0..proto_count {
        let size = if version >= 12 { Some(r.varint()? as usize) } else { None };
        let start = r.at;

        let max_stack = r.u8()?;
        let num_params = r.u8()?;
        let num_upvalues = r.u8()?;
        let is_vararg = r.u8()? != 0;
        let mut type_info = Vec::new();
        if version >= 4 {
            let _flags = r.u8()?;
            if types_version > 0 {
                let len = r.varint()? as usize;
                type_info = r.bytes(len)?.to_vec();
            }
        }

        let code = (0..r.count()?).map(|_| r.u32()).collect::<Result<Vec<_>, _>>()?;
        let code = decode_ops(code, key)?;

        let constant_count = r.count()?;
        let mut constants = Vec::with_capacity(constant_count);
        for _ in 0..constant_count {
            constants.push(match r.u8()? {
                0 => Constant::Nil,
                1 => Constant::Boolean(r.u8()? != 0),
                2 => Constant::Number(r.f64()?),
                3 => Constant::String(r.string_ref(&strings)?.unwrap_or_default()),
                4 => {
                    let id = r.u32()?;
                    let len = (id >> 30) as usize;
                    Constant::Import((0..len).map(|i| (id >> (20 - 10 * i)) & 1023).collect())
                }
                5 => Constant::Table((0..r.count()?).map(|_| Ok((r.varint()? as u32, None))).collect::<Result<_, ReadError>>()?),
                6 => Constant::Closure(r.varint()? as u32),
                7 => Constant::Vector([r.f32()? as f64, r.f32()? as f64, r.f32()? as f64, r.f32()? as f64]),
                8 => Constant::Table(
                    (0..r.count()?)
                        .map(|_| {
                            let key = r.varint()? as u32;
                            let value = r.u32()? as i32;
                            Ok((key, (value >= 0).then_some(value as u32)))
                        })
                        .collect::<Result<_, ReadError>>()?,
                ),
                9 => {
                    let negative = r.u8()? != 0;
                    let magnitude = r.varint()?;
                    Constant::Integer(if negative { (!magnitude).wrapping_add(1) as i64 } else { magnitude as i64 })
                }
                10 => {
                    r.varint()?;
                    let members = r.varint()? + r.varint()?;
                    for _ in 0..members {
                        r.varint()?;
                    }
                    Constant::ClassShape
                }
                11 => Constant::Vector([r.f64()?, r.f64()?, r.f64()?, r.f64()?]),
                tag => return Err(ReadError(format!("unknown constant tag {tag}"))),
            });
        }

        let children = (0..r.count()?).map(|_| Ok(r.varint()? as usize)).collect::<Result<Vec<_>, ReadError>>()?;
        let line_defined = r.varint()? as u32;
        let debug_name = r.string_ref(&strings)?;

        let lines = if r.u8()? != 0 {
            let gap = r.u8()?;
            if gap > 31 {
                return Err(ReadError(format!("line gap log2 {gap}")));
            }
            let mut offsets = Vec::with_capacity(code.len());
            let mut last = 0u8;
            for _ in 0..code.len() {
                last = last.wrapping_add(r.u8()?);
                offsets.push(last);
            }
            let intervals = if code.is_empty() { 0 } else { ((code.len() - 1) >> gap) + 1 };
            let mut absolute = Vec::with_capacity(intervals);
            let mut line = 0i32;
            for _ in 0..intervals {
                line = line.wrapping_add(r.u32()? as i32);
                absolute.push(line);
            }
            Some((0..code.len()).map(|pc| (absolute[pc >> gap] + offsets[pc] as i32) as u32).collect())
        } else {
            None
        };

        let mut locals = Vec::new();
        let mut upvalue_names = Vec::new();
        if r.u8()? != 0 {
            for _ in 0..r.count()? {
                locals.push(LocalVar {
                    name: r.string_ref(&strings)?.unwrap_or_default(),
                    start_pc: r.varint()? as usize,
                    end_pc: r.varint()? as usize,
                    register: r.u8()?,
                });
            }
            for _ in 0..r.count()? {
                upvalue_names.push(r.string_ref(&strings)?.unwrap_or_default());
            }
        }

        match size {
            // the size covers feedback vectors, cost and anything newer, so skip by it
            Some(size) => r.at = start + size,
            None if version >= 11 => {
                for _ in 0..r.count()? {
                    r.u8()?;
                    r.varint()?;
                }
            }
            None => {}
        }

        protos.push(Proto {
            max_stack,
            num_params,
            num_upvalues,
            is_vararg,
            code,
            constants,
            children,
            line_defined,
            debug_name,
            lines,
            locals,
            upvalue_names,
            type_info,
        });
    }

    let main = r.varint()? as usize;
    if main >= protos.len() {
        return Err(ReadError(format!("main proto {main} out of range")));
    }
    for proto in &protos {
        if proto.children.iter().any(|&c| c >= protos.len()) {
            return Err(ReadError("child proto out of range".into()));
        }
    }
    Ok(Chunk { version, trailing: data.len() - r.at, protos, main, types_version, userdata_types })
}

fn decode_ops(mut code: Vec<u32>, key: u8) -> Result<Vec<u32>, ReadError> {
    let mut pc = 0;
    while pc < code.len() {
        let byte = (code[pc] as u8).wrapping_mul(key);
        let op = Op::from_byte(byte).ok_or_else(|| ReadError(format!("bad opcode {byte} at pc {pc} (wrong encode key?)")))?;
        code[pc] = (code[pc] & !0xff) | byte as u32;
        pc += if op.has_aux() { 2 } else { 1 };
    }
    if pc != code.len() {
        return Err(ReadError("last instruction is missing its AUX word".into()));
    }
    Ok(code)
}
