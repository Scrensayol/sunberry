use itertools::Either;
use parking_lot::Mutex;
use rustc_hash::{FxHashMap, FxHashSet};
use triomphe::Arc;

use crate::{
    Block, Function, GenericFor, LValue, Literal, MethodCall, RValue, RcLocal, Select, Statement,
    Traverse, Upvalue,
};

const RESERVED: &[&str] = &[
    "and", "break", "continue", "do", "else", "elseif", "end", "export", "false", "for",
    "function", "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true",
    "type", "until", "while",
    // only methods get to be self, see name_local
    "self",
];

// Turns a string constant into something usable as a local name, or None when
// nothing sensible is left (empty, starts with a digit, absurdly long).
fn sanitize(raw: &[u8]) -> Option<String> {
    let name = raw
        .iter()
        .map(|&b| if b.is_ascii_alphanumeric() || b == b'_' { b as char } else { '_' })
        .collect::<String>();
    let name = name.trim_matches('_');
    let usable = !name.is_empty()
        && name.len() <= 32
        && !name.as_bytes()[0].is_ascii_digit()
        && name.bytes().any(|b| b != b'_');
    usable.then(|| name.to_string())
}

fn string_arg(arguments: &[RValue]) -> Option<&[u8]> {
    match arguments.first()? {
        RValue::Literal(Literal::String(s)) => Some(s),
        _ => None,
    }
}

fn index_key(value: &RValue) -> Option<&[u8]> {
    match value {
        RValue::Index(index) => match index.right.as_ref() {
            RValue::Literal(Literal::String(s)) => Some(s),
            _ => None,
        },
        _ => None,
    }
}

// Guesses a name for a local from the expression that declares it. Stripped
// bytecode keeps no local names, so this only goes off string constants: the
// service, child, module or class the value was fetched by.
fn name_hint(value: &RValue) -> Option<String> {
    match value {
        // a call truncated to one result, the usual shape for `local x = f()`
        RValue::Select(Select::Call(call)) => name_hint(&RValue::Call(call.clone())),
        RValue::Select(Select::MethodCall(call)) => name_hint(&RValue::MethodCall(call.clone())),
        RValue::MethodCall(call) => match call.method.as_str() {
            "GetService"
            | "WaitForChild"
            | "FindFirstChild"
            | "FindFirstChildOfClass"
            | "FindFirstChildWhichIsA"
            | "FindFirstAncestor"
            | "FindFirstAncestorOfClass"
            | "FindFirstAncestorWhichIsA" => sanitize(string_arg(&call.arguments)?),
            _ => None,
        },
        RValue::Call(call) => match call.value.as_ref() {
            // named after the module, however it was found: `a.B` or `a:WaitForChild("B")`
            RValue::Global(g) if g.0 == b"require" => match call.arguments.first()? {
                RValue::Global(_) => None,
                module => name_hint(module),
            },
            RValue::Global(g) if g.0 == b"tick" => Some("time".to_string()),
            RValue::Index(index) => match (index.left.as_ref(), index.right.as_ref()) {
                (RValue::Global(g), RValue::Literal(Literal::String(f)))
                    if g.0 == b"Instance" && f == b"new" =>
                {
                    let class = sanitize(string_arg(&call.arguments)?)?;
                    let mut chars = class.chars();
                    Some(chars.next()?.to_ascii_lowercase().to_string() + chars.as_str())
                }
                _ => None,
            },
            _ => None,
        },
        RValue::Index(_) => sanitize(index_key(value)?),
        RValue::Closure(closure) => sanitize(closure.function.lock().name.as_deref()?.as_bytes()),
        RValue::Global(g) => sanitize(&g.0),
        _ => None,
    }
}

// Conventional parameter names for an event's handler, mostly Roblox's own (scripts usually
// say `hit` and `gameProcessed` where the docs say `otherPart` and `gameProcessedEvent`), so a
// callback passed to `x.PlayerAdded:Connect` gets `player` instead of `p1`.
fn event_parameters(event: &[u8]) -> &'static [&'static str] {
    match event {
        b"PlayerAdded" | b"PlayerRemoving" | b"OnServerEvent" => &["player"],
        b"CharacterAdded" | b"CharacterRemoving" | b"CharacterAppearanceLoaded" => &["character"],
        b"ChildAdded" | b"ChildRemoved" => &["child"],
        b"DescendantAdded" | b"DescendantRemoving" => &["descendant"],
        b"AncestryChanged" => &["child", "parent"],
        b"Touched" | b"TouchEnded" => &["hit"],
        b"Heartbeat" | b"RenderStepped" | b"PreRender" | b"PreAnimation" | b"PreSimulation"
        | b"PostSimulation" => &["deltaTime"],
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

// What iterating over the result of these methods yields, for naming loop values.
fn collection_item(method: &str) -> Option<&'static str> {
    Some(match method {
        "GetChildren" => "child",
        "GetDescendants" => "descendant",
        "GetPlayers" => "player",
        "GetTagged" => "instance",
        "GetTouchingParts" | "GetPartsInPart" => "part",
        "GetPlayingAnimationTracks" => "track",
        _ => return None,
    })
}

fn local_key(local: &RcLocal) -> usize {
    Arc::as_ptr(&local.0 .0) as usize
}

struct Namer {
    rename: bool,
    counter: usize,
    upvalues: FxHashSet<RcLocal>,
    // every name a local can no longer be given: keywords, globals the script
    // reads or writes, and names already handed out
    taken: FxHashSet<String>,
    // names for parameters and loop variables, from how they're used. keyed by address,
    // holding the locals themselves would break the Arc::count check in name_local
    usage_hints: FxHashMap<usize, String>,
    // locals defined as `local function f`, so `Connect(f)` can name f's parameters
    local_functions: FxHashMap<usize, Arc<Mutex<Function>>>,
}

impl Namer {
    fn claim(&mut self, base: String) -> String {
        let mut name = base.clone();
        let mut suffix = 2;
        while !self.taken.insert(name.clone()) {
            name = format!("{}_{}", base, suffix);
            suffix += 1;
        }
        name
    }

    fn name_local(&mut self, prefix: &str, local: &RcLocal, hint: Option<String>) {
        let mut lock = local.0 .0.lock();
        if self.rename || lock.0.is_none() {
            // TODO: hacky and slow
            // a name already set came from debug info and beats any guess
            let known = lock.0.as_deref().and_then(|n| sanitize(n.as_bytes()));
            if Arc::count(&local.0 .0) == 1 {
                lock.0 = Some("_".to_string());
            } else if known.as_deref() == Some("self") {
                // every method has its own self, they shadow each other on purpose
                lock.0 = known;
            } else if let Some(name) = known.or(hint) {
                lock.0 = Some(self.claim(name));
            } else {
                let prefix = prefix.to_string()
                    + if self.upvalues.contains(local) {
                        "_u_"
                    } else {
                        ""
                    };
                let mut name = format!("{}{}", prefix, self.counter);
                self.counter += 1;
                while self.taken.contains(&name) {
                    name = format!("{}{}", prefix, self.counter);
                    self.counter += 1;
                }
                self.taken.insert(name.clone());
                lock.0 = Some(name);
            }
        }
    }

    // `x.Event:Connect(function(a, b) ... end)` or `x.Event:Connect(f)`
    fn hint_event_handler(&mut self, call: &MethodCall) {
        let function = match call.arguments.first() {
            Some(RValue::Closure(closure)) => Some(closure.function.0.clone()),
            Some(RValue::Local(local)) => self.local_functions.get(&local_key(local)).cloned(),
            _ => None,
        };
        if matches!(call.method.as_str(), "Connect" | "Once" | "ConnectParallel")
            && let RValue::Index(index) = call.value.as_ref()
            && let RValue::Literal(Literal::String(event)) = index.right.as_ref()
            && let Some(function) = function
        {
            let function = function.lock();
            for (param, name) in function.parameters.iter().zip(event_parameters(event)) {
                self.usage_hints.insert(local_key(param), name.to_string());
            }
        }
    }

    // `for _, v in x:GetChildren()` and `for _, v in ipairs(x:GetChildren())`
    fn hint_loop_value(&mut self, generic_for: &GenericFor) {
        let iterated = match generic_for.right.as_slice() {
            [RValue::Call(call) | RValue::Select(Select::Call(call))]
                if matches!(call.value.as_ref(), RValue::Global(g) if g.0 == b"ipairs" || g.0 == b"pairs") =>
            {
                call.arguments.first()
            }
            [single] => Some(single),
            _ => None,
        };
        if let Some(RValue::MethodCall(call) | RValue::Select(Select::MethodCall(call))) = iterated
            && let Some(item) = collection_item(&call.method)
            && let Some(value) = generic_for.res_locals.get(1)
        {
            self.usage_hints.insert(local_key(value), item.to_string());
        }
    }

    fn name_locals(&mut self, block: &mut Block) {
        for statement in &mut block.0 {
            // TODO: traverse_rvalues
            statement.post_traverse_values(&mut |value| -> Option<()> {
                if let itertools::Either::Right(RValue::Closure(closure)) = value {
                    let mut function = closure.function.lock();
                    for param in &function.parameters {
                        let hint = self.usage_hints.get(&local_key(param)).cloned();
                        self.name_local("p", param, hint);
                    }
                    self.name_locals(&mut function.body);
                };
                None
            });
            match statement {
                Statement::Assign(assign) if assign.prefix => {
                    let hints = if assign.left.len() == assign.right.len() {
                        assign.right.iter().map(name_hint).collect()
                    } else {
                        vec![None; assign.left.len()]
                    };
                    for (lvalue, hint) in assign.left.iter().zip(hints) {
                        self.name_local("v", lvalue.as_local().unwrap(), hint);
                    }
                }
                Statement::If(r#if) => {
                    self.name_locals(&mut r#if.then_block.lock());
                    self.name_locals(&mut r#if.else_block.lock());
                }
                Statement::While(r#while) => {
                    self.name_locals(&mut r#while.block.lock());
                }
                Statement::Repeat(repeat) => {
                    self.name_locals(&mut repeat.block.lock());
                }
                Statement::NumericFor(numeric_for) => {
                    self.name_local("v", &numeric_for.counter, None);
                    self.name_locals(&mut numeric_for.block.lock());
                }
                Statement::GenericFor(generic_for) => {
                    for res_local in &generic_for.res_locals {
                        let hint = self.usage_hints.get(&local_key(res_local)).cloned();
                        self.name_local("v", res_local, hint);
                    }
                    self.name_locals(&mut generic_for.block.lock());
                }
                Statement::Class(class) => {
                    let usable = !class.name.is_empty()
                        && class.name.bytes().next().is_some_and(|b| {
                            b.is_ascii_alphabetic() || b == b'_'
                        })
                        && class.name.bytes().all(|b| {
                            b.is_ascii_alphanumeric() || b == b'_'
                        });
                    if usable {
                        self.taken.insert(class.name.clone());
                        class.local.0 .0.lock().0 = Some(class.name.clone());
                    } else {
                        self.name_local("v", &class.local, None);
                    }
                }
                _ => {}
            }
        }
    }

    // TODO: does this need to be mut?
    fn find_upvalues(&mut self, block: &mut Block) {
        for statement in &mut block.0 {
            // TODO: traverse_values
            // TODO: doesnt need to be mut
            statement.post_traverse_values(&mut |value| -> Option<()> {
                match &value {
                    Either::Right(RValue::Global(g)) | Either::Left(LValue::Global(g)) => {
                        self.taken.insert(String::from_utf8_lossy(&g.0).into_owned());
                    }
                    _ => {}
                }
                if let Either::Right(
                    RValue::MethodCall(call) | RValue::Select(Select::MethodCall(call)),
                ) = &value
                {
                    self.hint_event_handler(call);
                }
                if let itertools::Either::Right(RValue::Closure(closure)) = value {
                    self.upvalues.extend(
                        closure
                            .upvalues
                            .iter()
                            .map(|u| match u {
                                Upvalue::Copy(l) | Upvalue::Ref(l) => l,
                            })
                            .cloned(),
                    );
                    self.find_upvalues(&mut closure.function.lock().body);
                };
                None
            });
            match statement {
                Statement::Assign(assign)
                    if assign.prefix
                        && let [LValue::Local(local)] = assign.left.as_slice()
                        && let [RValue::Closure(closure)] = assign.right.as_slice() =>
                {
                    self.local_functions
                        .insert(local_key(local), closure.function.0.clone());
                }
                Statement::MethodCall(call) => self.hint_event_handler(call),
                Statement::If(r#if) => {
                    self.find_upvalues(&mut r#if.then_block.lock());
                    self.find_upvalues(&mut r#if.else_block.lock());
                }
                Statement::While(r#while) => {
                    self.find_upvalues(&mut r#while.block.lock());
                }
                Statement::Repeat(repeat) => {
                    self.find_upvalues(&mut repeat.block.lock());
                }
                Statement::NumericFor(numeric_for) => {
                    self.find_upvalues(&mut numeric_for.block.lock());
                }
                Statement::GenericFor(generic_for) => {
                    self.hint_loop_value(generic_for);
                    self.find_upvalues(&mut generic_for.block.lock());
                }
                _ => {}
            }
        }
    }
}

pub fn name_locals(block: &mut Block, rename: bool) {
    let mut namer = Namer {
        rename,
        counter: 1,
        upvalues: FxHashSet::default(),
        taken: RESERVED.iter().map(|s| s.to_string()).collect(),
        usage_hints: FxHashMap::default(),
        local_functions: FxHashMap::default(),
    };
    namer.find_upvalues(block);
    namer.name_locals(block);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Assign, Block, Call, Global, Index, MethodCall, Return};

    fn string(s: &str) -> RValue {
        Literal::String(s.as_bytes().to_vec()).into()
    }

    fn global(s: &str) -> RValue {
        Global::new(s.as_bytes().to_vec()).into()
    }

    // single-result calls, as the lifter emits them
    fn call(value: RValue, arguments: Vec<RValue>) -> RValue {
        RValue::Select(Call::new(value, arguments).into())
    }

    fn method(value: RValue, name: &str, arguments: Vec<RValue>) -> RValue {
        RValue::Select(MethodCall::new(value, name.into(), arguments).into())
    }

    // `local <local> = <value>`, with the local read afterwards so it isn't
    // collapsed to `_`
    fn declare(local: &RcLocal, value: RValue) -> Vec<Statement> {
        let mut assign = Assign::new(vec![local.clone().into()], vec![value]);
        assign.prefix = true;
        vec![
            assign.into(),
            Return::new(vec![RValue::Local(local.clone())]).into(),
        ]
    }

    #[test]
    fn names_locals_from_constants() {
        let service = RcLocal::default();
        let child = RcLocal::default();
        let second_child = RcLocal::default();
        let module = RcLocal::default();
        let part = RcLocal::default();
        let field = RcLocal::default();
        let unknown = RcLocal::default();
        let clashing = RcLocal::default();
        let waited_module = RcLocal::default();
        let time = RcLocal::default();
        let mut body = Vec::new();
        body.extend(declare(
            &service,
            method(global("game"), "GetService", vec![string("Players")]),
        ));
        body.extend(declare(
            &child,
            method(RValue::Local(service.clone()), "WaitForChild", vec![string("PlayerGui")]),
        ));
        body.extend(declare(
            &second_child,
            method(RValue::Local(service.clone()), "WaitForChild", vec![string("PlayerGui")]),
        ));
        body.extend(declare(
            &module,
            call(
                global("require"),
                vec![Index::new(RValue::Local(service.clone()), string("GoodSignal")).into()],
            ),
        ));
        body.extend(declare(
            &part,
            call(Index::new(global("Instance"), string("new")).into(), vec![string("Part")]),
        ));
        body.extend(declare(&field, Index::new(RValue::Local(service.clone()), string("LocalPlayer")).into()));
        body.extend(declare(&unknown, call(global("mystery"), vec![])));
        body.extend(declare(
            &waited_module,
            call(
                global("require"),
                vec![method(global("script"), "WaitForChild", vec![string("CameraInput")])],
            ),
        ));
        body.extend(declare(&time, call(global("tick"), vec![])));
        // `workspace` is read as a global elsewhere, so the local can't take the name
        body.extend(declare(&clashing, Index::new(RValue::Local(service.clone()), string("workspace")).into()));
        body.push(Return::new(vec![global("workspace")]).into());

        name_locals(&mut Block(body), true);

        assert_eq!(service.to_string(), "Players");
        assert_eq!(child.to_string(), "PlayerGui");
        assert_eq!(second_child.to_string(), "PlayerGui_2");
        assert_eq!(module.to_string(), "GoodSignal");
        assert_eq!(part.to_string(), "part");
        assert_eq!(field.to_string(), "LocalPlayer");
        assert_eq!(unknown.to_string(), "v1");
        assert_eq!(clashing.to_string(), "workspace_2");
        assert_eq!(waited_module.to_string(), "CameraInput");
        assert_eq!(time.to_string(), "time");
    }

    #[test]
    fn rejects_unusable_hints() {
        assert_eq!(sanitize(b"my child"), Some("my_child".to_string()));
        assert_eq!(sanitize(b"3d"), None);
        assert_eq!(sanitize(b"___"), None);
        assert_eq!(sanitize(b""), None);
        assert_eq!(sanitize(&[b'a'; 40]), None);
    }
}
