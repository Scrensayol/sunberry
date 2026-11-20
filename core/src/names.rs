// Names for locals that debug info didn't name, from what they hold: a function's own name,
// a service, a module, a child looked up by name, a field, a loop's key and value. Only a
// suggestion: the printer still makes every name unique and keeps clear of globals the chunk
// uses.

use crate::ast::{Closure, Expr, Function, Stmt};
use crate::print::is_identifier;

// `protos`: each function's own name, which Luau keeps even at debug level 1
pub fn infer(function: &mut Function, protos: &[Option<Vec<u8>>]) {
    name_returned_function(function, protos);
    let mut hints = vec![None; function.names.len()];
    walk(&function.body, &mut hints, protos);
    for (name, hint) in function.names.iter_mut().zip(hints) {
        if name.is_none() {
            *name = hint;
        }
    }
}

fn set(hints: &mut [Option<Vec<u8>>], local: u32, name: Option<Vec<u8>>) {
    if let (Some(slot), Some(name)) = (hints.get_mut(local as usize), name)
        && slot.is_none()
        && is_identifier(&name)
    {
        *slot = Some(name);
    }
}

// `return function(...) ... end` from a function the source named, usually a module's
// `local function Attendance(props) ... end return Attendance`: give it its name back
fn name_returned_function(function: &mut Function, protos: &[Option<Vec<u8>>]) {
    let Some(Stmt::Return(values)) = function.body.last() else { return };
    let [Expr::Closure(closure)] = &values[..] else { return };
    let Some(name) = proto_name(closure, protos) else { return };
    let closure = closure.clone();
    let local = function.names.len() as u32;
    function.names.push(Some(name));
    function.body.pop();
    function.body.push(Stmt::Local { locals: vec![local], values: vec![Expr::Closure(closure)] });
    function.body.push(Stmt::Return(vec![Expr::Local(local)]));
}

fn proto_name(closure: &Closure, protos: &[Option<Vec<u8>>]) -> Option<Vec<u8>> {
    protos.get(closure.proto).cloned().flatten().filter(|n| is_identifier(n))
}

fn walk(body: &[Stmt], hints: &mut [Option<Vec<u8>>], protos: &[Option<Vec<u8>>]) {
    let hint = |e: &Expr| match e {
        Expr::Closure(c) => proto_name(c, protos),
        e => hint(e),
    };
    for stmt in body {
        match stmt {
            Stmt::Local { locals, values } if locals.len() == 1 && values.len() == 1 => set(hints, locals[0], hint(&values[0])),
            Stmt::LocalFunction { local, closure } => set(hints, *local, proto_name(closure, protos)),
            Stmt::Local { locals, values } if locals.len() == 2 && values.len() == 1 && is_pcall(&values[0]) => {
                set(hints, locals[0], Some(b"success".to_vec()));
                set(hints, locals[1], Some(b"result".to_vec()));
            }
            Stmt::Assign { targets, values } if targets.len() == 1 && values.len() == 1 => {
                if let Expr::Local(l) = targets[0] {
                    set(hints, l, hint(&values[0]));
                }
            }
            Stmt::NumericFor { var, .. } => set(hints, *var, Some(b"i".to_vec())),
            Stmt::GenericFor { vars, values, .. } => {
                let indexed = matches!(values.first(), Some(Expr::Call { func, .. }) if **func == Expr::Global(b"ipairs".to_vec()));
                let key: &[u8] = if indexed { b"i" } else { b"k" };
                match &vars[..] {
                    [v] => set(hints, *v, Some(if indexed { b"i".to_vec() } else { b"v".to_vec() })),
                    [k, v, ..] => {
                        set(hints, *k, Some(key.to_vec()));
                        set(hints, *v, Some(b"v".to_vec()));
                    }
                    [] => {}
                }
            }
            _ => {}
        }
        for nested in stmt.bodies() {
            walk(nested, hints, protos);
        }
    }
}

fn is_pcall(e: &Expr) -> bool {
    matches!(e, Expr::Call { func, .. } if matches!(func.as_ref(), Expr::Global(g) if g == b"pcall" || g == b"xpcall"))
}

const LOOKUPS: &[&[u8]] = &[
    b"GetService",
    b"WaitForChild",
    b"FindFirstChild",
    b"FindFirstChildOfClass",
    b"FindFirstChildWhichIsA",
    b"FindFirstAncestor",
    b"FindFirstAncestorOfClass",
    b"FindFirstAncestorWhichIsA",
];

fn hint(e: &Expr) -> Option<Vec<u8>> {
    match e {
        // game:GetService("Players"), char:WaitForChild("Humanoid")
        Expr::MethodCall { method, args, .. } if LOOKUPS.contains(&&method[..]) => match args.first() {
            Some(Expr::String(s)) => Some(s.clone()),
            _ => None,
        },
        Expr::Call { func, args, .. } => match (func.as_ref(), args.first()) {
            // require(script.Parent.Signal)
            (Expr::Global(g), Some(path)) if g == b"require" => hint(path),
            // Instance.new("Part")
            (Expr::Index(t, k), Some(Expr::String(class)))
                if **t == Expr::Global(b"Instance".to_vec()) && **k == Expr::String(b"new".to_vec()) =>
            {
                let mut name = class.clone();
                if let Some(c) = name.first_mut() {
                    c.make_ascii_lowercase();
                }
                Some(name)
            }
            _ => None,
        },
        // script.Parent, props.onActivated (not Enum.KeyCode.E)
        Expr::Index(t, k) => match k.as_ref() {
            Expr::String(s) if !enum_path(t) => Some(s.clone()),
            _ => None,
        },
        // plr.Character or plr.CharacterAdded:Wait()
        Expr::Binary(crate::ast::BinOp::Or, a, _) => hint(a),
        _ => None,
    }
}

fn enum_path(e: &Expr) -> bool {
    match e {
        Expr::Global(g) => g == b"Enum",
        Expr::Index(t, _) => enum_path(t),
        _ => false,
    }
}
