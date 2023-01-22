use std::fmt::write;

use ahash::AHashMap;
use lasso::Spur;
use strum::EnumDiscriminants;

use crate::{compiling::bytecode::Constant, gd::ids::*, sources::CodeArea};

use super::interpreter::{ValueKey, Vm};

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
}

#[derive(EnumDiscriminants, Debug, Clone, PartialEq)]
// `EnumDiscriminants` generates a new enum that is just the variant names without any data
// anything in `strum_discriminants` is applied to the `ValueType` enum
#[strum_discriminants(name(ValueType))]
#[strum_discriminants(derive(delve::EnumToStr))]
#[strum_discriminants(delve(rename_all = "lowercase"))]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<ValueKey>),
    Dict(AHashMap<Spur, ValueKey>),

    Group(Id),
    Color(Id),
    Block(Id),
    Item(Id),

    Builtins,
    Range(i64, i64, usize), //start, end, step
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "@{}", <&ValueType as Into<&'static str>>::into(self))
    }
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        self.into()
    }

    pub fn from_const(c: &Constant) -> Self {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::String(v) => Value::String(v.clone()),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::Id(c, v) => todo!(),
        }
    }

    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Array(arr) => {
                let mut s = String::new();
                s.push('[');
                for (i, el) in arr.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&vm.memory[*el].value.runtime_display(vm));
                }
                s.push(']');
                s
            }
            Value::Dict(d) => {
                let mut s = String::new();
                s.push('{');
                for (i, (k, v)) in d.iter().enumerate() {
                    if i != 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&vm.interner.borrow().resolve(k));
                    s.push_str(": ");
                    s.push_str(&vm.memory[*v].value.runtime_display(vm));
                }
                s.push('}');
                s
            }
            Value::Group(id) => id.fmt("g"),
            Value::Color(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".to_string(),
            Value::Range(n1, n2, s) => {
                if *s == 1 {
                    format!("{}..{}", n1, n2)
                } else {
                    format!("{}..{}..{}", n1, s, n2)
                }
            }
        }
    }
}
