use crate::{formatter::Formatter, has_side_effects, LocalRw, RcLocal, Traverse};

use super::RValue;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub left: Box<RValue>,
    pub right: Box<RValue>,
}

// this should be the same as MethodCall
has_side_effects!(Index);

impl Index {
    pub fn new(left: RValue, right: RValue) -> Self {
        Self {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl LocalRw for Index {
    fn values_read(&self) -> Vec<&RcLocal> {
        let mut out = Vec::new();
        self.push_values_read(&mut out);
        out
    }

    fn values_read_mut(&mut self) -> Vec<&mut RcLocal> {
        let mut out = Vec::new();
        self.push_values_read_mut(&mut out);
        out
    }

    fn push_values_read<'a>(&'a self, out: &mut Vec<&'a RcLocal>) {
        self.left.push_values_read(out);
        self.right.push_values_read(out);
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        self.left.push_values_read_mut(out);
        self.right.push_values_read_mut(out);
    }
}

impl Traverse for Index {
    fn rvalues_mut(&mut self) -> Vec<&mut RValue> {
        vec![&mut self.left, &mut self.right]
    }

    fn rvalues(&self) -> Vec<&RValue> {
        vec![&self.left, &self.right]
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Formatter {
            indentation_level: 0,
            indentation_mode: Default::default(),
            output: f,
        }
        .format_index(self)
    }
}
