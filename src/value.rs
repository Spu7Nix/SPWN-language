use crate::{interpreter::StoredValue, sources::CodeArea};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(isize),
    Float(f64),
    String(String),
    Bool(bool),
    Empty,
    Array(Vec<StoredValue>),
    TypeIndicator(ValueType),
    Pattern(Pattern),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Int,
    Float,
    String,
    Bool,
    Empty,
    Array,
    TypeIndicator,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Any,
}
