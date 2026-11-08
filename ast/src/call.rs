use std::fmt;

use crate::{formatter::Formatter, has_side_effects, LocalRw, RcLocal, Traverse};

use super::RValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub value: Box<RValue>,
    pub arguments: Vec<RValue>,
}

impl Call {
    pub fn new(value: RValue, arguments: Vec<RValue>) -> Self {
        Self {
            value: Box::new(value),
            arguments,
        }
    }
}

// call can error
has_side_effects!(Call);
// impl SideEffects for Call {
//     fn has_side_effects(&self) -> bool {
//         matches!(self.value, box RValue::Local(_))
//             || self.value.has_side_effects()
//             || self.arguments.iter().any(|arg| arg.has_side_effects())
//     }
// }

impl Traverse for Call {
    fn rvalues_mut(&mut self) -> Vec<&mut RValue> {
        std::iter::once(self.value.as_mut())
            .chain(self.arguments.iter_mut())
            .collect()
    }

    fn rvalues(&self) -> Vec<&RValue> {
        std::iter::once(self.value.as_ref())
            .chain(self.arguments.iter())
            .collect()
    }
}

impl LocalRw for Call {
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
        self.value.push_values_read(out);
        for argument in &self.arguments {
            argument.push_values_read(out);
        }
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        self.value.push_values_read_mut(out);
        for argument in &mut self.arguments {
            argument.push_values_read_mut(out);
        }
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Formatter {
            indentation_level: 0,
            indentation_mode: Default::default(),
            output: f,
        }
        .format_call(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCall {
    // TODO: STYLE: rename to object?
    pub value: Box<RValue>,
    pub method: String,
    pub arguments: Vec<RValue>,
}

impl MethodCall {
    pub fn new(value: RValue, method: String, arguments: Vec<RValue>) -> Self {
        Self {
            value: Box::new(value),
            method,
            arguments,
        }
    }
}

// this should reflect Index
has_side_effects!(MethodCall);

impl Traverse for MethodCall {
    fn rvalues_mut(&mut self) -> Vec<&mut RValue> {
        std::iter::once(self.value.as_mut())
            .chain(self.arguments.iter_mut())
            .collect()
    }

    fn rvalues(&self) -> Vec<&RValue> {
        std::iter::once(self.value.as_ref())
            .chain(self.arguments.iter())
            .collect()
    }
}

impl LocalRw for MethodCall {
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
        self.value.push_values_read(out);
        for argument in &self.arguments {
            argument.push_values_read(out);
        }
    }

    fn push_values_read_mut<'a>(&'a mut self, out: &mut Vec<&'a mut RcLocal>) {
        self.value.push_values_read_mut(out);
        for argument in &mut self.arguments {
            argument.push_values_read_mut(out);
        }
    }
}

impl fmt::Display for MethodCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Formatter {
            indentation_level: 0,
            indentation_mode: Default::default(),
            output: f,
        }
        .format_method_call(self)
    }
}
