use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use triomphe::Arc;

use crate::{Assign, Block, LocalRw, RValue, RcLocal, Statement, Traverse, Upvalue};

// A for loop's counter only exists inside the loop, but SSA destruction can merge it with a
// local that's still read after the loop: `found = i; break` turns into reading the counter
// itself once the loop is over, which is nil there. A loop like that gets a counter of its own
// and copies it into the local at the top of every iteration instead. Closures in the body
// capture the counter per iteration, so a loop whose counter is captured is left alone.
pub fn split_escaping_counters(block: &mut Block) {
    let mut counts = FxHashMap::default();
    count_block(block, &mut counts);
    split_block(block, &counts);
}

fn nested_blocks(statement: &Statement) -> Vec<Arc<Mutex<Block>>> {
    match statement {
        Statement::If(r#if) => vec![r#if.then_block.clone(), r#if.else_block.clone()],
        Statement::While(r#while) => vec![r#while.block.clone()],
        Statement::Repeat(repeat) => vec![repeat.block.clone()],
        Statement::NumericFor(numeric_for) => vec![numeric_for.block.clone()],
        Statement::GenericFor(generic_for) => vec![generic_for.block.clone()],
        Statement::Do(r#do) => vec![r#do.block.clone()],
        _ => Vec::new(),
    }
}

// closure bodies are other functions, which may still be decompiling, so this only counts
// what a closure captures
fn count_statement(statement: &Statement, counts: &mut FxHashMap<RcLocal, usize>) {
    for local in statement.values() {
        *counts.entry(local.clone()).or_default() += 1;
    }
    for nested in nested_blocks(statement) {
        count_block(&nested.lock(), counts);
    }
}

fn count_block(block: &Block, counts: &mut FxHashMap<RcLocal, usize>) {
    for statement in block.iter() {
        count_statement(statement, counts);
    }
}

fn captures(block: &mut Block, local: &RcLocal) -> bool {
    block.iter_mut().any(|statement| {
        let mut captured = false;
        statement.traverse_rvalues(&mut |rvalue| {
            if let RValue::Closure(closure) = rvalue
                && closure
                    .upvalues
                    .iter()
                    .any(|u| matches!(u, Upvalue::Copy(l) | Upvalue::Ref(l) if l == local))
            {
                captured = true;
            }
        });
        captured
            || nested_blocks(statement)
                .iter()
                .any(|nested| captures(&mut nested.lock(), local))
    })
}

fn split_block(block: &mut Block, counts: &FxHashMap<RcLocal, usize>) {
    for statement in block.iter_mut() {
        for nested in nested_blocks(statement) {
            split_block(&mut nested.lock(), counts);
        }
        if !matches!(statement, Statement::NumericFor(_) | Statement::GenericFor(_)) {
            continue;
        }
        let mut inside = FxHashMap::default();
        count_statement(statement, &mut inside);
        let escapes = |local: &RcLocal| counts[local] > inside[local];
        let (counters, body) = match statement {
            Statement::NumericFor(numeric_for) => {
                (vec![&mut numeric_for.counter], &numeric_for.block)
            }
            Statement::GenericFor(generic_for) => {
                (generic_for.res_locals.iter_mut().collect(), &generic_for.block)
            }
            _ => continue,
        };
        let mut copies = Vec::new();
        for counter in counters {
            if escapes(counter) && !captures(&mut body.lock(), counter) {
                let own = RcLocal::default();
                copies.push(
                    Assign::new(vec![counter.clone().into()], vec![own.clone().into()]).into(),
                );
                *counter = own;
            }
        }
        body.lock().0.splice(0..0, copies);
    }
}
