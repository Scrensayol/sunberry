#![feature(box_into_inner)]
#![feature(iter_order_by)]

pub mod block;
pub mod dot;
pub mod function;
// unfinished pattern matcher, nothing uses it yet
#[allow(dead_code)]
pub mod pattern;
pub mod ssa;
