use std::collections::HashSet;

use crate::ast::{BinOp, Capture, Closure, Expr, Field, Function, Stmt, UnOp};

const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local", "nil", "not",
    "or", "repeat", "return", "then", "true", "until", "while", "continue",
];

pub fn is_identifier(name: &[u8]) -> bool {
    let Ok(name) = std::str::from_utf8(name) else { return false };
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !KEYWORDS.contains(&name)
}

// Precedence, loosest first. Atoms and prefix expressions sit above every operator.
const OR: u8 = 1;
const AND: u8 = 2;
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
    }
}

fn precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Binary(op, ..) => binary_precedence(*op).0,
        Expr::Unary(..) => UNARY,
        Expr::Number(n) if n.is_finite() && n.is_sign_negative() => UNARY,
        Expr::Number(n) if n.is_nan() && n.is_sign_positive() => UNARY,
        _ => ATOM,
    }
}

fn is_prefix(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Local(_) | Expr::Upvalue(_) | Expr::Global(_) | Expr::Index(..) | Expr::Call { .. } | Expr::MethodCall { .. }
    )
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
    let mut rest = bytes;
    while !rest.is_empty() {
        let (valid, bad) = match std::str::from_utf8(rest) {
            Ok(s) => (s, &[][..]),
            Err(e) => (std::str::from_utf8(&rest[..e.valid_up_to()]).unwrap(), &rest[e.valid_up_to()..]),
        };
        for c in valid.chars() {
            match c {
                '"' => out.push_str("\\\""),
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
    out.push('"');
    out
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
}

pub fn print_chunk(functions: &[Function], main: usize) -> String {
    let mut reserved = HashSet::new();
    for function in functions {
        collect_globals(&function.body, &mut reserved);
    }
    let mut printer = Printer { functions, reserved, taken: HashSet::new(), counter: 0, scopes: Vec::new() };
    printer.scopes.push(Scope { proto: main, locals: vec![None; functions[main].names.len()], upvalues: Vec::new() });
    let mut out = String::new();
    printer.body(&functions[main].body, 0, &mut out);
    out
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
        }
    }
}

impl Printer<'_> {
    fn declare(&mut self, local: u32) -> String {
        let proto = self.scopes.last().unwrap().proto;
        let wanted = self.functions[proto].names.get(local as usize).cloned().flatten().filter(|n| is_identifier(n));
        let name = match wanted {
            Some(n) if !self.reserved.contains(&n) && !self.taken.contains(std::str::from_utf8(&n).unwrap()) => {
                String::from_utf8(n).unwrap()
            }
            wanted => {
                let base = wanted.map(|n| String::from_utf8(n).unwrap()).unwrap_or_else(|| "v".into());
                loop {
                    self.counter += 1;
                    let candidate = format!("{base}{}", self.counter);
                    if !self.taken.contains(&candidate) && !self.reserved.contains(candidate.as_bytes()) {
                        break candidate;
                    }
                }
            }
        };
        self.taken.insert(name.clone());
        self.scopes.last_mut().unwrap().locals[local as usize] = Some(name.clone());
        name
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

    fn stmt(&mut self, stmt: &Stmt, indent: usize) -> String {
        match stmt {
            Stmt::Local { locals, values } => {
                let values = self.list_for(values, locals.len(), indent);
                let names = locals.iter().map(|&l| self.declare(l)).collect::<Vec<_>>();
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
            Stmt::Assign { targets, values } => {
                let targets = targets.iter().map(|t| self.expr(t, indent)).collect::<Vec<_>>();
                let values = self.list_for(values, targets.len(), indent);
                format!("{} = {values}", targets.join(", "))
            }
            Stmt::Call(call) => self.expr(call, indent),
            Stmt::Return(values) => {
                let values = self.list(values, indent);
                if values.is_empty() { "return".into() } else { format!("return {values}") }
            }
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
        );
        let text = self.expr(expr, indent);
        if truncated { format!("({text})") } else { text }
    }

    fn operand(&mut self, expr: &Expr, wrap: bool, indent: usize) -> String {
        let text = self.expr(expr, indent);
        if wrap { format!("({text})") } else { text }
    }

    fn prefix(&mut self, expr: &Expr, indent: usize) -> String {
        let wrap = !is_prefix(expr);
        self.operand(expr, wrap, indent)
    }

    fn expr(&mut self, expr: &Expr, indent: usize) -> String {
        match expr {
            Expr::Nil => "nil".into(),
            Expr::Bool(b) => b.to_string(),
            Expr::Number(n) => number(*n),
            Expr::String(s) => string(s),
            Expr::Vector(v) => format!("Vector3.new({}, {}, {})", number(v[0]), number(v[1]), number(v[2])),
            Expr::VarArg { .. } => "...".into(),
            Expr::Local(l) => self.local(*l),
            Expr::Upvalue(u) => self.scopes.last().unwrap().upvalues[*u as usize].clone(),
            Expr::Global(name) => String::from_utf8_lossy(name).into_owned(),
            Expr::Index(table, key) => {
                let table = self.prefix(table, indent);
                match key.as_ref() {
                    Expr::String(name) if is_identifier(name) => format!("{table}.{}", String::from_utf8_lossy(name)),
                    key => format!("{table}[{}]", self.expr(key, indent)),
                }
            }
            Expr::Call { func, args, .. } => {
                let func = self.prefix(func, indent);
                format!("{func}({})", self.list(args, indent))
            }
            Expr::MethodCall { object, method, args, .. } => {
                let object = self.prefix(object, indent);
                format!("{object}:{}({})", String::from_utf8_lossy(method), self.list(args, indent))
            }
            Expr::Binary(op, left, right) => {
                let (p, right_assoc, symbol) = binary_precedence(*op);
                let (lp, rp) = (precedence(left), precedence(right));
                let left = self.operand(left, lp < p || (lp == p && right_assoc), indent);
                let right = self.operand(right, rp < p || (rp == p && !right_assoc), indent);
                format!("{left} {symbol} {right}")
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
        }
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

    fn closure(&mut self, closure: &Closure, indent: usize) -> String {
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
        let mut params = function.params.iter().map(|&p| self.declare(p)).collect::<Vec<_>>();
        if function.is_vararg {
            params.push("...".into());
        }
        let mut body = String::new();
        self.body(&function.body, indent + 1, &mut body);
        self.scopes.pop();
        format!("function({})\n{body}{}end", params.join(", "), "\t".repeat(indent))
    }
}
