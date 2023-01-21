use crate::{compiling::bytecode::Constant, gd::ids::*};

use super::interpreter::ValueKey;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<ValueKey>),

    Group(Id),
    Color(Id),
    Block(Id),
    Item(Id),

    // TriggerFunc(TriggerFunction),
    // Dict(AHashMap<LocalIntern<String>, StoredValue>),
    // Macro(Macro),

    // Array(Vec<StoredValue>),
    // Obj(Vec<(u16, ObjParam)>, ast::ObjectMode),
    Builtins,
    // // BuiltinFunction(Builtin),
    // TypeIndicator(TypeId),
    Range(i64, i64, usize), //start, end, step
                            // Pattern(Pattern),
}

impl Value {
    pub fn from_const(c: &Constant) -> Self {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::String(v) => Value::String(v.clone()),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::Id(c, v) => todo!(),
        }
    }
}
