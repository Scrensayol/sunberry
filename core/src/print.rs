use std::collections::HashSet;

use crate::ast::{BinOp, Capture, Closure, Expr, Field, Function, Stmt, UnOp};

const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local", "nil", "not",
    "or", "repeat", "return", "then", "true", "until", "while",
];
// (`continue` isn't one: Luau only reads it as a statement where it can't be a call or an
// assignment, so `while continue do` reads a variable)

pub fn is_identifier(name: &[u8]) -> bool {
    let Ok(name) = std::str::from_utf8(name) else { return false };
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !KEYWORDS.contains(&name)
}

// Precedence, loosest first. Atoms and prefix expressions sit above every operator.
const IF_ELSE: u8 = 0;
const OR: u8 = 1;
const AND: u8 = 2;
const COMPARE: u8 = 3;
const CONCAT: u8 = 4;
const ADD: u8 = 5;
const MUL: u8 = 6;
const UNARY: u8 = 7;
const POW: u8 = 8;
const ATOM: u8 = 10;

fn binary_precedence(op: BinOp) -> (u8, bool, &'static str) {
    match op {
        BinOp::Or => (OR, false, "or"),
        BinOp::And => (AND, false, "and"),
        BinOp::Concat => (CONCAT, true, ".."),
        BinOp::Add => (ADD, false, "+"),
        BinOp::Sub => (ADD, false, "-"),
        BinOp::Mul => (MUL, false, "*"),
        BinOp::Div => (MUL, false, "/"),
        BinOp::IDiv => (MUL, false, "//"),
        BinOp::Mod => (MUL, false, "%"),
        BinOp::Pow => (POW, true, "^"),
        BinOp::Eq => (COMPARE, false, "=="),
        BinOp::Ne => (COMPARE, false, "~="),
        BinOp::Lt => (COMPARE, false, "<"),
        BinOp::Le => (COMPARE, false, "<="),
        BinOp::Gt => (COMPARE, false, ">"),
        BinOp::Ge => (COMPARE, false, ">="),
    }
}

fn precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Binary(op, ..) => binary_precedence(*op).0,
        // printed as `a ~= b`
        Expr::Unary(UnOp::Not, inner) if matches!(inner.as_ref(), Expr::Binary(BinOp::Eq | BinOp::Ne, ..)) => COMPARE,
        Expr::Unary(..) => UNARY,
        Expr::Number(n) if n.is_finite() && n.is_sign_negative() => UNARY,
        Expr::Number(n) if n.is_nan() && n.is_sign_positive() => UNARY,
        Expr::IfElse(..) => IF_ELSE,
        _ => ATOM,
    }
}

fn is_prefix(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Local(_) | Expr::Upvalue(_) | Expr::Global(_) | Expr::Index(..) | Expr::Call { .. } | Expr::MethodCall { .. }
    )
}

// the operands of a chain of one operator, `a or b or c` however it nests
fn flatten<'e>(op: BinOp, e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match e {
        Expr::Binary(o, left, right) if *o == op => {
            flatten(op, left, out);
            flatten(op, right, out);
        }
        e => out.push(e),
    }
}

fn is_chain(e: &Expr) -> bool {
    matches!(e, Expr::Binary(BinOp::And | BinOp::Or, ..))
}

// printed text moved one level deeper (strings never hold a raw line break)
fn deeper(text: &str) -> String {
    text.replace('\n', "\n\t")
}

// whether parts joined by `sep` fit in `limit` on the lines where they meet: the lines inside a
// part spread over several (a closure, a table, a wrapped call) were already laid out at its level
fn fits(parts: &[&str], sep: &str, limit: usize) -> bool {
    let mut line = 0;
    for (i, part) in parts.iter().enumerate() {
        let first = part.split('\n').next().unwrap();
        line += if i == 0 { 0 } else { sep.len() } + first.len();
        if let Some((_, last)) = part.rsplit_once('\n') {
            if line > limit {
                return false;
            }
            line = last.len();
        }
    }
    line <= limit
}

pub fn number(n: f64) -> String {
    // 0 / 0 makes x86's default NaN, which has the sign bit set; negating it clears the sign.
    // The lifter refuses NaNs with any other payload.
    if n.is_nan() {
        if n.is_sign_negative() { "(0 / 0)".into() } else { "-(0 / 0)".into() }
    } else if n.is_infinite() {
        if n > 0.0 { "(1 / 0)".into() } else { "(-1 / 0)".into() }
    } else if n == 0.0 {
        if n.is_sign_negative() { "-0".into() } else { "0".into() }
    } else if n == n.trunc() && n.abs() < 1e16 {
        format!("{}", n as i64)
    } else if n.abs() >= 1e16 || n.abs() < 1e-5 {
        format!("{n:e}")
    } else {
        format!("{n}")
    }
}

pub fn string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() + 2);
    out.push('"');
    escape_into(bytes, '"', &mut out);
    out.push('"');
    out
}

// the literal text between an interpolated string's `{}` parts
fn interpolated_text(bytes: &[u8], out: &mut String) {
    escape_into(bytes, '`', out);
}

fn escape_into(bytes: &[u8], quote: char, out: &mut String) {
    let mut rest = bytes;
    while !rest.is_empty() {
        let (valid, bad) = match std::str::from_utf8(rest) {
            Ok(s) => (s, &[][..]),
            Err(e) => (std::str::from_utf8(&rest[..e.valid_up_to()]).unwrap(), &rest[e.valid_up_to()..]),
        };
        for c in valid.chars() {
            match c {
                c if c == quote => {
                    out.push('\\');
                    out.push(c);
                }
                // `{` opens an expression in an interpolated string
                '{' if quote == '`' => out.push_str("\\{"),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\{:03}", c as u32)),
                c => out.push(c),
            }
        }
        match bad.split_first() {
            Some((&byte, after)) => {
                out.push_str(&format!("\\{byte:03}"));
                rest = after;
            }
            None => rest = &[],
        }
    }
}

// whether `local` is indexed or has a method called on it in `body`, or in a closure there that
// captures it
fn used_as_object(functions: &[Function], body: &[Stmt], local: u32) -> bool {
    fn uses(functions: &[Function], body: &[Stmt], var: &Expr) -> bool {
        let expr = |e: &Expr| {
            let mut found = false;
            visit_expr(e, &mut |e| match e {
                Expr::Index(t, _) | Expr::MethodCall { object: t, .. } if **t == *var => found = true,
                Expr::Closure(c) => found |= captured_use(functions, c, var),
                _ => {}
            });
            found
        };
        body.iter().any(|stmt| {
            let here = match stmt {
                Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().any(expr),
                Stmt::Assign { targets, values } => targets.iter().chain(values).any(expr),
                Stmt::Call(e) => expr(e),
                Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond),
                Stmt::NumericFor { from, to, step, .. } => expr(from) || expr(to) || step.as_ref().is_some_and(expr),
                Stmt::GenericFor { values, .. } => values.iter().any(expr),
                Stmt::LocalFunction { closure, .. } => captured_use(functions, closure, var),
                Stmt::Break | Stmt::Continue | Stmt::Do(_) => false,
            };
            here || stmt.bodies().into_iter().any(|b| uses(functions, b, var))
        })
    }
    fn captured_use(functions: &[Function], closure: &Closure, var: &Expr) -> bool {
        closure.captures.iter().enumerate().any(|(k, c)| {
            let same = match (c, var) {
                (Capture::Local(l), Expr::Local(v)) => l == v,
                (Capture::Upvalue(u), Expr::Upvalue(v)) => u == v,
                _ => false,
            };
            same && uses(functions, &functions[closure.proto].body, &Expr::Upvalue(k as u8))
        })
    }
    uses(functions, body, &Expr::Local(local))
}

// every expression inside `e`, `e` included, but not inside functions it creates
fn visit_expr(e: &Expr, f: &mut impl FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Index(a, b) | Expr::Binary(_, a, b) => {
            visit_expr(a, f);
            visit_expr(b, f);
        }
        Expr::Unary(_, a) => visit_expr(a, f),
        Expr::IfElse(a, b, c) => {
            visit_expr(a, f);
            visit_expr(b, f);
            visit_expr(c, f);
        }
        Expr::Call { func, args, .. } => {
            visit_expr(func, f);
            args.iter().for_each(|a| visit_expr(a, f));
        }
        Expr::MethodCall { object, args, .. } => {
            visit_expr(object, f);
            args.iter().for_each(|a| visit_expr(a, f));
        }
        Expr::Table(fields) => fields.iter().for_each(|field| match field {
            Field::Positional(v) => visit_expr(v, f),
            Field::Keyed(k, v) => {
                visit_expr(k, f);
                visit_expr(v, f);
            }
        }),
        _ => {}
    }
}

fn compound(target: &Expr) -> bool {
    match target {
        Expr::Local(_) | Expr::Upvalue(_) | Expr::Global(_) => true,
        Expr::Index(table, key) => {
            matches!(table.as_ref(), Expr::Local(_)) && matches!(key.as_ref(), Expr::Local(_) | Expr::String(_) | Expr::Number(_))
        }
        _ => false,
    }
}

fn compound_symbol(op: BinOp, right: &Expr) -> Option<&'static str> {
    Some(match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::IDiv => "//",
        BinOp::Mod => "%",
        BinOp::Pow => "^",
        // `s = s .. a .. b` concatenates all three at once; `s ..= a .. b` doesn't
        BinOp::Concat if !matches!(right, Expr::Binary(BinOp::Concat, ..)) => "..",
        _ => return None,
    })
}

// `("%* of %*"):format(a, b)` is exactly what Luau compiles `{a} of {b}` in backticks to:
// the literal text with `%` doubled and `%*` for each expression
// (one value, like the call it stands for: one whose results spread stays a call)
fn interpolation(object: &Expr, method: &[u8], args: &[Expr], multret: bool) -> Option<Vec<Vec<u8>>> {
    let Expr::String(format) = object else { return None };
    if multret || method != b"format" || args.is_empty() || args.last().is_some_and(Expr::expands) {
        return None;
    }
    let mut parts = vec![Vec::new()];
    let mut bytes = format.iter();
    while let Some(&b) = bytes.next() {
        if b != b'%' {
            parts.last_mut().unwrap().push(b);
            continue;
        }
        match bytes.next() {
            Some(b'%') => parts.last_mut().unwrap().push(b'%'),
            Some(b'*') => parts.push(Vec::new()),
            _ => return None,
        }
    }
    (parts.len() == args.len() + 1).then_some(parts)
}

struct Scope {
    proto: usize,
    locals: Vec<Option<String>>,
    upvalues: Vec<String>,
}

struct Printer<'a> {
    functions: &'a [Function],
    reserved: HashSet<Vec<u8>>,
    taken: HashSet<String>,
    counter: usize,
    scopes: Vec<Scope>,
    // names called with `:` somewhere: a function stored under one is likely a method
    methods: HashSet<Vec<u8>>,
    // tables set up as classes (`X.__index = X`), by printed name
    class_tables: HashSet<String>,
    // globals Luau won't import here: assigned somewhere in the chunk, or all of them when it
    // uses getfenv/setfenv (None)
    no_imports: Option<HashSet<Vec<u8>>>,
    // per function, the locals something reads, writes or captures
    used: Vec<HashSet<u32>>,
}

// Globals whose fields Roblox's compiler never imports (its mutable globals), and Enum, whose
// values never change, so a cached lookup and one at run time read the same thing
const ROBLOX_NOT_IMPORTED: &[&[u8]] = &[b"game", b"Game", b"workspace", b"Workspace", b"script", b"plugin", b"shared", b"_G", b"Enum"];

// `roblox`: the bytecode came from Roblox's compiler, which imports differently (see
// Expr::RuntimeKey)
pub fn print_chunk(functions: &[Function], main: usize, roblox: bool) -> String {
    let mut reserved = HashSet::new();
    let mut methods = HashSet::new();
    for function in functions {
        collect_globals(&function.body, &mut reserved);
        collect_methods(&function.body, &mut methods);
    }
    // printed for vector constants
    reserved.insert(b"Vector3".to_vec());
    let mut written = HashSet::new();
    for function in functions {
        collect_written_globals(&function.body, &mut written);
    }
    let fenv = reserved.contains(&b"getfenv"[..]) || reserved.contains(&b"setfenv"[..]);
    if roblox {
        written.extend(ROBLOX_NOT_IMPORTED.iter().map(|g| g.to_vec()));
    }
    let no_imports = (!fenv).then_some(written);
    let used = functions
        .iter()
        .map(|f| {
            let mut used = HashSet::new();
            used_in(&f.body, &mut used);
            used
        })
        .collect();
    let mut printer = Printer { functions, reserved, taken: HashSet::new(), counter: 0, scopes: Vec::new(), methods, used, class_tables: HashSet::new(), no_imports };
    printer.scopes.push(Scope { proto: main, locals: vec![None; functions[main].names.len()], upvalues: Vec::new() });
    let mut out = String::new();
    printer.body(&functions[main].body, 0, &mut out);
    out
}

fn used_in(body: &[Stmt], out: &mut HashSet<u32>) {
    fn expr(e: &Expr, out: &mut HashSet<u32>) {
        match e {
            Expr::Local(l) => {
                out.insert(*l);
            }
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            Expr::Unary(_, a) => expr(a, out),
            Expr::IfElse(a, b, c) => {
                expr(a, out);
                expr(b, out);
                expr(c, out);
            }
            Expr::Call { func, args, .. } => {
                expr(func, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::Table(fields) => fields.iter().for_each(|f| match f {
                Field::Positional(v) => expr(v, out),
                Field::Keyed(k, v) => {
                    expr(k, out);
                    expr(v, out);
                }
            }),
            Expr::Closure(c) => closure(c, out),
            Expr::Nil | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::RuntimeKey(_) | Expr::Vector(_) | Expr::VarArg { .. } => {}
            Expr::Upvalue(_) | Expr::Global(_) | Expr::Tested(_) => {}
        }
    }
    fn closure(c: &Closure, out: &mut HashSet<u32>) {
        for capture in &c.captures {
            if let Capture::Local(l) = capture {
                out.insert(*l);
            }
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, out)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, out)),
            Stmt::Call(e) => expr(e, out),
            Stmt::LocalFunction { local, closure: c } => {
                out.insert(*local);
                closure(c, out);
            }
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, out),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, out);
                expr(to, out);
                step.iter().for_each(|s| expr(s, out));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, out)),
            Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for body in stmt.bodies() {
            used_in(body, out);
        }
    }
}

fn collect_written_globals(body: &[Stmt], out: &mut HashSet<Vec<u8>>) {
    for stmt in body {
        if let Stmt::Assign { targets, .. } = stmt {
            for target in targets {
                if let Expr::Global(g) = target {
                    out.insert(g.clone());
                }
            }
        }
        for body in stmt.bodies() {
            collect_written_globals(body, out);
        }
    }
}

fn collect_methods(body: &[Stmt], out: &mut HashSet<Vec<u8>>) {
    fn expr(e: &Expr, out: &mut HashSet<Vec<u8>>) {
        match e {
            Expr::MethodCall { object, method, args, .. } => {
                out.insert(method.clone());
                expr(object, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            Expr::Unary(_, a) => expr(a, out),
            Expr::IfElse(a, b, c) => {
                expr(a, out);
                expr(b, out);
                expr(c, out);
            }
            Expr::Call { func, args, .. } => {
                expr(func, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::Table(fields) => fields.iter().for_each(|f| match f {
                Field::Positional(v) => expr(v, out),
                Field::Keyed(k, v) => {
                    expr(k, out);
                    expr(v, out);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, out)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, out)),
            Stmt::Call(e) => expr(e, out),
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, out),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, out);
                expr(to, out);
                step.iter().for_each(|s| expr(s, out));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, out)),
            Stmt::LocalFunction { .. } | Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for body in stmt.bodies() {
            collect_methods(body, out);
        }
    }
}

fn collect_globals(body: &[Stmt], out: &mut HashSet<Vec<u8>>) {
    fn expr(e: &Expr, out: &mut HashSet<Vec<u8>>) {
        match e {
            Expr::Global(name) => {
                out.insert(name.clone());
            }
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, out);
                expr(b, out);
            }
            Expr::Unary(_, a) => expr(a, out),
            Expr::IfElse(a, b, c) => {
                expr(a, out);
                expr(b, out);
                expr(c, out);
            }
            Expr::Call { func, args, .. } => {
                expr(func, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, out);
                args.iter().for_each(|a| expr(a, out));
            }
            Expr::Table(fields) => fields.iter().for_each(|f| match f {
                Field::Positional(v) => expr(v, out),
                Field::Keyed(k, v) => {
                    expr(k, out);
                    expr(v, out);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, out)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, out)),
            Stmt::Call(e) => expr(e, out),
            Stmt::LocalFunction { .. } => {}
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, out),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, out);
                expr(to, out);
                step.iter().for_each(|s| expr(s, out));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, out)),
            Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for body in stmt.bodies() {
            collect_globals(body, out);
        }
    }
}

impl Printer<'_> {
    fn declare(&mut self, local: u32) -> String {
        self.declare_as(local, "v")
    }

    // `prefix` for a local with no name to go on: v1, or p1 for a parameter
    fn declare_as(&mut self, local: u32, prefix: &str) -> String {
        let proto = self.scopes.last().unwrap().proto;
        let wanted = self.functions[proto].names.get(local as usize).cloned().flatten().filter(|n| is_identifier(n));
        // every method has a `self` of its own, so `self` isn't handed out once like other
        // names; it's only given where no other `self` could be in sight
        if wanted.as_deref() == Some(b"self") {
            let visible = self.scopes.iter().any(|s| s.locals.iter().flatten().chain(&s.upvalues).any(|n| n == "self"));
            // the main chunk's objects are class tables, not a constructor's object
            if !visible && self.scopes.len() > 1 && !self.reserved.contains(&b"self"[..]) {
                self.scopes.last_mut().unwrap().locals[local as usize] = Some("self".into());
                return "self".into();
            }
        }
        let wanted = wanted.filter(|n| n != b"self");
        let free = |p: &Self, n: &str| !p.taken.contains(n) && !p.reserved.contains(n.as_bytes()) && n != "self";
        let name = match wanted.map(|n| String::from_utf8(n).unwrap()) {
            // a name of its own, numbered from 2 when it's taken
            Some(base) => (1..)
                .map(|i| if i == 1 { base.clone() } else { format!("{base}{i}") })
                .find(|c| free(self, c))
                .unwrap(),
            None => loop {
                self.counter += 1;
                let candidate = format!("{prefix}{}", self.counter);
                if free(self, &candidate) {
                    break candidate;
                }
            },
        };
        self.taken.insert(name.clone());
        self.scopes.last_mut().unwrap().locals[local as usize] = Some(name.clone());
        name
    }

    // a loop variable or parameter nothing uses is `_`, unless `_` already names something
    // that could be in scope or is a global the chunk reads
    fn declare_maybe_unused(&mut self, local: u32, prefix: &str) -> String {
        let proto = self.scopes.last().unwrap().proto;
        if !self.used[proto].contains(&local) && !self.taken.contains("_") && !self.reserved.contains(&b"_"[..]) {
            self.scopes.last_mut().unwrap().locals[local as usize] = Some("_".into());
            return "_".into();
        }
        self.declare_as(local, prefix)
    }

    fn local(&self, local: u32) -> String {
        self.scopes.last().unwrap().locals[local as usize].clone().expect("locals are declared before use")
    }

    fn body(&mut self, body: &[Stmt], indent: usize, out: &mut String) {
        // a trailing bare `return` is implied at the end of a function
        let body = match body.last() {
            Some(Stmt::Return(values)) if values.is_empty() => &body[..body.len() - 1],
            _ => body,
        };
        self.block(body, indent, out);
    }

    fn stmt(&mut self, stmt: &Stmt, indent: usize) -> String {
        if let Stmt::Assign { targets, values } = stmt
            && let ([Expr::Index(table, key)], [value]) = (&targets[..], &values[..])
            && **key == Expr::String(b"__index".to_vec())
            && **table == *value
            && let Some(path) = self.path(value)
        {
            self.class_tables.insert(path);
        }
        match stmt {
            // `local f = function` can't see f, so it is `local function f`
            Stmt::Local { locals, values } if locals.len() == 1 && matches!(values[..], [Expr::Closure(_)]) => {
                let Expr::Closure(closure) = &values[0] else { unreachable!() };
                let function = self.closure(closure, indent);
                let name = self.declare(locals[0]);
                format!("local function {name}{}", &function["function".len()..])
            }
            // nil is what a bare local starts as
            Stmt::Local { locals, values } if !values.is_empty() && values.len() <= locals.len() && values.iter().all(|v| *v == Expr::Nil) => {
                let names = locals.iter().map(|&l| self.declare(l)).collect::<Vec<_>>();
                format!("local {}", names.join(", "))
            }
            Stmt::Assign { targets, values } if targets.len() == 1 && matches!(values[..], [Expr::Closure(_)]) && self.path(&targets[0]).is_some() => {
                let Expr::Closure(closure) = &values[0] else { unreachable!() };
                let path = self.path(&targets[0]).unwrap();
                let method = match &targets[0] {
                    Expr::Index(table, key) => matches!(key.as_ref(), Expr::String(k) if self.is_method(closure, table, k)),
                    _ => false,
                };
                if method {
                    let (dot, name) = path.rsplit_once('.').unwrap();
                    let function = self.closure_as(closure, indent, true);
                    format!("function {dot}:{name}{}", &function["function".len()..])
                } else {
                    let function = self.closure(closure, indent);
                    format!("function {path}{}", &function["function".len()..])
                }
            }
            Stmt::Local { locals, values } if locals.len() == 1 && values.len() == 1 && is_chain(&values[0]) => {
                let name = self.declare(locals[0]);
                let prefix = format!("local {name} = ");
                format!("{prefix}{}", self.chain(&values[0], prefix.len(), indent))
            }
            Stmt::Local { locals, values } => {
                // `Roact.Component:extend("Name")` makes a class whose functions are methods
                let class = matches!(&values[..], [Expr::MethodCall { method, args, .. }] if method == b"extend" && matches!(args[..], [Expr::String(_)]));
                let values = self.list_for(values, locals.len(), indent);
                let names = locals.iter().map(|&l| self.declare(l)).collect::<Vec<_>>();
                if class {
                    self.class_tables.insert(names[0].clone());
                }
                if values.is_empty() {
                    format!("local {}", names.join(", "))
                } else {
                    format!("local {} = {values}", names.join(", "))
                }
            }
            Stmt::LocalFunction { local, closure } => {
                let name = self.declare(*local);
                let function = self.closure(closure, indent);
                format!("local function {name}{}", &function["function".len()..])
            }
            // `x = x + y` as `x += y`, where the target is read and written the same way both
            // times: a register, a global, or a register's field with a constant or register key
            Stmt::Assign { targets, values }
                if let ([target], [Expr::Binary(op, left, right)]) = (&targets[..], &values[..])
                    && **left == *target
                    && compound(target)
                    && let Some(symbol) = compound_symbol(*op, right) =>
            {
                let target = self.expr(target, indent);
                format!("{target} {symbol}= {}", self.expr(right, indent))
            }
            Stmt::Assign { targets, values } if targets.len() == 1 && values.len() == 1 && is_chain(&values[0]) => {
                let prefix = format!("{} = ", self.expr(&targets[0], indent));
                format!("{prefix}{}", self.chain(&values[0], prefix.len(), indent))
            }
            Stmt::Assign { targets, values } => {
                let targets = targets.iter().map(|t| self.expr(t, indent)).collect::<Vec<_>>();
                let values = self.list_for(values, targets.len(), indent);
                format!("{} = {values}", targets.join(", "))
            }
            // a string literal can't start a statement, so no interpolation here
            Stmt::Call(Expr::MethodCall { object, method, args, .. }) => self.method_call(object, method, args, indent),
            Stmt::Call(call) => self.expr(call, indent),
            Stmt::Return(values) if values.len() == 1 && is_chain(&values[0]) => format!("return {}", self.chain(&values[0], "return ".len(), indent)),
            Stmt::Return(values) => {
                let values = self.list(values, indent);
                if values.is_empty() { "return".into() } else { format!("return {values}") }
            }
            Stmt::If { cond, then_body, else_body } => {
                let mut out = String::new();
                self.if_chain(cond, then_body, else_body, indent, &mut out);
                out
            }
            Stmt::While { cond, body } => {
                let cond = self.condition(cond, indent);
                let mut out = if cond.starts_with('\n') { format!("while{cond}do\n") } else { format!("while {cond} do\n") };
                self.block(body, indent + 1, &mut out);
                out.push_str(&"\t".repeat(indent));
                out.push_str("end");
                out
            }
            Stmt::Repeat { body, cond } => {
                let mut out = "repeat\n".to_string();
                self.block(body, indent + 1, &mut out);
                // the condition sees the body's locals, so it is printed after them
                let cond = if is_chain(cond) { self.chain(cond, "until ".len(), indent) } else { self.expr(cond, indent) };
                out.push_str(&"\t".repeat(indent));
                out.push_str(&format!("until {cond}"));
                out
            }
            Stmt::NumericFor { var, from, to, step, body } => {
                let (from, to) = (self.expr(from, indent), self.expr(to, indent));
                let step = step.as_ref().map(|s| format!(", {}", self.expr(s, indent))).unwrap_or_default();
                let var = self.declare_maybe_unused(*var, "v");
                let mut out = format!("for {var} = {from}, {to}{step} do\n");
                self.block(body, indent + 1, &mut out);
                out.push_str(&"\t".repeat(indent));
                out.push_str("end");
                out
            }
            Stmt::GenericFor { vars, values, body } => {
                let values = self.list(values, indent);
                let vars = vars.iter().map(|&v| self.declare_maybe_unused(v, "v")).collect::<Vec<_>>();
                let mut out = format!("for {} in {values} do\n", vars.join(", "));
                self.block(body, indent + 1, &mut out);
                out.push_str(&"\t".repeat(indent));
                out.push_str("end");
                out
            }
            Stmt::Break => "break".into(),
            Stmt::Continue => "continue".into(),
            Stmt::Do(body) => {
                let mut out = "do\n".to_string();
                self.block(body, indent + 1, &mut out);
                out.push_str(&"\t".repeat(indent));
                out.push_str("end");
                out
            }
        }
    }

    fn if_chain(&mut self, cond: &Expr, then_body: &[Stmt], else_body: &[Stmt], indent: usize, out: &mut String) {
        // `if not c then a else b end` reads better the other way round
        if let Expr::Unary(UnOp::Not, inner) = cond
            && !else_body.is_empty()
            && !matches!(else_body, [Stmt::If { .. }])
        {
            return self.if_chain(inner, else_body, then_body, indent, out);
        }
        let pad = "\t".repeat(indent);
        let cond = self.condition(cond, indent);
        out.push_str(&if cond.starts_with('\n') { format!("if{cond}then\n") } else { format!("if {cond} then\n") });
        self.block(then_body, indent + 1, out);
        match else_body {
            [] => {}
            [Stmt::If { cond, then_body, else_body }] => {
                out.push_str(&pad);
                out.push_str("else");
                return self.if_chain(cond, then_body, else_body, indent, out);
            }
            body => {
                out.push_str(&pad);
                out.push_str("else\n");
                self.block(body, indent + 1, out);
            }
        }
        out.push_str(&pad);
        out.push_str("end");
    }

    // a nested block: unlike a function body, a trailing bare `return` is kept
    fn block(&mut self, body: &[Stmt], indent: usize, out: &mut String) {
        let pad = "\t".repeat(indent);
        let mut previous_end = None;
        for stmt in body {
            let text = self.stmt(stmt, indent);
            // a statement starting with `(` would otherwise continue the previous line as a call
            if text.starts_with('(')
                && let Some(end) = previous_end
            {
                out.insert(end, ';');
            }
            out.push_str(&pad);
            out.push_str(&text);
            previous_end = Some(out.len());
            out.push('\n');
        }
    }

    // values assigned to `targets` names: a single-valued call in last place needs no
    // parentheses unless there are more names than values for it to spread into
    fn list_for(&mut self, values: &[Expr], targets: usize, indent: usize) -> String {
        let last = values.len().saturating_sub(1);
        values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i == last && targets > values.len() { self.expr_last(v, indent) } else { self.expr(v, indent) }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    // an expression list where the last value spreads (arguments, returns)
    fn list(&mut self, values: &[Expr], indent: usize) -> String {
        let last = values.len().saturating_sub(1);
        values
            .iter()
            .enumerate()
            .map(|(i, v)| if i == last { self.expr_last(v, indent) } else { self.expr(v, indent) })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn expr_last(&mut self, expr: &Expr, indent: usize) -> String {
        let truncated = matches!(
            expr,
            Expr::VarArg { multret: false } | Expr::Call { multret: false, .. } | Expr::MethodCall { multret: false, .. }
        ) && !matches!(expr, Expr::MethodCall { object, method, args, multret } if interpolation(object, method, args, *multret).is_some());
        let text = self.expr(expr, indent);
        if truncated { format!("({text})") } else { text }
    }

    fn operand(&mut self, expr: &Expr, wrap: bool, indent: usize) -> String {
        let text = self.expr(expr, indent);
        if wrap { format!("({text})") } else { text }
    }

    fn prefix(&mut self, expr: &Expr, indent: usize) -> String {
        // `(`{a}`):upper()`: an interpolated string can't be called or indexed bare
        let interpolated = matches!(expr, Expr::MethodCall { object, method, args, multret } if interpolation(object, method, args, *multret).is_some());
        let wrap = !is_prefix(expr) || interpolated;
        self.operand(expr, wrap, indent)
    }

    fn expr(&mut self, expr: &Expr, indent: usize) -> String {
        match expr {
            Expr::Nil => "nil".into(),
            Expr::Bool(b) => b.to_string(),
            Expr::Number(n) => number(*n),
            Expr::String(s) | Expr::RuntimeKey(s) => string(s),
            Expr::Vector(v) => format!("Vector3.new({}, {}, {})", number(v[0]), number(v[1]), number(v[2])),
            Expr::VarArg { .. } => "...".into(),
            Expr::Local(l) => self.local(*l),
            Expr::Upvalue(u) => self.scopes.last().unwrap().upvalues[*u as usize].clone(),
            Expr::Global(name) => String::from_utf8_lossy(name).into_owned(),
            Expr::Index(table, key) => {
                // a key read at run time stays in brackets unless Luau wouldn't import the path
                // anyway (see Expr::RuntimeKey)
                let dotted = match key.as_ref() {
                    Expr::String(name) => is_identifier(name),
                    Expr::RuntimeKey(name) => {
                        let root = match table.as_ref() {
                            Expr::Global(g) => Some(g),
                            Expr::Index(t, _) => match t.as_ref() {
                                Expr::Global(g) => Some(g),
                                _ => None,
                            },
                            _ => None,
                        };
                        is_identifier(name) && self.no_imports.as_ref().is_none_or(|w| root.is_some_and(|g| w.contains(g)))
                    }
                    _ => false,
                };
                let table = self.prefix(table, indent);
                match key.as_ref() {
                    Expr::String(name) | Expr::RuntimeKey(name) if dotted => format!("{table}.{}", String::from_utf8_lossy(name)),
                    key => format!("{table}[{}]", self.expr(key, indent)),
                }
            }
            Expr::Call { func, args, .. } => {
                let func = self.prefix(func, indent);
                self.arguments(func, args, indent)
            }
            Expr::MethodCall { object, method, args, multret } if let Some(parts) = interpolation(object, method, args, *multret) => {
                let mut out = String::from("`");
                for (i, part) in parts.iter().enumerate() {
                    interpolated_text(part, &mut out);
                    if let Some(arg) = args.get(i) {
                        let text = self.expr(arg, indent);
                        // `{{` isn't allowed in an interpolated string
                        let text = if text.starts_with('{') { format!("({text})") } else { text };
                        out.push('{');
                        out.push_str(&text);
                        out.push('}');
                    }
                }
                out.push('`');
                out
            }
            Expr::MethodCall { object, method, args, .. } => self.method_call(object, method, args, indent),
            // `0 < x` reads better as `x > 0`, and `a > b` is defined as `b < a`
            Expr::Binary(op @ (BinOp::Lt | BinOp::Le), left, right)
                if matches!(left.as_ref(), Expr::Number(_) | Expr::String(_))
                    && !matches!(right.as_ref(), Expr::Number(_) | Expr::String(_)) =>
            {
                let (p, ..) = binary_precedence(*op);
                let symbol = if *op == BinOp::Lt { ">" } else { ">=" };
                let right = self.operand(right, precedence(right) <= p, indent);
                let left = self.operand(left, precedence(left) <= p, indent);
                format!("{right} {symbol} {left}")
            }
            Expr::Binary(op, left, right) => {
                let (p, right_assoc, symbol) = binary_precedence(*op);
                let (lp, rp) = (precedence(left), precedence(right));
                let compare = p == COMPARE;
                let left = self.operand(left, lp < p || (lp == p && (right_assoc || compare)), indent);
                // `a and (b and c)` is `a and b and c`: same value, same order of evaluation
                let same = matches!(op, BinOp::And | BinOp::Or) && matches!(right.as_ref(), Expr::Binary(o, ..) if o == op);
                let right = self.operand(right, rp < p || (rp == p && (!right_assoc || compare) && !same), indent);
                format!("{left} {symbol} {right}")
            }
            // `a ~= b` is defined as `not (a == b)`, metamethods included (unlike `<`, where
            // NaN makes `not (a < b)` and `a >= b` differ)
            Expr::Unary(UnOp::Not, inner) if matches!(inner.as_ref(), Expr::Binary(BinOp::Eq | BinOp::Ne, ..)) => {
                let Expr::Binary(op, a, b) = inner.as_ref() else { unreachable!() };
                let flipped = if *op == BinOp::Eq { BinOp::Ne } else { BinOp::Eq };
                self.expr(&Expr::Binary(flipped, a.clone(), b.clone()), indent)
            }
            Expr::Unary(op, operand) => {
                // `- -x` would start a comment, so a negated negation keeps its parentheses
                let wrap = precedence(operand) < UNARY || (*op == UnOp::Neg && precedence(operand) == UNARY);
                let operand = self.operand(operand, wrap, indent);
                match op {
                    UnOp::Not => format!("not {operand}"),
                    UnOp::Neg => format!("-{operand}"),
                    UnOp::Len => format!("#{operand}"),
                }
            }
            Expr::Table(fields) => self.table(fields, indent),
            Expr::Closure(closure) => self.closure(closure, indent),
            Expr::IfElse(..) => {
                // `else if` chains read as `elseif`; a long one gets a line per branch
                let mut branches = Vec::new();
                let mut rest = expr;
                while let Expr::IfElse(cond, a, b) = rest {
                    let cond_text = self.operand(cond, precedence(cond) == IF_ELSE, indent + 1);
                    let text = self.operand(a, precedence(a) == IF_ELSE, indent + 1);
                    branches.push((Some(cond_text), a.as_ref(), text));
                    rest = b;
                }
                let text = self.expr(rest, indent + 1);
                branches.push((None, rest, text));
                let heads = branches
                    .iter()
                    .enumerate()
                    .map(|(i, (cond, ..))| match cond {
                        Some(cond) => format!("{} {cond} then", if i == 0 { "if" } else { "elseif" }),
                        None => "else".into(),
                    })
                    .collect::<Vec<_>>();
                let parts = heads.iter().zip(&branches).map(|(head, (.., text))| format!("{head} {text}")).collect::<Vec<_>>();
                // a nested if-expression spread over lines reads best with every branch around it on its own line
                let nested = branches.iter().any(|(_, value, text)| matches!(value, Expr::IfElse(..)) && text.contains('\n'));
                if !nested && fits(&parts.iter().map(String::as_str).collect::<Vec<_>>(), " ", 100) {
                    return parts.join(" ");
                }
                // a value too long to follow its condition goes on a line of its own below it
                let mut lines = Vec::new();
                for (head, (.., text)) in heads.iter().zip(&branches) {
                    if fits(&[head, text], " ", 120) {
                        lines.push(format!("{head} {text}"));
                    } else {
                        lines.push(format!("{head}\n{}{}", "\t".repeat(indent + 2), deeper(text)));
                    }
                }
                lines.join(&format!("\n{}", "\t".repeat(indent + 1)))
            }
            Expr::Tested(_) => unreachable!("value regions never leave a tested placeholder"),
        }
    }

    // A statement's only value that is an `and`/`or` chain; past 120 characters (with the
    // statement's start, `prefix` long) a line per operand of its loosest operator. Each
    // operand is printed once: printing a closure again would name its locals again.
    fn chain(&mut self, value: &Expr, prefix: usize, indent: usize) -> String {
        let Expr::Binary(op, ..) = value else { unreachable!() };
        let mut parts = Vec::new();
        flatten(*op, value, &mut parts);
        let (p, _, symbol) = binary_precedence(*op);
        let texts = parts.iter().map(|part| self.operand(part, precedence(part) <= p, indent + 1)).collect::<Vec<_>>();
        let line = texts.join(&format!(" {symbol} "));
        if prefix + line.len() <= 120 || line.contains('\n') {
            return line;
        }
        texts.join(&format!("\n{}{symbol} ", "\t".repeat(indent + 1)))
    }

    // an `if` or `while` condition; a long `a and b or c` chain gets a line per operand of its
    // loosest operator, between the keyword and `then`/`do`
    fn condition(&mut self, cond: &Expr, indent: usize) -> String {
        let line = self.operand(cond, precedence(cond) == IF_ELSE, indent);
        let Expr::Binary(op @ (BinOp::And | BinOp::Or), ..) = cond else { return line };
        if line.len() <= 100 || line.contains('\n') {
            return line;
        }
        // `(a or b) or c` and `a or (b or c)` both read as `a or b or c`
        let mut parts = Vec::new();
        flatten(*op, cond, &mut parts);
        let (p, _, symbol) = binary_precedence(*op);
        let inner = "\t".repeat(indent + 1);
        let mut out = String::from("\n");
        for (i, part) in parts.iter().enumerate() {
            let text = self.operand(part, precedence(part) <= p, indent + 1);
            out.push_str(&inner);
            if i > 0 {
                out.push_str(symbol);
                out.push(' ');
            }
            out.push_str(&text);
            out.push('\n');
        }
        out.push_str(&"\t".repeat(indent));
        out
    }

    fn method_call(&mut self, object: &Expr, method: &[u8], args: &[Expr], indent: usize) -> String {
        let object = self.prefix(object, indent);
        self.arguments(format!("{object}:{}", String::from_utf8_lossy(method)), args, indent)
    }

    // `f(a, b)`; past 120 characters, with two or more arguments that fit on a line each, one
    // argument per line
    fn arguments(&mut self, func: String, args: &[Expr], indent: usize) -> String {
        let last = args.len().saturating_sub(1);
        let items = args
            .iter()
            .enumerate()
            .map(|(i, a)| if i == last { self.expr_last(a, indent) } else { self.expr(a, indent) })
            .collect::<Vec<_>>();
        let list = items.join(", ");
        let open = format!("{func}(");
        if args.len() < 2 || fits(&[&open, &list, ")"], "", 120) || func.contains('\n') {
            return format!("{func}({list})");
        }
        // printed once: printing a closure again would name its locals again
        let inner = "\t".repeat(indent + 1);
        let lines = items.iter().map(|item| format!("{inner}{}", deeper(item))).collect::<Vec<_>>();
        format!("{func}(\n{}\n{})", lines.join(",\n"), "\t".repeat(indent))
    }

    fn table(&mut self, fields: &[Field], indent: usize) -> String {
        if fields.is_empty() {
            return "{}".into();
        }
        let last = fields.len() - 1;
        let parts = fields
            .iter()
            .enumerate()
            .map(|(i, field)| match field {
                Field::Positional(v) if i == last => self.expr_last(v, indent + 1),
                Field::Positional(v) => self.expr(v, indent + 1),
                Field::Keyed(Expr::String(k), v) if is_identifier(k) => {
                    format!("{} = {}", String::from_utf8_lossy(k), self.expr(v, indent + 1))
                }
                Field::Keyed(k, v) => format!("[{}] = {}", self.expr(k, indent + 1), self.expr(v, indent + 1)),
            })
            .collect::<Vec<_>>();
        let inline = parts.len() <= 4 && parts.iter().all(|p| !p.contains('\n')) && parts.iter().map(String::len).sum::<usize>() < 80;
        if inline {
            format!("{{{}}}", parts.join(", "))
        } else {
            let pad = "\t".repeat(indent + 1);
            let body = parts.iter().map(|p| format!("{pad}{p}")).collect::<Vec<_>>().join(",\n");
            format!("{{\n{body}\n{}}}", "\t".repeat(indent))
        }
    }

    // `a.b.c` for a place a function statement can name
    fn path(&self, e: &Expr) -> Option<String> {
        match e {
            Expr::Local(l) => self.scopes.last().unwrap().locals[*l as usize].clone(),
            Expr::Upvalue(u) => self.scopes.last().unwrap().upvalues.get(*u as usize).cloned(),
            Expr::Global(g) if is_identifier(g) => Some(String::from_utf8_lossy(g).into_owned()),
            Expr::Index(t, k) => match k.as_ref() {
                Expr::String(k) if is_identifier(k) => Some(format!("{}.{}", self.path(t)?, String::from_utf8_lossy(k))),
                _ => None,
            },
            _ => None,
        }
    }

    // `t.m = function(self, ...)` as `function t:m(...)`: when debug info calls the first
    // parameter self, or it's used as an object and the function sits on a class table
    // (`t.__index = t`) or its name is called with `:` somewhere. Not when the body sees an
    // outer `self` (the parameter would hide it) or `self` is a global here.
    fn is_method(&self, closure: &Closure, table: &Expr, name: &[u8]) -> bool {
        let function = &self.functions[closure.proto];
        let Some(&first) = function.params.first() else { return false };
        let named = function.names.get(first as usize).cloned().flatten();
        let likely = match &named {
            Some(n) => n == b"self",
            None => {
                let class = self.path(table).is_some_and(|p| self.class_tables.contains(&p));
                // React calls these on the class, not on an object
                let stat = matches!(name, b"getDerivedStateFromProps" | b"getDerivedStateFromError");
                (class || self.methods.contains(name)) && !stat && used_as_object(self.functions, &function.body, first)
            }
        };
        let parent = self.scopes.last().unwrap();
        let outer_self = closure.captures.iter().any(|c| {
            let n = match c {
                Capture::Local(l) => parent.locals[*l as usize].clone(),
                Capture::Upvalue(u) => parent.upvalues.get(*u as usize).cloned(),
            };
            n.as_deref() == Some("self")
        });
        likely && !outer_self && !self.reserved.contains(&b"self"[..])
    }

    fn closure(&mut self, closure: &Closure, indent: usize) -> String {
        self.closure_as(closure, indent, false)
    }

    // a method's first parameter is `self`, left out of the list
    fn closure_as(&mut self, closure: &Closure, indent: usize, method: bool) -> String {
        let parent = self.scopes.last().unwrap();
        let upvalues = closure
            .captures
            .iter()
            .map(|c| match c {
                Capture::Local(l) => parent.locals[*l as usize].clone().expect("captured locals are declared"),
                Capture::Upvalue(u) => parent.upvalues[*u as usize].clone(),
            })
            .collect();
        let function = &self.functions[closure.proto];
        self.scopes.push(Scope { proto: closure.proto, locals: vec![None; function.names.len()], upvalues });
        let mut params = Vec::new();
        for (i, &p) in function.params.iter().enumerate() {
            if method && i == 0 {
                self.scopes.last_mut().unwrap().locals[p as usize] = Some("self".into());
                continue;
            }
            let name = self.declare_maybe_unused(p, "p");
            params.push(match function.param_types.get(i).cloned().flatten() {
                Some(t) => format!("{name}: {t}"),
                None => name,
            });
        }
        if function.is_vararg {
            params.push("...".into());
        }
        let mut body = String::new();
        self.body(&function.body, indent + 1, &mut body);
        self.scopes.pop();
        format!("function({})\n{body}{}end", params.join(", "), "\t".repeat(indent))
    }
}
