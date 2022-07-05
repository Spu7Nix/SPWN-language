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

// ok so this is a temporary thing until we get builtins and i can replace this with _plus_ and such
pub mod value_ops {
    use crate::{
        interpreter::{Globals, StoredValue},
        sources::CodeArea,
    };

    use super::Value;

    pub fn plus(
        a: &StoredValue,
        b: &StoredValue,
        area: &CodeArea,
        globals: Globals,
    ) -> StoredValue {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 + *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 + *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 + *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 + *n2),
            _ => todo!(),
        };
        StoredValue {
            value,
            def_area: area.clone(),
        }
    }
}
