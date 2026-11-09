use parking_lot::Mutex;
use triomphe::Arc;

use crate::{formatter::Formatter, has_side_effects, Block, LocalRw, Traverse};
use std::fmt;

// `do ... end`, only emitted to scope locals, see limit_locals
#[derive(Debug, Clone)]
pub struct Do {
    pub block: Arc<Mutex<Block>>,
}

impl PartialEq for Do {
    fn eq(&self, _other: &Self) -> bool {
        // TODO: compare block
        false
    }
}

has_side_effects!(Do);

impl Do {
    pub fn new(block: Block) -> Self {
        Self {
            block: Arc::new(block.into()),
        }
    }
}

impl Traverse for Do {}

impl LocalRw for Do {}

impl fmt::Display for Do {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Formatter {
            indentation_level: 0,
            indentation_mode: Default::default(),
            output: f,
        }
        .format_do(self)
    }
}
