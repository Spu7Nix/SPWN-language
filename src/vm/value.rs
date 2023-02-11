use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use ahash::AHashMap;
use delve::{FieldNames, ModifyField, VariantNames};
use lasso::Spur;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::error::RuntimeError;
use super::interpreter::{FuncCoord, RuntimeResult, ValueKey, Vm};
use super::pattern::Pattern;
use crate::compiling::bytecode::Constant;
use crate::compiling::compiler::CustomTypeKey;
use crate::gd::gd_object::ObjParam;
use crate::gd::ids::*;
use crate::parsing::ast::{MacroArg, ObjKeyType, ObjectType, Spanned};
use crate::sources::CodeArea;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
}

#[derive(Clone)]
pub struct BuiltinFn(pub Rc<dyn Fn(Vec<ValueKey>, &mut Vm, CodeArea) -> RuntimeResult<Value>>);

#[derive(Clone, Debug)]
pub enum MacroTarget {
    Macro {
        func: FuncCoord,
        captured: Vec<ValueKey>,
    },
    Builtin(BuiltinFn),
}

impl Debug for BuiltinFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<builtin fn>")
    }
}

#[derive(Clone, Debug)]
pub struct MacroData {
    pub target: MacroTarget,
    pub args: Vec<MacroArg<Spanned<Spur>, ValueKey>>,
    pub self_arg: Option<ValueKey>,
}

impl PartialEq for MacroData {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[rustfmt::skip]
macro_rules! value {
    (
        $(
            $(#[$($meta:meta)*] )?
            $name:ident
                $( ( $( $t0:ty ),* ) )?
                $( { $( $n:ident: $t1:ty ,)* } )?
            ,
        )*

        => $i_name:ident
            $( ( $( $it0:ty ),* ) )?
            $( { $( $in:ident: $it1:ty ,)* } )?
        ,
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Value {
            $(
                $name $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )?,
            )*
            $i_name $( ( $( $it0 ),* ) )? $( { $( $in: $it1 ,)* } )?,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
        #[derive(delve::EnumFromStr, delve::EnumVariantNames, delve::EnumToStr)]
        #[delve(rename_all = "snake_case")]
        pub enum ValueType {
            $(
                $( #[$($meta)* ])?
                $name,
            )*
            
            #[delve(skip)]
            Custom(CustomTypeKey),
        }

        impl Value {
            pub fn get_type(&self) -> ValueType {
                match self {
                    Self::Instance { typ, .. } => ValueType::Custom(*typ),
                    
                    $(
                        Self::$name {..} => ValueType::$name,
                    )*
                }
            }
        }

        pub mod arg_aliases {
            use super::*;
            use crate::vm::builtins::builtin_utils::IntoArg;
            
            $(
                value!{ @semicolon $name $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )? }
            )*

            paste::paste! {
                $(
                    impl IntoArg<[<A $name>]> for ValueKey {
                        fn into_arg(self, vm: &Vm) -> [<A $name>] {

                            let val = vm.memory[self].value.clone();

                            value! {@into_arg_empty val $name $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )?}

                            $(
                                value! {@tuple_match val $name $( $t0, )*}
                            )?

                            match val {
                                $(
                                    Value::$name {$($n,)*} => return [<A $name>] {$($n,)*},
                                )?
                                _ => (),
                            }

                            unreachable!();
 
                        }
                    }
                )*
            }

        }
    };

    (@semicolon $name:ident) => {
        paste::paste! {
            pub struct [<A $name>];
        }
    };
    (@semicolon $name:ident ( $( $t0:ty ),* )) => {
        paste::paste! {
            pub struct [<A $name>] ( $( pub $t0 ),* );
        }
    };
    (@semicolon $name:ident { $( $n:ident: $t1:ty ,)* }) => {
        paste::paste! {
            pub struct [<A $name>] { $( pub $n: $t1 ,)* }
        }
    };

    (@tuple_match $self:ident $name:ident $t1:ty, $t2:ty, $t3:ty, $t4:ty, ) => {
        paste::paste! {
            match $self {
                Value::$name (a, b, c, d) => return [<A $name>](a, b, c, d),
                _ => (),
            }
        }
    };
    (@tuple_match $self:ident $name:ident $t1:ty, $t2:ty, $t3:ty, ) => {
        paste::paste! {
            match $self {
                Value::$name (a, b, c) => return [<A $name>](a, b, c),
                _ => (),
            }
        }
    };
    (@tuple_match $self:ident $name:ident $t1:ty, $t2:ty, ) => {
        paste::paste! {
            match $self {
                Value::$name (a, b) => return [<A $name>](a, b),
                _ => (),
            }
        }
    };
    (@tuple_match $self:ident $name:ident $t1:ty, ) => {
        paste::paste! {
            match $self {
                Value::$name (a) => return [<A $name>](a),
                _ => (),
            }
        }
    };

    (@into_arg_empty $self:ident $name:ident) => {
        paste::paste! {
            match $self {
                Value::$name => return [<A $name>],
                _ => (),
            }
        }
    };
    (@into_arg_empty $self:ident $name:ident $($t:tt)+) => {};
}

impl ValueType {
    pub fn runtime_display(self, vm: &Vm) -> String {
        format!(
            "@{}",
            match self {
                Self::Custom(t) => vm.resolve(&vm.types[t].value.name),
                _ => <ValueType as Into<&'static str>>::into(self).into(),
            }
        )
    }
}

value! {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<ValueKey>),
    Dict(AHashMap<Spur, ValueKey>),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range(i64, i64, usize), //start, end, step

    Maybe(Option<ValueKey>),
    Empty,
    Macro(MacroData),

    Type(ValueType),
    Pattern(Pattern),

    Module {
        exports: AHashMap<Spur, ValueKey>,
        types: Vec<CustomTypeKey>,
    },

    TriggerFunction {
        group: Id,
        prev_context: Id,
    },

    Object(AHashMap<u8, ObjParam>, ObjectType),

    Epsilon,

    => Instance {
        typ: CustomTypeKey,
        items: AHashMap<Spur, ValueKey>,
    },
}

impl Value {
    pub fn from_const(c: &Constant) -> Self {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::String(v) => Value::String(v.clone()),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::Id(c, v) => {
                let id = Id::Specific(*v);
                match c {
                    IDClass::Group => Value::Group(id),
                    IDClass::Color => Value::Channel(id),
                    IDClass::Block => Value::Block(id),
                    IDClass::Item => Value::Item(id),
                }
            }
            Constant::Type(k) => Value::Type(*k),
        }
    }

    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|k| vm.memory[*k].value.runtime_display(vm))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Dict(d) => format!(
                "{{ {} }}",
                d.iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        vm.memory[*k].value.runtime_display(vm)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Group(id) => id.fmt("g"),
            Value::Channel(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".to_string(),
            Value::Range(n1, n2, s) => {
                if *s == 1 {
                    format!("{n1}..{n2}")
                } else {
                    format!("{n1}..{s}..{n2}")
                }
            }
            Value::Maybe(o) => match o {
                Some(k) => format!("({})?", vm.memory[*k].value.runtime_display(vm)),
                None => "?".into(),
            },
            Value::Empty => "()".into(),

            Value::Macro(MacroData { args, .. }) => format!(
                "({}) {{...}}",
                args.iter()
                    .map(|a| vm.resolve(&a.name().value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Value::TriggerFunction { .. } => "!{...}".to_string(),
            Value::Type(t) => t.runtime_display(vm),
            Value::Object(map, typ) => format!(
                "{} {{ {} }}",
                match typ {
                    ObjectType::Object => "obj",
                    ObjectType::Trigger => "trigger",
                },
                map.iter()
                    .map(|(s, k)| format!("{s}: {k:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Epsilon => "$.epsilon()".to_string(),
            Value::Module { exports, types } => format!(
                "module {{ {}{} }}",
                exports
                    .iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        vm.memory[*k].value.runtime_display(vm)
                    ))
                    .collect::<Vec<_>>()
                    .join(", "),
                if !types.is_empty() {
                    format!(
                        "; {}",
                        types
                            .iter()
                            .map(|t| format!("@{}", vm.resolve(&vm.types[*t].value.name)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    "".into()
                }
            ),
            Value::Pattern(p) => p.runtime_display(vm),
            Value::Instance { typ, items } => format!(
                "@{}::{{ {} }}",
                vm.resolve(&vm.types[*typ].value.name),
                items
                    .iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        vm.memory[*k].value.runtime_display(vm)
                    ))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}
