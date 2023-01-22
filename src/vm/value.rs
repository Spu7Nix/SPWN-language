use ahash::AHashMap;
use lasso::Spur;

use crate::{compiling::bytecode::Constant, gd::ids::*, sources::CodeArea, util::Interner};

use super::interpreter::{ValueKey, Vm};

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
}

macro_rules! value_types {
    (
        $(
            #[type_name($s:literal)]
            $name:ident
            $( ($($t:ty),+) )?
            $( {$($field:ident: $field_t:ty,)+} )?,
        )+
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Value {
            $(
                $name $( ($($t),+) )? $( {$($field: $field_t,)+} )?,
            )+
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ValueType {
            $(
                $name,
            )+
        }

        impl Value {
            pub fn get_type(&self) -> ValueType {
                match self {
                    $(
                        Value::$name {..} => ValueType::$name,
                    )+
                }
            }
        }

        impl ValueType {
            pub fn type_name(&self) -> &str {
                match self {
                    $(
                        ValueType::$name => $s,
                    )+
                }
            }
        }
    };
}

value_types! {
    #[type_name("@int")]
    Int(i64),
    #[type_name("@float")]
    Float(f64),
    #[type_name("@bool")]
    Bool(bool),
    #[type_name("@string")]
    String(String),

    #[type_name("@array")]
    Array(Vec<ValueKey>),
    #[type_name("@dict")]
    Dict(AHashMap<Spur, ValueKey>),

    #[type_name("@group")]
    Group(Id),
    #[type_name("@color")]
    Color(Id),
    #[type_name("@block")]
    Block(Id),
    #[type_name("@item")]
    Item(Id),

    // TriggerFunc(TriggerFunction),
    // Dict(AHashMap<LocalIntern<String>, StoredValue>),
    // Macro(Macro),

    // Array(Vec<StoredValue>),
    // Obj(Vec<(u16, ObjParam)>, ast::ObjectMode),
    #[type_name("@builtins")]
    Builtins,
    // // BuiltinFunction(Builtin),
    // TypeIndicator(TypeId),
    #[type_name("@range")]
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

    pub fn display(&self, vm: &Vm) -> String {
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
                    s.push_str(&vm.memory[*el].value.display(vm));
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
                    s.push_str(&vm.memory[*v].value.display(vm));
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
