use itertools::Either;
use rustc_hash::FxHashMap;

use crate::{Block, LocalRw, RValue, RcLocal, Statement, Traverse};

// Luau allows 200 locals in scope at once. Big scripts, generated config tables mostly,
// decompile to more than that in one block, because every temporary gets declared at the
// top level of its function. Past LIMIT, this wraps runs of statements in `do ... end` so
// locals that are dead after the run go out of scope, which doesn't change behaviour.
const LIMIT: usize = 180;
const TARGET: usize = 150;

pub fn limit_locals(block: &mut Block) {
    for_each_block(block, &mut |block| {
        if declared_count(block) > LIMIT {
            scope_block(block);
        }
    });
}

fn declared(statement: &Statement) -> Vec<&RcLocal> {
    match statement {
        Statement::Assign(assign) if assign.prefix => {
            assign.left.iter().filter_map(|l| l.as_local()).collect()
        }
        _ => Vec::new(),
    }
}

fn declared_count(block: &Block) -> usize {
    block.iter().map(|s| declared(s).len()).sum()
}

// every block in `block`, innermost first, including the bodies of closures
pub(crate) fn for_each_block(block: &mut Block, f: &mut impl FnMut(&mut Block)) {
    for statement in &mut block.0 {
        statement.post_traverse_values(&mut |value| -> Option<()> {
            if let Either::Right(RValue::Closure(closure)) = value {
                for_each_block(&mut closure.function.lock().body, f);
            }
            None
        });
        match statement {
            Statement::If(r#if) => {
                for_each_block(&mut r#if.then_block.lock(), f);
                for_each_block(&mut r#if.else_block.lock(), f);
            }
            Statement::While(r#while) => for_each_block(&mut r#while.block.lock(), f),
            Statement::Repeat(repeat) => for_each_block(&mut repeat.block.lock(), f),
            Statement::NumericFor(numeric_for) => for_each_block(&mut numeric_for.block.lock(), f),
            Statement::GenericFor(generic_for) => for_each_block(&mut generic_for.block.lock(), f),
            Statement::Do(r#do) => for_each_block(&mut r#do.block.lock(), f),
            _ => {}
        }
    }
    f(block);
}

// every local referenced anywhere in `statement`, nested blocks and closures included
pub fn referenced(statement: &mut Statement, out: &mut Vec<RcLocal>) {
    out.extend(statement.values().into_iter().cloned());
    statement.post_traverse_values(&mut |value| -> Option<()> {
        if let Either::Right(RValue::Closure(closure)) = value {
            for statement in &mut closure.function.lock().body.0 {
                referenced(statement, out);
            }
        }
        None
    });
    let nested = |block: &mut Block, out: &mut Vec<RcLocal>| {
        for statement in &mut block.0 {
            referenced(statement, out);
        }
    };
    match statement {
        Statement::If(r#if) => {
            nested(&mut r#if.then_block.lock(), out);
            nested(&mut r#if.else_block.lock(), out);
        }
        Statement::While(r#while) => nested(&mut r#while.block.lock(), out),
        Statement::Repeat(repeat) => nested(&mut repeat.block.lock(), out),
        Statement::NumericFor(numeric_for) => nested(&mut numeric_for.block.lock(), out),
        Statement::GenericFor(generic_for) => nested(&mut generic_for.block.lock(), out),
        Statement::Do(r#do) => nested(&mut r#do.block.lock(), out),
        _ => {}
    }
}

fn scope_block(block: &mut Block) {
    // the last statement each local declared here is used in
    let mut last_use = FxHashMap::<RcLocal, usize>::default();
    for (index, statement) in block.0.iter_mut().enumerate() {
        for local in declared(statement) {
            last_use.insert(local.clone(), index);
        }
        let mut locals = Vec::new();
        referenced(statement, &mut locals);
        for local in locals {
            if let Some(last) = last_use.get_mut(&local) {
                *last = index;
            }
        }
    }

    // A local used far past its declaration has to stay at the top level, so its
    // declaration splits the block into segments. Inside a segment, cut the shortest runs
    // that nothing declared in them outlives and group consecutive runs into do blocks of
    // about TARGET locals each.
    const WINDOW: usize = 64;
    let pinned = |index: usize, statement: &Statement| {
        declared(statement)
            .iter()
            .any(|local| last_use[*local] > index + WINDOW)
    };
    let mut groups = Vec::new();
    let mut index = 0;
    while index < block.len() {
        if pinned(index, &block[index]) {
            index += 1;
            continue;
        }
        let mut segment_end = index;
        while segment_end < block.len() && !pinned(segment_end, &block[segment_end]) {
            segment_end += 1;
        }

        let (mut start, mut end, mut count) = (index, 0, 0);
        let mut last_closable = None;
        for (i, statement) in block.0[index..segment_end].iter().enumerate() {
            let i = index + i;
            for local in declared(statement) {
                end = end.max(last_use[local]);
                count += 1;
            }
            if i < end {
                continue;
            }
            if count >= TARGET {
                groups.push(start..i + 1);
                (start, count, last_closable) = (i + 1, 0, None);
            } else if count == 0 {
                start = i + 1;
            } else {
                last_closable = Some(i);
            }
        }
        if let Some(last) = last_closable {
            groups.push(start..last + 1);
        }
        index = segment_end;
    }

    for range in groups.into_iter().rev() {
        let body = block.0.splice(range.clone(), std::iter::empty()).collect::<Vec<_>>();
        block.0.insert(range.start, crate::Do::new(body.into()).into());
    }
}
