use itertools::Either;
use rustc_hash::FxHashSet;

use crate::{
    formatter::Formatter, Block, LValue, Literal, LocalRw, RValue, RcLocal, Select, Statement,
    Traverse,
};

// Marks functions assigned to fields as methods, so they print as `function X:m(...)`
// with their first parameter named self. That is exactly `X.m = function(self, ...)`,
// so this only changes how the output reads. A function counts as a method when debug
// info names its first parameter self, or when that parameter is used as an object and
// the function either sits on a class table (`X.__index = X`) or its name is called
// with `:` somewhere in the script.
pub fn method_sugar(block: &mut Block) {
    let mut facts = Facts::default();
    collect_facts(block, &mut facts);
    mark_methods(block, &facts, &mut Vec::new());
}

#[derive(Default)]
struct Facts {
    called_methods: FxHashSet<String>,
    class_tables: Vec<RValue>,
}

enum Node<'a> {
    Statement(&'a Statement),
    Value(Either<&'a LValue, &'a RValue>),
}

// calls `f` on every statement in `block`, its nested blocks and the bodies of closures in
// it, and on every value inside those statements, in one traversal
fn walk(block: &mut Block, f: &mut impl FnMut(Node<'_>)) {
    for statement in &mut block.0 {
        f(Node::Statement(statement));
        statement.post_traverse_values(&mut |value| -> Option<()> {
            f(Node::Value(match &value {
                Either::Left(lvalue) => Either::Left(&**lvalue),
                Either::Right(rvalue) => Either::Right(&**rvalue),
            }));
            if let Either::Right(RValue::Closure(closure)) = value {
                walk(&mut closure.function.lock().body, f);
            }
            None
        });
        match statement {
            Statement::If(r#if) => {
                walk(&mut r#if.then_block.lock(), f);
                walk(&mut r#if.else_block.lock(), f);
            }
            Statement::While(r#while) => walk(&mut r#while.block.lock(), f),
            Statement::Repeat(repeat) => walk(&mut repeat.block.lock(), f),
            Statement::NumericFor(numeric_for) => walk(&mut numeric_for.block.lock(), f),
            Statement::GenericFor(generic_for) => walk(&mut generic_for.block.lock(), f),
            _ => {}
        }
    }
}

fn collect_facts(block: &mut Block, facts: &mut Facts) {
    walk(block, &mut |node| match node {
        Node::Statement(Statement::MethodCall(call))
        | Node::Value(Either::Right(
            RValue::MethodCall(call) | RValue::Select(Select::MethodCall(call)),
        )) => {
            facts.called_methods.insert(call.method.clone());
        }
        Node::Statement(Statement::Assign(assign))
            if let [LValue::Index(index)] = assign.left.as_slice()
                && let [value] = assign.right.as_slice()
                && let RValue::Literal(Literal::String(key)) = index.right.as_ref()
                && key == b"__index"
                && *index.left == *value
                && matches!(value, RValue::Local(_) | RValue::Global(_)) =>
        {
            facts.class_tables.push(value.clone());
        }
        _ => {}
    });
}

// is `local` indexed, assigned into or used as a method call receiver anywhere in `block`?
fn used_as_object(block: &mut Block, local: &RcLocal) -> bool {
    let is_local = |value: &RValue| matches!(value, RValue::Local(l) if l == local);
    let mut used = false;
    walk(block, &mut |node| {
        used |= match node {
            Node::Statement(Statement::MethodCall(call))
            | Node::Value(Either::Right(
                RValue::MethodCall(call) | RValue::Select(Select::MethodCall(call)),
            )) => is_local(&call.value),
            Node::Value(Either::Left(LValue::Index(index)) | Either::Right(RValue::Index(index))) => {
                is_local(&index.left)
            }
            _ => false,
        };
    });
    used
}

fn references_any(block: &mut Block, locals: &[RcLocal]) -> bool {
    let mut found = false;
    walk(block, &mut |node| {
        if let Node::Statement(statement) = node {
            found |= statement.values().into_iter().any(|l| locals.contains(l));
        }
    });
    found
}

// `selfs` holds the self of every method we're currently inside of
fn mark_methods(block: &mut Block, facts: &Facts, selfs: &mut Vec<RcLocal>) {
    for statement in &mut block.0 {
        if let Statement::Assign(assign) = statement
            && let [LValue::Index(index)] = assign.left.as_slice()
            && let [RValue::Closure(closure)] = assign.right.as_slice()
            && let RValue::Literal(Literal::String(key)) = index.right.as_ref()
            && Formatter::<String>::is_valid_name(key)
        {
            let mut function = closure.function.lock();
            let Some(first) = function.parameters.first().cloned() else {
                continue;
            };
            let named_self = first.0 .0.lock().0.as_deref() == Some("self");
            let looks_like_method = || {
                facts.class_tables.contains(&index.left)
                    || facts
                        .called_methods
                        .contains(String::from_utf8_lossy(key).as_ref())
            };
            // an outer self used in here would be shadowed by this one
            if (named_self || looks_like_method() && used_as_object(&mut function.body, &first))
                && (selfs.is_empty() || !references_any(&mut function.body, selfs))
            {
                function.is_method = true;
                first.0 .0.lock().0 = Some("self".to_string());
                selfs.push(first);
                mark_methods(&mut function.body, facts, selfs);
                selfs.pop();
                continue;
            }
        }

        statement.post_traverse_values(&mut |value| -> Option<()> {
            if let Either::Right(RValue::Closure(closure)) = value {
                mark_methods(&mut closure.function.lock().body, facts, selfs);
            }
            None
        });
        match statement {
            Statement::If(r#if) => {
                mark_methods(&mut r#if.then_block.lock(), facts, selfs);
                mark_methods(&mut r#if.else_block.lock(), facts, selfs);
            }
            Statement::While(r#while) => mark_methods(&mut r#while.block.lock(), facts, selfs),
            Statement::Repeat(repeat) => mark_methods(&mut repeat.block.lock(), facts, selfs),
            Statement::NumericFor(numeric_for) => {
                mark_methods(&mut numeric_for.block.lock(), facts, selfs)
            }
            Statement::GenericFor(generic_for) => {
                mark_methods(&mut generic_for.block.lock(), facts, selfs)
            }
            _ => {}
        }
    }
}
