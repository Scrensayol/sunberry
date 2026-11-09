use crate::{limit_locals::for_each_block, Statement};

// Removes a `continue` that ends a loop body, where falling off the end does the same.
// Loop matching leaves these behind when it turns a loop's back edge into a statement.
pub fn remove_redundant_continues(block: &mut crate::Block) {
    for_each_block(block, &mut |block| {
        for statement in &mut block.0 {
            let body = match statement {
                Statement::While(r#while) => &r#while.block,
                Statement::Repeat(repeat) => &repeat.block,
                Statement::NumericFor(numeric_for) => &numeric_for.block,
                Statement::GenericFor(generic_for) => &generic_for.block,
                _ => continue,
            };
            let mut body = body.lock();
            if matches!(body.last(), Some(Statement::Continue(_))) {
                body.pop();
            }
        }
    });
}
