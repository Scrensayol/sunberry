// Names for locals that debug info didn't name, from what they hold: a function's own name,
// a service, a module, a child looked up by name, a field, a loop's key and value, a Roblox
// event's arguments. Only a suggestion: the printer still makes every name unique and keeps
// clear of globals the chunk uses.

use std::collections::HashMap;

use crate::ast::{Closure, Expr, Function, Stmt};
use crate::print::is_identifier;

type Name = Option<Vec<u8>>;

// `protos`: each function's own name, which Luau keeps even at debug level 1. `script`: the
// script's name, for the local a module returns when nothing better suggests itself.
pub fn infer(functions: &mut [Function], protos: &[Name], main: usize, script: Option<&str>) {
    for function in functions.iter_mut() {
        name_returned_function(function, protos);
    }
    // parameter names for handlers passed to a Roblox event, keyed by function
    let mut params = HashMap::<usize, Vec<&'static str>>::new();
    for function in functions.iter() {
        event_handlers(&function.body, &mut HashMap::new(), &mut params);
    }
    for (id, function) in functions.iter_mut().enumerate() {
        let mut hints = vec![None; function.names.len()];
        walk(&function.body, &mut hints, protos);
        for (&param, name) in function.params.iter().zip(params.get(&id).into_iter().flatten()) {
            set(&mut hints, param, Some(name.as_bytes().to_vec()));
        }
        param_roles(function, &mut hints);
        if id == main
            && let Some(Stmt::Return(values)) = function.body.last()
            && let [Expr::Local(module)] = values[..]
        {
            set(&mut hints, module, script.and_then(sanitize));
        }
        for (name, hint) in function.names.iter_mut().zip(hints) {
            if name.is_none() {
                *name = hint;
            }
        }
    }
}

// a script name as a local name: `Camera Module` is Camera_Module, `002_Foo` is nothing
fn sanitize(raw: &str) -> Option<Vec<u8>> {
    let name = raw.bytes().map(|b| if b.is_ascii_alphanumeric() || b == b'_' { b } else { b'_' }).collect::<Vec<_>>();
    let start = name.iter().position(|&b| b != b'_')?;
    let end = name.iter().rposition(|&b| b != b'_')? + 1;
    let name = name[start..end].to_vec();
    (name.len() <= 32 && !name[0].is_ascii_digit() && is_identifier(&name)).then_some(name)
}

// `return function(...) ... end` from a function the source named, usually a module's
// `local function Attendance(props) ... end return Attendance`: give it its name back
fn name_returned_function(function: &mut Function, protos: &[Name]) {
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

fn proto_name(closure: &Closure, protos: &[Name]) -> Name {
    protos.get(closure.proto).cloned().flatten().filter(|n| is_identifier(n))
}

fn set(hints: &mut [Name], local: u32, name: Name) {
    if let (Some(slot), Some(name)) = (hints.get_mut(local as usize), name)
        && slot.is_none()
        && is_identifier(&name)
    {
        *slot = Some(name);
    }
}

fn walk(body: &[Stmt], hints: &mut [Name], protos: &[Name]) {
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
                        let item = collection_item(values).map(<[u8]>::to_vec).or_else(|| iterated_field(values).and_then(singular));
                        set(hints, *v, Some(item.unwrap_or_else(|| b"v".to_vec())));
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

fn hint(e: &Expr) -> Name {
    match e {
        // game:GetService("Players"), char:WaitForChild("Humanoid")
        Expr::MethodCall { method, args, .. } if LOOKUPS.contains(&&method[..]) => match args.first() {
            Some(Expr::String(s)) => Some(s.clone()),
            _ => None,
        },
        Expr::Call { func, args, .. } => match (func.as_ref(), args.first()) {
            // require(script.Parent.Signal)
            (Expr::Global(g), Some(path)) if g == b"require" => hint(path),
            (Expr::Global(g), None) if g == b"tick" => Some(b"time".to_vec()),
            // setmetatable({}, Class) or setmetatable(Base.new(...), Class): a constructor's
            // object. The printer only uses `self` where no other self is in sight
            (Expr::Global(g), Some(object)) if g == b"setmetatable" && args.len() == 2 && constructed(object) => {
                Some(b"self".to_vec())
            }
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
            Expr::String(s) | Expr::RuntimeKey(s) if !enum_path(t) => Some(s.clone()),
            _ => None,
        },
        // plr.Character or plr.CharacterAdded:Wait()
        Expr::Binary(crate::ast::BinOp::Or, a, _) => hint(a),
        _ => None,
    }
}

fn constructed(object: &Expr) -> bool {
    match object {
        Expr::Table(_) => true,
        Expr::Call { func, .. } => matches!(func.as_ref(), Expr::Index(_, k) if **k == Expr::String(b"new".to_vec())),
        _ => false,
    }
}

fn enum_path(e: &Expr) -> bool {
    match e {
        Expr::Global(g) => g == b"Enum",
        Expr::Index(t, _) => enum_path(t),
        _ => false,
    }
}

// `for _, rsvp in event.rsvps`: the field a loop goes over, for naming its items
fn iterated_field(values: &[Expr]) -> Option<&[u8]> {
    let iterated = match values {
        [Expr::Call { func, args, .. }] if matches!(func.as_ref(), Expr::Global(g) if g == b"ipairs" || g == b"pairs") => args.first()?,
        [single] => single,
        _ => return None,
    };
    match iterated {
        Expr::Index(_, k) => match k.as_ref() {
            Expr::String(s) | Expr::RuntimeKey(s) => Some(s),
            _ => None,
        },
        _ => None,
    }
}

// a plural field name's singular, lowercased like a local: Items -> item, entries -> entry
fn singular(plural: &[u8]) -> Option<Vec<u8>> {
    let word = std::str::from_utf8(plural).ok()?;
    let lower = word.to_ascii_lowercase();
    let one = if word.len() > 4 && lower.ends_with("ies") {
        format!("{}y", &word[..word.len() - 3])
    } else if lower.ends_with("children") {
        format!("{}child", &word[..word.len() - 8])
    } else if ["sses", "xes", "ches", "shes"].iter().any(|e| lower.ends_with(e)) {
        word[..word.len() - 2].to_string()
    } else if word.len() > 3 && lower.ends_with('s') && !lower.ends_with("ss") && !lower.ends_with("us") {
        word[..word.len() - 1].to_string()
    } else {
        return None;
    };
    let mut one = one.into_bytes();
    if let Some(c) = one.first_mut() {
        c.make_ascii_lowercase();
    }
    is_identifier(&one).then_some(one)
}

// parameters named after how the function uses them: only ever called is `callback`; the
// first parameter of a React component (it returns createElement and reads fields off the
// parameter) is `props`
fn param_roles(function: &Function, hints: &mut [Name]) {
    for (i, &param) in function.params.iter().enumerate() {
        let mut called = false;
        let mut other = false;
        let mut indexed = false;
        visit(&function.body, &mut |e| match e {
            Expr::Call { func, args, .. } => {
                if **func == Expr::Local(param) {
                    called = true;
                }
                other |= args.contains(&Expr::Local(param));
            }
            Expr::Index(t, _) if **t == Expr::Local(param) => indexed = true,
            Expr::MethodCall { object, args, .. } => {
                other |= **object == Expr::Local(param) || args.contains(&Expr::Local(param));
            }
            Expr::Binary(_, a, b) => other |= **a == Expr::Local(param) || **b == Expr::Local(param),
            _ => {}
        });
        if called && !other && !indexed {
            set(hints, param, Some(b"callback".to_vec()));
        } else if i == 0 && indexed && returns_element(&function.body) {
            set(hints, param, Some(b"props".to_vec()));
        }
    }
}

fn returns_element(body: &[Stmt]) -> bool {
    body.iter().any(|stmt| match stmt {
        Stmt::Return(values) => matches!(values.first(), Some(Expr::Call { func, .. }) if matches!(func.as_ref(), Expr::Index(_, k) if matches!(k.as_ref(), Expr::String(n) | Expr::RuntimeKey(n) if n == b"createElement"))),
        _ => stmt.bodies().into_iter().any(|b| returns_element(b)),
    })
}

// every expression in `body`, nested ones included, but not inside functions it creates
fn visit(body: &[Stmt], f: &mut impl FnMut(&Expr)) {
    fn expr(e: &Expr, f: &mut impl FnMut(&Expr)) {
        f(e);
        match e {
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, f);
                expr(b, f);
            }
            Expr::Unary(_, a) => expr(a, f),
            Expr::IfElse(a, b, c) => {
                expr(a, f);
                expr(b, f);
                expr(c, f);
            }
            Expr::Call { func, args, .. } => {
                expr(func, f);
                args.iter().for_each(|a| expr(a, f));
            }
            Expr::MethodCall { object, args, .. } => {
                expr(object, f);
                args.iter().for_each(|a| expr(a, f));
            }
            Expr::Table(fields) => fields.iter().for_each(|field| match field {
                crate::ast::Field::Positional(v) => expr(v, f),
                crate::ast::Field::Keyed(k, v) => {
                    expr(k, f);
                    expr(v, f);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { values, .. } | Stmt::Return(values) => values.iter().for_each(|v| expr(v, f)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, f)),
            Stmt::Call(e) => expr(e, f),
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, f),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, f);
                expr(to, f);
                step.iter().for_each(|s| expr(s, f));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, f)),
            Stmt::LocalFunction { .. } | Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for nested in stmt.bodies() {
            visit(nested, f);
        }
    }
}

// what iterating `x:GetChildren()` (or `ipairs(...)` of it) yields
fn collection_item(values: &[Expr]) -> Option<&'static [u8]> {
    let iterated = match values {
        [Expr::Call { func, args, .. }] if matches!(func.as_ref(), Expr::Global(g) if g == b"ipairs" || g == b"pairs") => args.first()?,
        [single] => single,
        _ => return None,
    };
    let Expr::MethodCall { method, .. } = iterated else { return None };
    Some(match &method[..] {
        b"GetChildren" => b"child",
        b"GetDescendants" => b"descendant",
        b"GetPlayers" => b"player",
        b"GetTagged" => b"instance",
        b"GetTouchingParts" | b"GetPartsInPart" => b"part",
        b"GetPlayingAnimationTracks" => b"track",
        _ => return None,
    })
}

// Conventional parameter names for an event's handler, mostly Roblox's own (scripts usually
// say `hit` and `gameProcessed` where the docs say `otherPart` and `gameProcessedEvent`)
fn event_parameters(event: &[u8]) -> &'static [&'static str] {
    match event {
        b"PlayerAdded" | b"PlayerRemoving" | b"OnServerEvent" => &["player"],
        b"CharacterAdded" | b"CharacterRemoving" | b"CharacterAppearanceLoaded" => &["character"],
        b"ChildAdded" | b"ChildRemoved" => &["child"],
        b"DescendantAdded" | b"DescendantRemoving" => &["descendant"],
        b"AncestryChanged" => &["child", "parent"],
        b"Touched" | b"TouchEnded" => &["hit"],
        b"Heartbeat" | b"RenderStepped" | b"PreRender" | b"PreAnimation" | b"PreSimulation" | b"PostSimulation" => &["deltaTime"],
        b"Stepped" => &["time", "deltaTime"],
        b"InputBegan" | b"InputEnded" | b"InputChanged" => &["input", "gameProcessed"],
        b"Chatted" => &["message"],
        b"HealthChanged" => &["health"],
        b"StateChanged" => &["oldState", "newState"],
        b"MoveToFinished" => &["reached"],
        b"FocusLost" => &["enterPressed", "inputThatCausedFocusLoss"],
        _ => &[],
    }
}

// `Players.PlayerAdded:Connect(function(player) ... end)`, or a local function connected by
// name: the handler's parameters get the event's argument names
fn event_handlers(body: &[Stmt], functions: &mut HashMap<u32, usize>, params: &mut HashMap<usize, Vec<&'static str>>) {
    fn expr(e: &Expr, functions: &HashMap<u32, usize>, params: &mut HashMap<usize, Vec<&'static str>>) {
        match e {
            Expr::MethodCall { object, method, args, .. } => {
                if matches!(&method[..], b"Connect" | b"Once" | b"ConnectParallel")
                    && let Expr::Index(_, event) = object.as_ref()
                    && let Expr::String(event) | Expr::RuntimeKey(event) = event.as_ref()
                    && let [handler] = &args[..]
                {
                    let proto = match handler {
                        Expr::Closure(c) => Some(c.proto),
                        Expr::Local(l) => functions.get(l).copied(),
                        _ => None,
                    };
                    let names = event_parameters(event);
                    if let Some(proto) = proto
                        && !names.is_empty()
                    {
                        params.entry(proto).or_insert_with(|| names.to_vec());
                    }
                }
                expr(object, functions, params);
                args.iter().for_each(|a| expr(a, functions, params));
            }
            Expr::Index(a, b) | Expr::Binary(_, a, b) => {
                expr(a, functions, params);
                expr(b, functions, params);
            }
            Expr::Unary(_, a) => expr(a, functions, params),
            Expr::IfElse(a, b, c) => {
                expr(a, functions, params);
                expr(b, functions, params);
                expr(c, functions, params);
            }
            Expr::Call { func, args, .. } => {
                expr(func, functions, params);
                args.iter().for_each(|a| expr(a, functions, params));
            }
            Expr::Table(fields) => fields.iter().for_each(|f| match f {
                crate::ast::Field::Positional(v) => expr(v, functions, params),
                crate::ast::Field::Keyed(k, v) => {
                    expr(k, functions, params);
                    expr(v, functions, params);
                }
            }),
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            Stmt::Local { locals, values } => {
                values.iter().for_each(|v| expr(v, functions, params));
                if let ([local], [Expr::Closure(c)]) = (&locals[..], &values[..]) {
                    functions.insert(*local, c.proto);
                }
            }
            Stmt::LocalFunction { local, closure } => {
                functions.insert(*local, closure.proto);
            }
            Stmt::Return(values) => values.iter().for_each(|v| expr(v, functions, params)),
            Stmt::Assign { targets, values } => targets.iter().chain(values).for_each(|v| expr(v, functions, params)),
            Stmt::Call(e) => expr(e, functions, params),
            Stmt::If { cond, .. } | Stmt::While { cond, .. } | Stmt::Repeat { cond, .. } => expr(cond, functions, params),
            Stmt::NumericFor { from, to, step, .. } => {
                expr(from, functions, params);
                expr(to, functions, params);
                step.iter().for_each(|s| expr(s, functions, params));
            }
            Stmt::GenericFor { values, .. } => values.iter().for_each(|v| expr(v, functions, params)),
            Stmt::Break | Stmt::Continue | Stmt::Do(_) => {}
        }
        for nested in stmt.bodies() {
            event_handlers(nested, functions, params);
        }
    }
}
