use std::collections::HashMap;

use crate::{interpreter::StoredValue, sources::CodeArea};

pub type ArbitraryId = u16;
pub type SpecificId = u16;

#[derive(Debug, Clone, PartialEq)]
pub enum Id {
    Specific(SpecificId),
    ArbitraryId(ArbitraryId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(isize),
    Float(f64),

    String(String),

    Bool(bool),

    Empty,

    Array(Vec<StoredValue>),
    Dict(HashMap<String, StoredValue>),

    TypeIndicator(ValueType),
    Pattern(Pattern),

    Group(Id),
    TriggerFunc { start_group: Id },
}

impl Value {
    pub fn into_stored(self, area: CodeArea) -> StoredValue {
        StoredValue {
            value: self,
            def_area: area,
        }
    }
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
    // more soon
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

    pub fn plus(a: &StoredValue, b: &StoredValue, area: CodeArea) -> StoredValue {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 + *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 + *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 + *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 + *n2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1.clone() + s2),
            _ => todo!(), // here is where id return a runtime error but i have to see how errors work
        };
        value.into_stored(area)
    }
    pub fn minus(a: &StoredValue, b: &StoredValue, area: CodeArea) -> StoredValue {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 - *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 - *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 - *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 - *n2),
            _ => todo!(),
        };
        value.into_stored(area)
    }
    pub fn mult(a: &StoredValue, b: &StoredValue, area: CodeArea) -> StoredValue {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 * *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 * *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 * *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 * *n2),

            (Value::Int(n), Value::String(s)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            (Value::String(s), Value::Int(n)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            _ => todo!(),
        };
        value.into_stored(area)
    }
    pub fn div(a: &StoredValue, b: &StoredValue, area: CodeArea) -> StoredValue {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 / *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 / *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 / *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 / *n2),
            _ => todo!(),
        };
        value.into_stored(area)
    }
}
