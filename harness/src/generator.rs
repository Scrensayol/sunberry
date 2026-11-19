// Seeded generator of terminating, type-correct Luau programs. Every value a program
// computes reaches `out`, which prints it in a form that doesn't depend on table layout
// or heap addresses, so two faithful compilations of a program print the same lines.

use std::rc::Rc;

use crate::rng::Rng;

pub const GENERATOR_VERSION: &str = "typed-v2";

// Installed in the VM before every program as the global `out`, so it's no part of what
// gets decompiled.
pub const PRELUDE: &str = r##"local function describe(v, depth)
	local kind = type(v)
	if kind == "number" then
		if v ~= v then
			return "nan"
		elseif v == 0 then
			return if 1 / v < 0 then "-0" else "0"
		end
		return tostring(v)
	elseif kind == "string" then
		return string.format("%q", v)
	elseif kind == "table" then
		if depth > 2 then
			return "{...}"
		end
		local parts = {}
		for key, value in next, v do
			table.insert(parts, describe(key, depth + 1) .. "=" .. describe(value, depth + 1))
		end
		table.sort(parts)
		return "{" .. table.concat(parts, ",") .. "}"
	elseif kind == "function" then
		return "fn"
	end
	return tostring(v)
end
function out(label, ...)
	local count = select("#", ...)
	local parts = table.create(count)
	for index = 1, count do
		parts[index] = describe((select(index, ...)), 0)
	end
	print(label, count, table.concat(parts, " "))
end
"##;

#[derive(Clone, Debug, PartialEq)]
enum Ty {
    Num,
    Str,
    Bool,
    Any,
    Rec(Rc<Vec<(Key, Ty)>>),
    Arr(Rc<Ty>, usize),
    Fun(Rc<Sig>),
}

#[derive(Clone, Debug, PartialEq)]
enum Key {
    Name(&'static str),
    Text(&'static str),
    Num(&'static str),
}

impl Key {
    fn access(&self) -> String {
        match self {
            Key::Name(n) => format!(".{n}"),
            Key::Text(t) => format!("[\"{t}\"]"),
            Key::Num(n) => format!("[{n}]"),
        }
    }

    fn field(&self) -> String {
        match self {
            Key::Name(n) => n.to_string(),
            Key::Text(t) => format!("[\"{t}\"]"),
            Key::Num(n) => format!("[{n}]"),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Sig {
    params: Vec<Ty>,
    rets: Vec<Ty>,
    vararg: bool,
}

#[derive(Clone)]
struct Var {
    id: usize,
    name: String,
    ty: Ty,
    mutable: bool,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub text: String,
    pub body: Vec<Node>,
    pub close: Option<String>,
}

impl Node {
    fn leaf(text: impl Into<String>) -> Node {
        Node { text: text.into(), body: Vec::new(), close: None }
    }

    fn block(text: impl Into<String>, body: Vec<Node>, close: impl Into<String>) -> Node {
        Node { text: text.into(), body, close: Some(close.into()) }
    }

    fn open(text: impl Into<String>, body: Vec<Node>) -> Node {
        Node { text: text.into(), body, close: None }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub units: Vec<Node>,
}

impl Program {
    pub fn render(&self) -> String {
        let mut out = String::new();
        for unit in &self.units {
            render_node(unit, 0, &mut out);
        }
        out
    }
}

fn render_node(node: &Node, indent: usize, out: &mut String) {
    let pad = "\t".repeat(indent);
    let mut line = |text: &str| {
        out.push_str(&pad);
        out.push_str(text);
        out.push('\n');
    };
    // only the first line is indented: later lines may be inside a long string literal
    if !node.text.is_empty() {
        let mut lines = node.text.splitn(2, '\n');
        line(lines.next().unwrap());
        if let Some(rest) = lines.next() {
            out.push_str(rest);
            out.push('\n');
        }
    }
    for child in &node.body {
        render_node(child, indent + 1, out);
    }
    if let Some(close) = &node.close {
        let pad = "\t".repeat(indent);
        out.push_str(&pad);
        out.push_str(close);
        out.push('\n');
    }
}

const NUMS: &[&str] = &[
    "0", "1", "2", "3", "7", "-1", "-5", "0.5", "-2.5", "10", "100", "0.1", "1e10", "1e-7", "(2 ^ 53)",
    "9007199254740993", "0.30000000000000004", "123456789.125", "5e-324", "1.7976931348623157e308", "(-0)",
    "(1 / 0)", "(-1 / 0)", "(0 / 0)", "math.huge", "math.pi", "255", "65536", "4294967296", "1e300", "3.5",
    "0x7F", "1_000", "2.5e-3",
];
const SMALL_NUMS: &[&str] = &["0", "1", "2", "3", "-1", "-2", "0.5", "4", "10", "-0.5"];
const STRS: &[&str] = &[
    "\"\"", "\"a\"", "\"hello\"", "\"x y\"", "\"\\n\"", "\"tab\\there\"", "\"quote\\\"s\"", "\"back\\\\slash\"",
    "\"\\0nul\"", "\"é\"", "\"\\u{1F600}\"", "\"[[\"", "\"]]\"", "\"--not a comment\"", "\"%d%%\"",
    "\"{braces}\"", "'single'", "[[long\nstring]]", "[==[a]]b]==]", "\"\\x41\\65\"", "\"10\"", "\"-3.5\"",
    "\"0x1F\"", "\"  spaced  \"", "\"\\r\\t\\v\\f\\a\\b\"", "\"\\255\\128\"", "\"end\"", "\"nil\"",
];
const FIELDS: &[Key] = &[
    Key::Name("x"),
    Key::Name("y"),
    Key::Name("name"),
    Key::Name("count"),
    Key::Name("_hidden"),
    Key::Text("two words"),
    Key::Text("end"),
    Key::Text("nil"),
    Key::Text("1abc"),
    Key::Text(""),
    Key::Num("1"),
    Key::Num("2"),
    Key::Num("1.5"),
    Key::Num("-1"),
    Key::Num("1e300"),
];

#[derive(Clone, Copy, PartialEq)]
enum LoopKind {
    Plain,
    Repeat,
}

struct FnCtx {
    loops: Vec<LoopKind>,
    rets: Option<Vec<Ty>>,
    vararg: bool,
}

struct Gen {
    rng: Rng,
    scopes: Vec<Vec<Var>>,
    fns: Vec<FnCtx>,
    next_id: usize,
    label: usize,
    stmts_left: usize,
    // arrays being walked by a loop: appending to one would never end
    walking: Vec<usize>,
    call_depth: u32,
    straight: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Grammar {
    Full,
    // no jumps at all: no branches, loops, comparisons, and/or or if-expressions
    Straight,
}

pub fn generate(seed: u64, grammar: Grammar) -> Program {
    let mut g = Gen {
        rng: Rng::new(seed),
        scopes: vec![Vec::new()],
        fns: vec![FnCtx { loops: Vec::new(), rets: None, vararg: true }],
        next_id: 0,
        label: 0,
        stmts_left: 0,
        walking: Vec::new(),
        call_depth: 0,
        straight: grammar == Grammar::Straight,
    };
    let units = (0..g.rng.range(4, 8)).map(|_| g.unit()).collect();
    Program { units }
}

impl Gen {
    fn unit(&mut self) -> Node {
        self.stmts_left = self.rng.range(6, 20) as usize;
        self.scopes.push(Vec::new());
        let mut body = Vec::new();
        if self.rng.chance(45) {
            self.family(&mut body);
        }
        while self.stmts_left > 0 {
            self.stmt(3, &mut body);
        }
        body.push(self.observe_all());
        self.scopes.pop();
        Node::block("do", body, "end")
    }

    fn fresh(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }

    fn label(&mut self) -> String {
        self.label += 1;
        format!("\"L{}\"", self.label)
    }

    fn declare(&mut self, name: String, ty: Ty, mutable: bool) {
        self.next_id += 1;
        let id = self.next_id;
        self.scopes.last_mut().unwrap().push(Var { id, name, ty, mutable });
    }

    fn ctx(&mut self) -> &mut FnCtx {
        self.fns.last_mut().unwrap()
    }

    fn visible(&self) -> Vec<Var> {
        let mut seen = Vec::<String>::new();
        let mut vars = Vec::new();
        for scope in self.scopes.iter().rev() {
            for var in scope.iter().rev() {
                if !seen.contains(&var.name) {
                    seen.push(var.name.clone());
                    vars.push(var.clone());
                }
            }
        }
        vars
    }

    fn pick_var(&mut self, test: impl Fn(&Var) -> bool) -> Option<Var> {
        let vars = self.visible().into_iter().filter(|v| test(v)).collect::<Vec<_>>();
        (!vars.is_empty()).then(|| vars[self.rng.below(vars.len())].clone())
    }

    fn scalar_ty(&mut self) -> Ty {
        match self.rng.below(100) {
            0..=39 => Ty::Num,
            40..=64 => Ty::Str,
            65..=79 => Ty::Bool,
            _ => Ty::Any,
        }
    }

    // ---- expressions ----

    fn expr(&mut self, ty: &Ty, d: u32) -> String {
        match ty {
            Ty::Num => self.num(d),
            Ty::Str => self.string(d),
            Ty::Bool => self.boolean(d),
            _ => self.any(d),
        }
    }

    fn read(&mut self, ty: &Ty) -> Option<String> {
        match self.rng.below(4) {
            0 => {
                let var = self.pick_var(|v| match &v.ty {
                    Ty::Rec(fields) => fields.iter().any(|(_, t)| t == ty),
                    _ => false,
                })?;
                let Ty::Rec(fields) = &var.ty else { unreachable!() };
                let matching = fields.iter().filter(|(_, t)| t == ty).collect::<Vec<_>>();
                let (key, _) = matching[self.rng.below(matching.len())];
                Some(format!("{}{}", var.name, key.access()))
            }
            1 => {
                let var = self.pick_var(|v| matches!(&v.ty, Ty::Arr(e, n) if **e == *ty && *n > 0))?;
                let Ty::Arr(_, len) = var.ty else { unreachable!() };
                let index = if self.rng.chance(20) { format!("#{}", var.name) } else { self.rng.range(1, len as i64).to_string() };
                Some(format!("{}[{index}]", var.name))
            }
            2 if self.rng.chance(50) => self.call_returning(ty),
            _ => self.pick_var(|v| v.ty == *ty).map(|v| v.name),
        }
    }

    fn call_returning(&mut self, ty: &Ty) -> Option<String> {
        if self.call_depth >= 2 {
            return None;
        }
        let var = self.pick_var(|v| matches!(&v.ty, Ty::Fun(s) if s.rets.first() == Some(ty)))?;
        let Ty::Fun(sig) = &var.ty else { unreachable!() };
        self.call_depth += 1;
        let call = format!("{}({})", var.name, self.args(sig, 1));
        self.call_depth -= 1;
        Some(if sig.rets.len() > 1 { format!("({call})") } else { call })
    }

    fn args(&mut self, sig: &Sig, d: u32) -> String {
        let mut args = sig.params.iter().map(|p| self.expr(p, d)).collect::<Vec<_>>();
        if sig.vararg {
            for _ in 0..self.rng.below(4) {
                let ty = self.scalar_ty();
                args.push(self.expr(&ty, d));
            }
        }
        args.join(", ")
    }

    fn num(&mut self, d: u32) -> String {
        if d == 0 || self.rng.chance(30) {
            if self.rng.chance(55)
                && let Some(read) = self.read(&Ty::Num)
            {
                return read;
            }
            return self.rng.pick(NUMS).to_string();
        }
        let d = d - 1;
        match self.rng.below(16) {
            0..=4 => {
                let op = *self.rng.pick(&["+", "-", "*", "/", "//", "%", "+", "-", "*"]);
                format!("({} {op} {})", self.num(d), self.num(d))
            }
            5 => format!("({} ^ {})", self.num(d), self.rng.pick(&["2", "3", "-1", "0", "0.25"])),
            6 => format!("(- {})", self.num(d)),
            7 => format!("math.{}({})", self.rng.pick(&["floor", "abs", "ceil", "sign", "round"]), self.num(d)),
            8 => format!("math.{}({}, {})", self.rng.pick(&["max", "min"]), self.num(d), self.num(d)),
            9 => format!("(#{})", self.string_operand(d)),
            10 if !self.straight => format!("(if {} then {} else {})", self.boolean(d), self.num(d), self.num(d)),
            11 => format!("math.clamp({}, -4, 4)", self.num(d)),
            12 if self.fns.last().unwrap().vararg => "select(\"#\", ...)".to_string(),
            13 => format!("tonumber({})", self.rng.pick(&["\"10\"", "\"0x1F\"", "\"-3.5\"", "\"1e3\"", "\" 7 \""])),
            14 => format!("string.len({})", self.string(d)),
            _ => self.read(&Ty::Num).unwrap_or_else(|| self.rng.pick(NUMS).to_string()),
        }
    }

    // a string expression usable right after `#` or before `:`
    fn string_operand(&mut self, d: u32) -> String {
        let s = self.string(d);
        if s.starts_with('(') || s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            s
        } else {
            format!("({s})")
        }
    }

    fn string(&mut self, d: u32) -> String {
        if d == 0 || self.rng.chance(30) {
            if self.rng.chance(55)
                && let Some(read) = self.read(&Ty::Str)
            {
                return read;
            }
            return self.rng.pick(STRS).to_string();
        }
        let d = d - 1;
        match self.rng.below(13) {
            0..=2 => format!("({} .. {})", self.string(d), self.string(d)),
            3 => format!("({} .. {})", self.string(d), self.num(d)),
            4 => format!("tostring({})", self.num(d)),
            5 => format!("tostring({})", self.boolean(d)),
            6 => format!("{}:{}()", self.string_operand(d), self.rng.pick(&["upper", "lower", "reverse"])),
            7 => format!("string.rep({}, {})", self.string(d), self.rng.range(0, 3)),
            8 => format!("{}:sub({}, {})", self.string_operand(d), self.rng.range(-3, 3), self.rng.range(-2, 4)),
            9 => format!("string.format(\"%s|%g\", {}, {})", self.string(d), self.num(d)),
            10 => {
                let parts = ["", "v=", " | ", "\\{x}", "end"];
                let (a, b) = (self.any(d), self.num(d));
                format!("`{}{{{a}}}{}{{{b}}}`", self.rng.pick(&parts), self.rng.pick(&parts))
            }
            11 if !self.straight => format!("(if {} then {} else {})", self.boolean(d), self.string(d), self.string(d)),
            _ => self.read(&Ty::Str).unwrap_or_else(|| self.rng.pick(STRS).to_string()),
        }
    }

    fn boolean(&mut self, d: u32) -> String {
        if self.straight && d > 0 {
            return match self.rng.below(3) {
                0 => format!("not {}", self.boolean(d - 1)),
                1 => format!("rawequal({}, {})", self.any(d - 1), self.any(d - 1)),
                _ => self.boolean(0),
            };
        }
        if d == 0 || self.rng.chance(20) {
            if self.rng.chance(50)
                && let Some(read) = self.read(&Ty::Bool)
            {
                return read;
            }
            return self.rng.pick(&["true", "false"]).to_string();
        }
        let d = d - 1;
        match self.rng.below(12) {
            0..=3 => {
                let op = *self.rng.pick(&["<", "<=", ">", ">=", "==", "~="]);
                format!("({} {op} {})", self.num(d), self.num(d))
            }
            4 => {
                let op = *self.rng.pick(&["<", "<=", ">", ">=", "==", "~="]);
                format!("not ({} {op} {})", self.num(d), self.num(d))
            }
            5 => format!("({} {} {})", self.string(d), self.rng.pick(&["<", "==", "~=", ">="]), self.string(d)),
            6 => format!("not {}", self.boolean(d)),
            7 => format!("({} and {})", self.boolean(d), self.boolean(d)),
            8 => format!("({} or {})", self.boolean(d), self.boolean(d)),
            9 => format!("({} == {})", self.any(d), self.any(d)),
            10 => format!("(type({}) == \"{}\")", self.any(d), self.rng.pick(&["number", "string", "nil", "boolean"])),
            _ => format!("rawequal({}, {})", self.any(d), self.any(d)),
        }
    }

    fn any(&mut self, d: u32) -> String {
        if self.straight && d > 0 {
            return match self.rng.below(3) {
                0 => format!("not {}", self.any(d - 1)),
                1 => {
                    let ty = self.scalar_ty();
                    self.expr(&ty, d - 1)
                }
                _ => self.any(0),
            };
        }
        if d == 0 || self.rng.chance(25) {
            if self.rng.chance(40)
                && let Some(read) = self.read(&Ty::Any)
            {
                return read;
            }
            return self.rng.pick(&["nil", "false", "true", "0", "\"s\"", "(0 / 0)"]).to_string();
        }
        let d = d - 1;
        match self.rng.below(8) {
            0 | 1 => format!("({} and {})", self.any(d), self.any(d)),
            2 | 3 => format!("({} or {})", self.any(d), self.any(d)),
            4 => format!("({} and {} or {})", self.boolean(d), self.any(d), self.any(d)),
            5 => {
                let ty = self.scalar_ty();
                self.expr(&ty, d)
            }
            6 => format!("(if {} then {} else {})", self.any(d), self.any(d), self.any(d)),
            _ => format!("not {}", self.any(d)),
        }
    }

    fn cond(&mut self, d: u32) -> String {
        if self.rng.chance(70) { self.boolean(d) } else { self.any(d) }
    }

    // ---- statements ----

    fn block(&mut self, depth: u32, max: usize) -> Vec<Node> {
        self.scopes.push(Vec::new());
        let body = self.stmts(depth, max);
        self.scopes.pop();
        body
    }

    // statements in the current scope, so whatever follows them in the same block (an
    // `until`, a final `return`) sees their locals the way Luau resolves names
    fn stmts(&mut self, depth: u32, max: usize) -> Vec<Node> {
        let mut body = Vec::new();
        let count = self.rng.range(1, max as i64) as usize;
        for _ in 0..count {
            if self.stmts_left == 0 {
                break;
            }
            self.stmt(depth, &mut body);
        }
        body
    }

    fn stmt(&mut self, depth: u32, out: &mut Vec<Node>) {
        self.stmts_left = self.stmts_left.saturating_sub(1);
        let in_loop = !self.fns.last().unwrap().loops.is_empty();
        let in_fn = self.fns.last().unwrap().rets.is_some();
        if self.straight {
            return match self.rng.below(100) {
                0..=27 => self.local_stmt(out),
                28..=43 => self.assign_stmt(out),
                44..=63 => {
                    let o = self.out_stmt();
                    out.push(o)
                }
                64..=75 if depth > 0 => self.function_stmt(depth, out),
                76..=79 if depth > 0 => {
                    let body = self.block(depth - 1, 4);
                    out.push(Node::block("do", body, "end"))
                }
                80..=87 => self.multi_assign(out),
                88..=97 => self.call_stmt(out),
                _ => self.local_stmt(out),
            };
        }
        match self.rng.below(100) {
            0..=21 => self.local_stmt(out),
            22..=35 => self.assign_stmt(out),
            36..=49 => {
                let o = self.out_stmt();
                out.push(o)
            }
            50..=57 if depth > 0 => self.if_stmt(depth, out),
            58..=62 if depth > 0 => self.numeric_for(depth, out),
            63..=65 if depth > 0 => self.while_stmt(depth, out),
            66..=67 if depth > 0 => self.repeat_stmt(depth, out),
            68..=71 if depth > 0 => self.for_in(depth, out),
            72..=78 if depth > 0 => self.function_stmt(depth, out),
            79..=80 if depth > 0 => {
                let body = self.block(depth - 1, 4);
                out.push(Node::block("do", body, "end"))
            }
            81..=85 if in_loop => self.loop_exit(out),
            86..=88 if in_fn => self.early_return(out),
            89..=92 => self.multi_assign(out),
            93..=97 => self.call_stmt(out),
            _ => self.local_stmt(out),
        }
    }

    fn observe_all(&mut self) -> Node {
        let vars = self.visible().into_iter().filter(|v| !matches!(v.ty, Ty::Fun(_))).take(6).collect::<Vec<_>>();
        let label = self.label();
        if vars.is_empty() {
            return Node::leaf(format!("out({label})"));
        }
        let names = vars.iter().map(|v| v.name.as_str()).collect::<Vec<_>>().join(", ");
        Node::leaf(format!("out({label}, {names})"))
    }

    fn out_stmt(&mut self) -> Node {
        let label = self.label();
        let count = self.rng.range(1, 4);
        let mut exprs = Vec::new();
        for _ in 0..count {
            if self.rng.chance(35)
                && let Some(var) = self.pick_var(|v| !matches!(v.ty, Ty::Fun(_)))
            {
                exprs.push(var.name);
                continue;
            }
            let ty = self.scalar_ty();
            exprs.push(self.expr(&ty, 3));
        }
        if self.rng.chance(15)
            && let Some(var) = self.pick_var(|v| matches!(&v.ty, Ty::Fun(s) if s.rets.len() > 1))
        {
            let Ty::Fun(sig) = &var.ty else { unreachable!() };
            exprs.push(format!("{}({})", var.name, self.args(sig, 1)));
        }
        Node::leaf(format!("out({label}, {})", exprs.join(", ")))
    }

    fn new_name(&mut self, prefix: &str) -> String {
        if self.rng.chance(12)
            && let Some(var) = self.pick_var(|v| v.mutable)
        {
            return var.name;
        }
        self.fresh(prefix)
    }

    fn local_stmt(&mut self, out: &mut Vec<Node>) {
        match self.rng.below(100) {
            0..=54 => {
                let ty = self.scalar_ty();
                let value = self.expr(&ty, 3);
                let name = self.new_name(match ty {
                    Ty::Num => "n",
                    Ty::Str => "s",
                    Ty::Bool => "b",
                    _ => "a",
                });
                out.push(Node::leaf(format!("local {name} = {value}")));
                self.declare(name, ty, true);
            }
            55..=62 => {
                let name = self.new_name("a");
                out.push(Node::leaf(format!("local {name}")));
                self.declare(name, Ty::Any, true);
            }
            63..=74 => {
                let mut fields = Vec::new();
                let mut entries = Vec::new();
                for _ in 0..self.rng.range(1, 4) {
                    let key = self.rng.pick(FIELDS).clone();
                    if fields.iter().any(|(k, _)| *k == key) {
                        continue;
                    }
                    let ty = match self.scalar_ty() {
                        Ty::Any => Ty::Num,
                        ty => ty,
                    };
                    entries.push(format!("{} = {}", key.field(), self.expr(&ty, 2)));
                    fields.push((key, ty));
                }
                let name = self.fresh("r");
                out.push(Node::leaf(format!("local {name} = {{{}}}", entries.join(", "))));
                self.declare(name, Ty::Rec(Rc::new(fields)), false);
            }
            75..=84 => {
                let elem = if self.rng.chance(60) { Ty::Num } else { Ty::Str };
                let len = self.rng.range(0, 5) as usize;
                let items = (0..len).map(|_| self.expr(&elem, 2)).collect::<Vec<_>>();
                let name = self.fresh("t");
                out.push(Node::leaf(format!("local {name} = {{{}}}", items.join(", "))));
                self.declare(name, Ty::Arr(Rc::new(elem), len), false);
            }
            85..=93 => {
                let Some(var) = self.pick_var(|v| matches!(&v.ty, Ty::Fun(s) if s.rets.len() > 1)) else {
                    return self.local_stmt(out);
                };
                let Ty::Fun(sig) = &var.ty else { unreachable!() };
                let take = self.rng.range(1, sig.rets.len() as i64 + 1) as usize;
                let names = (0..take).map(|_| self.fresh("m")).collect::<Vec<_>>();
                out.push(Node::leaf(format!("local {} = {}({})", names.join(", "), var.name, self.args(sig, 1))));
                for (i, name) in names.into_iter().enumerate() {
                    let ty = sig.rets.get(i).cloned().unwrap_or(Ty::Any);
                    self.declare(name, ty, true);
                }
            }
            _ => {
                let count = self.rng.range(2, 3) as usize;
                let values = self.rng.range(1, count as i64) as usize;
                let tys = (0..values).map(|_| self.scalar_ty()).collect::<Vec<_>>();
                let exprs = tys.iter().map(|t| self.expr(t, 2)).collect::<Vec<_>>();
                let names = (0..count).map(|_| self.fresh("v")).collect::<Vec<_>>();
                out.push(Node::leaf(format!("local {} = {}", names.join(", "), exprs.join(", "))));
                for (i, name) in names.into_iter().enumerate() {
                    self.declare(name, tys.get(i).cloned().unwrap_or(Ty::Any), true);
                }
            }
        }
    }

    // `name = value`, `name op= value`, or a field/element store
    fn target(&mut self) -> Option<(String, Ty)> {
        match self.rng.below(5) {
            0 => {
                let var = self.pick_var(|v| matches!(v.ty, Ty::Rec(_)))?;
                let Ty::Rec(fields) = &var.ty else { unreachable!() };
                let (key, ty) = fields[self.rng.below(fields.len())].clone();
                Some((format!("{}{}", var.name, key.access()), ty))
            }
            1 => {
                let var = self.pick_var(|v| matches!(&v.ty, Ty::Arr(_, n) if *n > 0))?;
                let Ty::Arr(elem, len) = &var.ty else { unreachable!() };
                Some((format!("{}[{}]", var.name, self.rng.range(1, *len as i64)), (**elem).clone()))
            }
            _ => {
                let var = self.pick_var(|v| v.mutable && matches!(v.ty, Ty::Num | Ty::Str | Ty::Bool | Ty::Any))?;
                Some((var.name, var.ty))
            }
        }
    }

    fn assign_stmt(&mut self, out: &mut Vec<Node>) {
        let Some((target, ty)) = self.target() else {
            return self.local_stmt(out);
        };
        let text = match ty {
            Ty::Num if self.rng.chance(40) => {
                let op = self.rng.pick(&["+=", "-=", "*=", "/=", "//=", "%=", "^="]);
                let value = if *op == "^=" { self.rng.pick(&["2", "0.5", "-1"]).to_string() } else { self.num(2) };
                format!("{target} {op} {value}")
            }
            Ty::Str if self.rng.chance(40) => format!("{target} ..= {}", self.string(2)),
            ty => format!("{target} = {}", self.expr(&ty, 3)),
        };
        out.push(Node::leaf(text));
    }

    fn multi_assign(&mut self, out: &mut Vec<Node>) {
        let (Some((a, ta)), Some((b, tb))) = (self.target(), self.target()) else {
            return self.assign_stmt(out);
        };
        if a == b {
            return self.assign_stmt(out);
        }
        if ta == tb && self.rng.chance(50) {
            out.push(Node::leaf(format!("{a}, {b} = {b}, {a}")));
        } else {
            out.push(Node::leaf(format!("{a}, {b} = {}, {}", self.expr(&ta, 2), self.expr(&tb, 2))));
        }
    }

    fn call_stmt(&mut self, out: &mut Vec<Node>) {
        let walking = self.walking.clone();
        if self.rng.chance(30)
            && let Some(var) = self.pick_var(|v| matches!(v.ty, Ty::Arr(_, _)) && !walking.contains(&v.id))
        {
            let Ty::Arr(elem, _) = &var.ty else { unreachable!() };
            let value = self.expr(&elem.clone(), 2);
            out.push(Node::leaf(format!("table.insert({}, {value})", var.name)));
            return;
        }
        let Some(var) = self.pick_var(|v| matches!(v.ty, Ty::Fun(_))) else {
            return self.local_stmt(out);
        };
        let Ty::Fun(sig) = &var.ty else { unreachable!() };
        out.push(Node::leaf(format!("{}({})", var.name, self.args(sig, 2))));
    }

    fn if_stmt(&mut self, depth: u32, out: &mut Vec<Node>) {
        let cond = self.cond(3);
        let body = self.block(depth - 1, 4);
        out.push(Node::open(format!("if {cond} then"), body));
        for _ in 0..self.rng.below(3) {
            let cond = self.cond(3);
            let body = self.block(depth - 1, 3);
            out.push(Node::open(format!("elseif {cond} then"), body));
        }
        if self.rng.chance(50) {
            let body = self.block(depth - 1, 3);
            out.push(Node::open("else", body));
        }
        out.push(Node::leaf("end"));
    }

    fn loop_body(&mut self, kind: LoopKind, depth: u32, vars: Vec<(String, Ty)>, prefix: Vec<Node>) -> Vec<Node> {
        self.ctx().loops.push(kind);
        self.scopes.push(Vec::new());
        for (name, ty) in vars {
            self.declare(name, ty, false);
        }
        let mut body = prefix;
        body.extend(self.block(depth - 1, 4));
        self.scopes.pop();
        self.ctx().loops.pop();
        body
    }

    fn numeric_for(&mut self, depth: u32, out: &mut Vec<Node>) {
        let name = self.fresh("i");
        let header = match self.rng.below(6) {
            0 => format!("{name} = 1, math.min(#{}, 4)", self.string_operand(1)),
            1 => format!("{name} = 0, 1, 0.25"),
            2 => format!("{name} = {}, {}, -1", self.rng.range(1, 4), self.rng.range(-1, 1)),
            3 => format!("{name} = 3, 1"),
            _ => {
                let start = self.rng.range(-2, 3);
                format!("{name} = {start}, {}", start + self.rng.range(0, 3))
            }
        };
        let body = self.loop_body(LoopKind::Plain, depth, vec![(name, Ty::Num)], Vec::new());
        out.push(Node::block(format!("for {header} do"), body, "end"));
    }

    fn while_stmt(&mut self, depth: u32, out: &mut Vec<Node>) {
        let counter = self.fresh("w");
        out.push(Node::leaf(format!("local {counter} = 0")));
        let limit = self.rng.range(1, 4);
        let cond = if self.rng.chance(40) {
            format!("{counter} < {limit} and {}", self.cond(2))
        } else {
            format!("{counter} < {limit}")
        };
        let body = self.loop_body(LoopKind::Plain, depth, Vec::new(), vec![Node::leaf(format!("{counter} += 1"))]);
        out.push(Node::block(format!("while {cond} do"), body, "end"));
        self.declare(counter, Ty::Num, false);
    }

    fn repeat_stmt(&mut self, depth: u32, out: &mut Vec<Node>) {
        let counter = self.fresh("r");
        out.push(Node::leaf(format!("local {counter} = 0")));
        let limit = self.rng.range(1, 4);
        self.ctx().loops.push(LoopKind::Repeat);
        self.scopes.push(Vec::new());
        let mut body = vec![Node::leaf(format!("{counter} += 1"))];
        // a local declared in the body is visible to `until`
        if self.rng.chance(50) {
            let name = self.fresh("u");
            body.push(Node::leaf(format!("local {name} = {}", self.num(2))));
            self.declare(name, Ty::Num, false);
        }
        body.extend(self.stmts(depth - 1, 3));
        let until = if self.rng.chance(50) {
            format!("until {counter} >= {limit} or {}", self.cond(2))
        } else {
            format!("until {counter} >= {limit}")
        };
        self.scopes.pop();
        self.ctx().loops.pop();
        out.push(Node::block("repeat", body, until));
        self.declare(counter, Ty::Num, false);
    }

    fn for_in(&mut self, depth: u32, out: &mut Vec<Node>) {
        let roll = self.rng.below(6);
        if roll < 3
            && let Some(arr) = self.pick_var(|v| matches!(v.ty, Ty::Arr(_, _)))
        {
            let Ty::Arr(elem, _) = &arr.ty else { unreachable!() };
            let (i, v) = (self.fresh("k"), self.fresh("e"));
            let source = match roll {
                0 => format!("ipairs({})", arr.name),
                1 => arr.name.clone(),
                _ => format!("next, {}", arr.name),
            };
            self.walking.push(arr.id);
            let body = self.loop_body(LoopKind::Plain, depth, vec![(i.clone(), Ty::Num), (v.clone(), (**elem).clone())], Vec::new());
            self.walking.pop();
            out.push(Node::block(format!("for {i}, {v} in {source} do"), body, "end"));
            return;
        }
        if roll == 3 {
            let w = self.fresh("w");
            let text = *self.rng.pick(&["\"a bb ccc\"", "\"x=1, y=22\"", "\"\""]);
            let body = self.loop_body(LoopKind::Plain, depth, vec![(w.clone(), Ty::Str)], Vec::new());
            out.push(Node::block(format!("for {w} in string.gmatch({text}, \"%w+\") do"), body, "end"));
            return;
        }
        let (i, v) = (self.fresh("k"), self.fresh("e"));
        let items = (0..self.rng.range(0, 3)).map(|_| self.num(1)).collect::<Vec<_>>();
        let body = self.loop_body(LoopKind::Plain, depth, vec![(i.clone(), Ty::Num), (v.clone(), Ty::Num)], Vec::new());
        out.push(Node::block(format!("for {i}, {v} in {{{}}} do", items.join(", ")), body, "end"));
    }

    fn loop_exit(&mut self, out: &mut Vec<Node>) {
        let can_continue = self.fns.last().unwrap().loops.last() == Some(&LoopKind::Plain);
        let word = if can_continue && self.rng.chance(45) { "continue" } else { "break" };
        if self.rng.chance(10) {
            out.push(Node::block("do", vec![Node::leaf(word)], "end"));
        } else {
            let cond = self.cond(2);
            out.push(Node::block(format!("if {cond} then"), vec![Node::leaf(word)], "end"));
        }
    }

    fn return_values(&mut self) -> String {
        let rets = self.fns.last().unwrap().rets.clone().unwrap_or_default();
        rets.iter().map(|t| self.expr(t, 2)).collect::<Vec<_>>().join(", ")
    }

    fn early_return(&mut self, out: &mut Vec<Node>) {
        let cond = self.cond(2);
        let values = self.return_values();
        let ret = if values.is_empty() { "return".to_string() } else { format!("return {values}") };
        out.push(Node::block(format!("if {cond} then"), vec![Node::leaf(ret)], "end"));
    }

    fn function_stmt(&mut self, depth: u32, out: &mut Vec<Node>) {
        let params = (0..self.rng.below(4)).map(|_| self.scalar_ty()).collect::<Vec<_>>();
        let rets = (0..self.rng.range(0, 3)).map(|_| self.scalar_ty()).collect::<Vec<_>>();
        let vararg = self.rng.chance(15);
        let names = params.iter().map(|_| self.fresh("p")).collect::<Vec<_>>();
        let mut list = names.clone();
        if vararg {
            list.push("...".into());
        }

        self.fns.push(FnCtx { loops: Vec::new(), rets: Some(rets.clone()), vararg });
        self.scopes.push(Vec::new());
        for (name, ty) in names.iter().zip(&params) {
            self.declare(name.clone(), ty.clone(), true);
        }
        let mut body = self.stmts(depth.min(2), 5);
        let values = self.return_values();
        if !values.is_empty() {
            body.push(Node::leaf(format!("return {values}")));
        }
        self.scopes.pop();
        self.fns.pop();

        let name = self.fresh("f");
        let header = if self.rng.chance(70) {
            format!("local function {name}({})", list.join(", "))
        } else {
            format!("local {name} = function({})", list.join(", "))
        };
        out.push(Node::block(header, body, "end"));
        self.declare(name, Ty::Fun(Rc::new(Sig { params, rets, vararg })), false);
    }

    // ---- templates for bug classes random statements rarely line up ----

    fn family(&mut self, out: &mut Vec<Node>) {
        if self.straight {
            let lines = match self.rng.below(4) {
                0 => self.fam_counter(),
                1 => self.fam_varargs(),
                2 => self.fam_constructors(),
                _ => self.fam_pressure(),
            };
            out.push(Node::leaf(lines.join("\n")));
            return;
        }
        let lines = match self.rng.below(15) {
            0 => self.fam_loop_closures(),
            1 => self.fam_counter(),
            2 => self.fam_class(),
            3 => self.fam_proxy(),
            4 => self.fam_errors(),
            5 => self.fam_varargs(),
            6 => self.fam_constructors(),
            7 => self.fam_order(),
            8 => self.fam_numbers(),
            9 => self.fam_strings(),
            10 => self.fam_pressure(),
            11 => self.fam_shadowing(),
            12 => self.fam_if_chain(),
            13 => self.fam_and_or(),
            _ => self.fam_iterators(),
        };
        out.push(Node::leaf(lines.join("\n")));
    }

    fn small(&mut self) -> &'static str {
        self.rng.pick(SMALL_NUMS)
    }

    fn fam_loop_closures(&mut self) -> Vec<String> {
        let (fs, gs) = (self.fresh("fs"), self.fresh("gs"));
        let k = self.rng.range(2, 4);
        let m = self.small();
        let (l1, l2) = (self.label(), self.label());
        vec![
            format!("local {fs} = {{}}"),
            format!("for i = 1, {k} do"),
            format!("\tlocal j = i * {m}"),
            format!("\t{fs}[i] = function(x) j += x return i + j end"),
            "end".into(),
            format!("out({l1}, {fs}[1](1), {fs}[1](1), {fs}[{k}](2))"),
            format!("local {gs}, w = {{}}, 0"),
            format!("while w < {k} do"),
            "\tw += 1".into(),
            "\tlocal c = w * 10".into(),
            "\tif w == 2 then continue end".into(),
            format!("\t{gs}[w] = function() c += 1 return c, w end"),
            "end".into(),
            format!("out({l2}, {gs}[1](), {gs}[1](), {gs}[{k}] and {gs}[{k}](), w)"),
        ]
    }

    fn fam_counter(&mut self) -> Vec<String> {
        let make = self.fresh("make");
        let (a, b, c) = (self.small(), self.small(), self.small());
        let (l1, l2) = (self.label(), self.label());
        vec![
            format!("local function {make}(start)"),
            "\tlocal n = start".into(),
            "\tlocal function inc(by) n += by return n end".into(),
            "\tlocal function get() return n end".into(),
            "\treturn inc, get, function() n = start end".into(),
            "end".into(),
            format!("local inc, get, reset = {make}({a})"),
            format!("inc({b})"),
            format!("out({l1}, get())"),
            "reset()".into(),
            format!("local inc2 = {make}({c})"),
            format!("out({l2}, inc(1), get(), inc2(5), get())"),
        ]
    }

    fn fam_class(&mut self) -> Vec<String> {
        let cls = self.fresh("Class");
        let (a, b) = (self.small(), self.small());
        let mut lines = vec![
            format!("local {cls} = {{}}"),
            format!("{cls}.__index = {cls}"),
            format!("function {cls}.new(x) return setmetatable({{x = x}}, {cls}) end"),
            format!("function {cls}:get() return self.x end"),
            format!("function {cls}:add(n) self.x += n return self end"),
        ];
        let mut uses = vec!["o1:get()".to_string(), "o1:add(2):get()".to_string()];
        let options: [(&str, &str); 8] = [
            ("__add = function(p, q) return {cls}.new(p.x + q.x) end", "(o1 + o2):get()"),
            ("__eq = function(p, q) return p.x == q.x end", "o1 == o2, o1 ~= o2"),
            ("__lt = function(p, q) return p.x < q.x end", "o1 < o2, o1 > o2"),
            ("__le = function(p, q) return p.x <= q.x end", "o1 <= o2, o1 >= o2"),
            ("__concat = function(p, q) return tostring(p) .. \"&\" .. tostring(q) end", "o1 .. \"s\""),
            ("__call = function(self, y) return self.x * y end", "o1(3)"),
            ("__len = function() return 7 end", "#o1"),
            ("__unm = function(p) return -p.x end", "-o1"),
        ];
        for (method, usage) in options {
            if self.rng.chance(50) {
                lines.push(format!("{cls}.{}", method.replace("{cls}", &cls)));
                uses.push(usage.to_string());
            }
        }
        let label = self.label();
        lines.push(format!("local o1, o2 = {cls}.new({a}), {cls}.new({b})"));
        lines.push(format!("out({label}, {})", uses.join(", ")));
        lines
    }

    fn fam_proxy(&mut self) -> Vec<String> {
        let (store, proxy, base, child) = (self.fresh("store"), self.fresh("proxy"), self.fresh("base"), self.fresh("child"));
        let (l1, l2) = (self.label(), self.label());
        vec![
            format!("local {store} = {{}}"),
            format!("local {proxy} = setmetatable({{}}, {{"),
            "\t__index = function(_, k) return tostring(k) .. \"!\" end,".into(),
            format!("\t__newindex = function(_, k, v) rawset({store}, k, v) end,"),
            "})".into(),
            format!("{proxy}.a = {}", self.small()),
            format!("{proxy}[1] = \"one\""),
            format!("out({l1}, {proxy}.a, {proxy}.zz, rawget({proxy}, \"a\"), {store}.a, {store}[1])"),
            format!("local {base} = {{greet = function(self) return \"hi \" .. self.name end, name = \"base\"}}"),
            format!("{base}.__index = {base}"),
            format!("local {child} = setmetatable({{name = \"child\"}}, {base})"),
            format!("out({l2}, {child}:greet(), {child}.greet == {base}.greet, getmetatable({child}) == {base})"),
        ]
    }

    fn fam_errors(&mut self) -> Vec<String> {
        let label = self.label();
        let n = self.small();
        vec![
            "local ok1, e1 = pcall(error, \"plain\")".into(),
            "local ok2, e2 = pcall(function() error({code = 5}) end)".into(),
            "local ok3, e3 = pcall(function() local z = nil return z.field end)".into(),
            "local ok4, e4 = pcall(function(t) return 1 + t end, {})".into(),
            format!("local ok5, e5 = pcall(function(a) if a > 1 then error(\"big \" .. a, 0) end return a end, {n})"),
            "local ok6, e6 = xpcall(function() error(\"x\") end, function(m) return \"handled: \" .. m end)".into(),
            "local ok7, e7, e8 = pcall(pcall, error, \"nested\")".into(),
            "local ok9, e9 = pcall(error)".into(),
            format!("out({label}, ok1, e1, ok2, e2.code, ok3, e3, ok4, e4, ok5, e5, ok6, e6, ok7, e7, e8, ok9, e9)"),
        ]
    }

    fn fam_varargs(&mut self) -> Vec<String> {
        let (va, fwd, tail) = (self.fresh("va"), self.fresh("fwd"), self.fresh("tail"));
        let [l1, l2, l3, l4] = [self.label(), self.label(), self.label(), self.label()];
        vec![
            format!("local function {va}(...)"),
            "\tlocal n = select(\"#\", ...)".into(),
            "\tlocal t = table.pack(...)".into(),
            "\tlocal first, second = ...".into(),
            "\treturn n, t.n, first, second, select(-1, ...)".into(),
            "end".into(),
            format!("local function {fwd}(...) return {va}(...) end"),
            format!("local function {tail}(...) return ... end"),
            format!("out({l1}, {va}(1, nil, 3, nil))"),
            format!("out({l2}, {fwd}(\"a\"), {tail}(1, 2), {tail}(3))"),
            format!("out({l3}, ({tail}(1, 2)), {{{tail}(1, 2, 3)}}, {{{tail}(1, 2), 9}}, select(2, \"a\", \"b\", \"c\"))"),
            format!("out({l4}, #{{...}}, select(\"#\", ...), {{n = select(\"#\", {tail}())}})"),
        ]
    }

    fn fam_constructors(&mut self) -> Vec<String> {
        let tail = self.fresh("tail");
        let mut lines = vec![format!("local function {tail}(...) return ... end")];
        let mut names = Vec::new();
        for _ in 0..self.rng.range(2, 4) {
            let mut entries = Vec::new();
            for _ in 0..self.rng.range(0, 6) {
                entries.push(match self.rng.below(7) {
                    0 | 1 => self.num(1),
                    2 => format!("{} = {}", self.rng.pick(FIELDS).field(), self.string(1)),
                    3 => format!("[{}] = {}", self.rng.pick(&["1", "2", "3", "1.5", "true", "\"k\""]), self.num(1)),
                    4 => format!("{{{}, {}}}", self.num(0), self.string(0)),
                    5 => format!("({tail}(1, 2))"),
                    _ => self.string(1),
                });
            }
            if self.rng.chance(40) {
                entries.push(format!("{tail}({}, {})", self.small(), self.small()));
            }
            let name = self.fresh("c");
            lines.push(format!("local {name} = {{{}}}", entries.join(", ")));
            names.push(name);
        }
        if self.rng.chance(50) {
            let big = self.fresh("big");
            let count = self.rng.range(40, 140);
            let items = (0..count).map(|i| (i * 3 % 17).to_string()).collect::<Vec<_>>();
            lines.push(format!("local {big} = {{{}, {tail}(7, 8, 9)}}", items.join(", ")));
            names.push(format!("#{big}, {big}[1], {big}[{count}], {big}[{}]", count + 3));
        }
        let label = self.label();
        lines.push(format!("out({label}, {})", names.join(", ")));
        lines
    }

    fn fam_order(&mut self) -> Vec<String> {
        let (log, tap, tbl) = (self.fresh("log"), self.fresh("tap"), self.fresh("tbl"));
        let label = self.label();
        vec![
            format!("local {log} = {{}}"),
            format!("local function {tap}(name, v) table.insert({log}, name) return v end"),
            format!("local {tbl} = {{}}"),
            format!("{tbl}[{tap}(\"k\", \"key\")] = {tap}(\"v\", 1)"),
            format!("local a, b = {tap}(\"a\", 1), {tap}(\"b\", 2)"),
            format!("a, b = {tap}(\"b2\", b), {tap}(\"a2\", a)"),
            format!("{tbl}.x, {tbl}.y = {tap}(\"x\", a), {tap}(\"y\", b)"),
            format!("local sum = {tap}(\"l\", 1) + {tap}(\"r\", 2) * {tap}(\"m\", 3)"),
            format!("local short = {tap}(\"p\", false) and {tap}(\"q\", 1) or {tap}(\"z\", 2)"),
            format!("local cmp = {tap}(\"c1\", 1) < {tap}(\"c2\", 2)"),
            format!("{tbl}[{tap}(\"ck\", \"count\")] = ({tbl}.count or 0) + {tap}(\"cv\", 1)"),
            format!("local s = {tap}(\"s1\", \"a\") .. {tap}(\"s2\", \"b\") .. {tap}(\"s3\", \"c\")"),
            format!("out({label}, table.concat({log}, \",\"), a, b, sum, short, cmp, s, {tbl})"),
        ]
    }

    fn fam_numbers(&mut self) -> Vec<String> {
        let label = self.label();
        let (a, b) = (self.rng.pick(NUMS), self.rng.pick(NUMS));
        vec![
            format!("local x, y = {a}, {b}"),
            "local nan, inf, nz = 0 / 0, math.huge, -0".into(),
            format!("out({label}, x + y, x - y, x * y, x / y, x // y, x % y, -x, x ^ 2,"),
            "\tnot (x < y), x >= y, not (x <= nan), nan ~= nan, nan == nan, x > nan,".into(),
            "\t1 / nz, nz == 0, -7 // 2, -7 % 3, 7 % -3, 5.5 % 2, -5.5 // 2, inf - inf, 2 ^ 53 + 1,".into(),
            "\t0.1 + 0.2, 1e308 * 10, -1e308 * 10, 1 / 3, 100 / 3, 3 % math.huge, -3 % math.huge, 8 // 0, -8 % 0)".into(),
        ]
    }

    fn fam_strings(&mut self) -> Vec<String> {
        let label = self.label();
        let picks = (0..3).map(|_| *self.rng.pick(STRS)).collect::<Vec<_>>();
        vec![
            format!("local s1, s2, s3 = {}, {}, {}", picks[0], picks[1], picks[2]),
            format!("out({label}, s1, #s1, s1:byte(1, -1))"),
            "out(\"Ls\", s2 .. s3, s2 < s3, string.format(\"%q|%5.2f|%-4s|%x\", s2, 3.14159, \"ab\", 255))".into(),
            "out(\"Lt\", (\"abc\"):rep(2, \",\"), s3:upper(), s1:find(\"a\", 1, true), #`{s1}{s2}`, `\\{{s3}}`)".into(),
        ]
    }

    fn fam_pressure(&mut self) -> Vec<String> {
        let heavy = self.fresh("heavy");
        let count = self.rng.range(30, 180) as usize;
        let mut lines = vec![format!("local function {heavy}(a, b)")];
        for i in 1..=count {
            let pick = |g: &mut Gen| -> String {
                if i == 1 || g.rng.chance(20) {
                    g.rng.pick(&["a", "b"]).to_string()
                } else {
                    format!("v{}", g.rng.range(1, i as i64 - 1))
                }
            };
            let (x, y) = (pick(self), pick(self));
            let op = self.rng.pick(&["+", "-", "*", "%"]);
            let wrap = if *op == "*" { format!("({x} {op} {y}) % 1000") } else { format!("{x} {op} {y}") };
            lines.push(format!("\tlocal v{i} = {wrap}"));
        }
        let rets = (0..4).map(|_| format!("v{}", self.rng.range(1, count as i64))).collect::<Vec<_>>();
        lines.push(format!("\treturn {}", rets.join(", ")));
        lines.push("end".into());
        let label = self.label();
        lines.push(format!("out({label}, {heavy}({}, {}))", self.rng.range(-5, 9), self.rng.range(-5, 9)));
        lines
    }

    fn fam_shadowing(&mut self) -> Vec<String> {
        let (l1, l2, l3) = (self.label(), self.label(), self.label());
        vec![
            "do".into(),
            "do".into(),
            "\tlocal math = {floor = function(x) return \"fake\" .. tostring(x) end, max = math.max}".into(),
            format!("\tout({l1}, math.floor(1.5), math.max(1, 2))"),
            "end".into(),
            "local string = {format = function(...) return select(\"#\", ...) end}".into(),
            format!("out({l2}, (\"abc\"):upper(), string.format(\"%d\", 1), math.floor(1.5))"),
            "local type = function() return \"mine\" end".into(),
            "local x = 1".into(),
            "do local x = x + 1 x *= 10 out(\"Lx\", x) end".into(),
            "for x = x, x + 1 do out(\"Ly\", x) end".into(),
            format!("out({l3}, type(1), x)"),
            "end".into(),
        ]
    }

    fn fam_if_chain(&mut self) -> Vec<String> {
        let classify = self.fresh("classify");
        let mut lines = vec![format!("local function {classify}(n)")];
        let count = self.rng.range(4, 12);
        for i in 0..count {
            let word = if i == 0 { "if" } else { "elseif" };
            let test = match self.rng.below(4) {
                0 => format!("n == {i}"),
                1 => format!("n > {} and n < {}", i * 3, i * 3 + 2),
                2 => format!("not (n ~= {i}) or n == -{i}"),
                _ => format!("n >= {} and (n % 2 == 0 or n == {})", i * 2, i * 5),
            };
            lines.push(format!("\t{word} {test} then"));
            lines.push(format!("\t\treturn \"b{i}\""));
        }
        if self.rng.chance(60) {
            lines.push("\telse".into());
            lines.push("\t\treturn \"other\"".into());
        }
        lines.push("\tend".into());
        lines.push("end".into());
        let label = self.label();
        let calls = (0..8).map(|_| format!("{classify}({})", self.rng.range(-2, 20))).collect::<Vec<_>>();
        lines.push(format!("out({label}, {}, {classify}(0 / 0))", calls.join(", ")));
        lines
    }

    fn fam_and_or(&mut self) -> Vec<String> {
        let label = self.label();
        let vals = ["nil", "false", "true", "0", "\"\"", "1"];
        let [a, b, c, d] = [0; 4].map(|_| *self.rng.pick(&vals));
        vec![
            format!("local a, b, c, d = {a}, {b}, {c}, {d}"),
            "local r1 = a and b or c".into(),
            "local r2 = (a or b) and (c or d)".into(),
            "local r3 = not not a".into(),
            "local r4 = a == nil".into(),
            "local r5 = if a then b else c".into(),
            "local r6 = a and (b or c) and d".into(),
            "local r7 = (a and b) == (c and d)".into(),
            "local r8 = not (a and b) or not c".into(),
            "if a and not b or c then r1 = \"then\" elseif not (c or d) then r1 = \"elseif\" end".into(),
            "while a and b do a = nil end".into(),
            format!("out({label}, r1, r2, r3, r4, r5, r6, r7, r8, a)"),
        ]
    }

    fn fam_iterators(&mut self) -> Vec<String> {
        let (range, keys, rec) = (self.fresh("range"), self.fresh("keys"), self.fresh("rec"));
        let (l1, l2, l3) = (self.label(), self.label(), self.label());
        let n = self.rng.range(0, 4);
        vec![
            format!("local function {range}(n)"),
            "\tlocal i = 0".into(),
            "\treturn function()".into(),
            "\t\ti += 1".into(),
            "\t\tif i <= n then return i, i * i end".into(),
            "\t\treturn nil".into(),
            "\tend".into(),
            "end".into(),
            "local acc = {}".into(),
            format!("for a, b in {range}({n}) do table.insert(acc, a + b) end"),
            format!("out({l1}, acc)"),
            format!("local {rec} = {{x = 1, y = 2, [\"two words\"] = 3, [10] = 4}}"),
            format!("local {keys} = {{}}"),
            format!("for k in pairs({rec}) do table.insert({keys}, tostring(k)) end"),
            format!("table.sort({keys})"),
            format!("out({l2}, table.concat({keys}, \",\"))"),
            "local down = {}".into(),
            "for i = #acc, 1, -1 do table.insert(down, acc[i]) end".into(),
            "local function stateless(t, i) i += 1 if t[i] then return i, t[i] end return nil end".into(),
            "for i, v in stateless, down, 0 do acc[i] = v * 2 end".into(),
            format!("out({l3}, down, acc)"),
        ]
    }
}
