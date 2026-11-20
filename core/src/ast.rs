// The output tree. Locals are numbered per function; upvalues are slots resolved through
// the capture list of the closure expression that creates the function, so each function
// can be lifted on its own and names are only decided when printing.

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Nil,
    Bool(bool),
    Number(f64),
    String(Vec<u8>),
    Vector([f64; 3]),
    // `multret` when every value is used; otherwise a single value, which needs parentheses
    // wherever the position would expand it (last argument, last field, return list)
    VarArg { multret: bool },
    Local(u32),
    Upvalue(u8),
    Global(Vec<u8>),
    Index(Box<Expr>, Box<Expr>),
    // a string key read at run time off a global path short enough that `a.b.c` would compile
    // to an import, which is looked up once when the script loads: printed `a.b["c"]` so it
    // stays a read at run time. Only ever an Index's key
    RuntimeKey(Vec<u8>),
    Call { func: Box<Expr>, args: Vec<Expr>, multret: bool },
    MethodCall { object: Box<Expr>, method: Vec<u8>, args: Vec<Expr>, multret: bool },
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Table(Vec<Field>),
    Closure(Box<Closure>),
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
    // while lifting a value region: the value a test just looked at, where `x and y` or
    // `x or y` keeps it; never left in a finished tree
    Tested(u64),
}

impl Expr {
    pub fn expands(&self) -> bool {
        matches!(
            self,
            Expr::VarArg { multret: true } | Expr::Call { multret: true, .. } | Expr::MethodCall { multret: true, .. }
        )
    }

    pub fn single(self) -> Expr {
        match self {
            Expr::VarArg { .. } => Expr::VarArg { multret: false },
            Expr::Call { func, args, .. } => Expr::Call { func, args, multret: false },
            Expr::MethodCall { object, method, args, .. } => Expr::MethodCall { object, method, args, multret: false },
            other => other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IDiv,
    Mod,
    Pow,
    Concat,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl BinOp {
    pub fn compares(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge)
    }
}

impl Expr {
    // always true or false, so `if e then true else false` is `e` itself
    pub fn is_boolean(&self) -> bool {
        match self {
            Expr::Bool(_) | Expr::Unary(UnOp::Not, _) => true,
            Expr::Binary(op, a, b) => op.compares() || (matches!(op, BinOp::And | BinOp::Or) && a.is_boolean() && b.is_boolean()),
            _ => false,
        }
    }

    pub fn contains_tested(&self, id: u64) -> bool {
        match self {
            Expr::Tested(t) => *t == id,
            Expr::Index(a, b) | Expr::Binary(_, a, b) => a.contains_tested(id) || b.contains_tested(id),
            Expr::Unary(_, a) => a.contains_tested(id),
            Expr::IfElse(a, b, c) => a.contains_tested(id) || b.contains_tested(id) || c.contains_tested(id),
            Expr::Call { func, args, .. } => func.contains_tested(id) || args.iter().any(|a| a.contains_tested(id)),
            Expr::MethodCall { object, args, .. } => object.contains_tested(id) || args.iter().any(|a| a.contains_tested(id)),
            Expr::Table(fields) => fields.iter().any(|f| match f {
                Field::Positional(v) => v.contains_tested(id),
                Field::Keyed(k, v) => k.contains_tested(id) || v.contains_tested(id),
            }),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
    Len,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Field {
    Positional(Expr),
    Keyed(Expr, Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub proto: usize,
    pub captures: Vec<Capture>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Capture {
    Local(u32),
    Upvalue(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Local { locals: Vec<u32>, values: Vec<Expr> },
    // `local function f` — the closure captures `local` itself
    LocalFunction { local: u32, closure: Closure },
    Assign { targets: Vec<Expr>, values: Vec<Expr> },
    Call(Expr),
    Return(Vec<Expr>),
    If { cond: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Repeat { body: Vec<Stmt>, cond: Expr },
    NumericFor { var: u32, from: Expr, to: Expr, step: Option<Expr>, body: Vec<Stmt> },
    GenericFor { vars: Vec<u32>, values: Vec<Expr>, body: Vec<Stmt> },
    Break,
    Continue,
    Do(Vec<Stmt>),
}

impl Stmt {
    // the statement lists nested directly in this one
    pub fn bodies(&self) -> Vec<&Vec<Stmt>> {
        match self {
            Stmt::If { then_body, else_body, .. } => vec![then_body, else_body],
            Stmt::While { body, .. } | Stmt::Repeat { body, .. } | Stmt::NumericFor { body, .. } | Stmt::GenericFor { body, .. } | Stmt::Do(body) => vec![body],
            _ => Vec::new(),
        }
    }

    pub fn bodies_mut(&mut self) -> Vec<&mut Vec<Stmt>> {
        match self {
            Stmt::If { then_body, else_body, .. } => vec![then_body, else_body],
            Stmt::While { body, .. } | Stmt::Repeat { body, .. } | Stmt::NumericFor { body, .. } | Stmt::GenericFor { body, .. } | Stmt::Do(body) => vec![body],
            _ => Vec::new(),
        }
    }

    // control never continues past it
    pub fn terminates(&self) -> bool {
        matches!(self, Stmt::Return(_) | Stmt::Break | Stmt::Continue)
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub params: Vec<u32>,
    pub is_vararg: bool,
    pub body: Vec<Stmt>,
    // debug names (debug level 2), indexed by local
    pub names: Vec<Option<Vec<u8>>>,
    // a type annotation for each parameter, where the compiler's type info gives one
    pub param_types: Vec<Option<String>>,
}
