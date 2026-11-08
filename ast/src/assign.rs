use std::fmt;

use crate::{formatter::Formatter, RcLocal, SideEffects, Traverse};

use super::{LValue, LocalRw, RValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub left: Vec<LValue>,
    pub right: Vec<RValue>,
    pub prefix: bool,
    pub parallel: bool,
}

impl Assign {
    pub fn new(left: Vec<LValue>, right: Vec<RValue>) -> Self {
        Self {
            left,
            right,
            prefix: false,
            parallel: false,
        }
    }
}

impl Traverse for Assign {
    fn lvalues_mut(&mut self) -> Vec<&mut LValue> {
        self.left.iter_mut().collect()
    }

    fn rvalues_mut(&mut self) -> Vec<&mut RValue> {
        self.right.iter_mut().collect()
    }

    fn rvalues(&self) -> Vec<&RValue> {
        self.right.iter().collect()
    }
}

impl SideEffects for Assign {
    fn has_side_effects(&self) -> bool {
        self.right.iter().any(|r| r.has_side_effects())
            || self.left.iter().any(|l| l.has_side_effects())
    }
}

impl LocalRw for Assign {
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
        for l in &self.left {
            l.push_values_read(out);
        }
        for r in &self.right {
            r.push_values_read(out);
        }
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        for l in &mut self.left {
            l.push_values_read_mut(out);
        }
        for r in &mut self.right {
            r.push_values_read_mut(out);
        }
    }

    fn values_written(&self) -> Vec<&RcLocal> {
        self.left.iter().flat_map(|l| l.values_written()).collect()
    }

    fn values_written_mut(&mut self) -> Vec<&mut RcLocal> {
        self.left
            .iter_mut()
            .flat_map(|l| l.values_written_mut())
            .collect()
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Formatter {
            indentation_level: 0,
            indentation_mode: Default::default(),
            output: f,
        }
        .format_assign(self)
    }
}
