use std::rc::Rc;
use std::str::FromStr;

use ahash::AHashMap;
use delve::VariantNames;
use lasso::Spur;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::builtins::builtin_funcs::Builtin;
use super::builtins::builtin_utils::BuiltinType;
use super::error::RuntimeError;
use super::interpreter::{FuncCoord, ValueKey, Vm};
use super::pattern::Pattern;
use crate::compiling::bytecode::Constant;
use crate::compiling::compiler::CustomTypeKey;
use crate::gd::gd_object::ObjParam;
use crate::gd::ids::*;
use crate::parsing::ast::{ObjKeyType, ObjectType};
use crate::sources::CodeArea;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ArgData {
    pub name: Spur,
    pub default: Option<ValueKey>,
    pub pattern: Option<ValueKey>,
}

#[derive(Clone)]
pub struct BuiltinFn(
    pub Rc<dyn Fn(&mut Vec<ValueKey>, &mut Vm, CodeArea) -> Result<Value, RuntimeError>>,
);

#[derive(Clone)]
pub enum MacroCode {
    Normal {
        func: FuncCoord,
        args: Vec<ArgData>,
        captured: Vec<ValueKey>,
    },
    Builtin(BuiltinFn),
}

impl PartialEq for MacroCode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Normal {
                    func: f,
                    args: a,
                    captured: c,
                },
                Self::Normal {
                    func: of,
                    args: oa,
                    captured: oc,
                },
            ) => f == of && a == oa && c == oc,
            _ => false,
        }
    }
}

impl std::fmt::Debug for MacroCode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal {
                func: f,
                args: a,
                captured: c,
            } => {
                write!(
                    fmt,
                    "Normal {{ func: {f:?}, args: {a:?}, captured: {c:?} }}"
                )
            }
            Self::Builtin(..) => write!(fmt, "<builtin fn>"),
        }
    }
}

#[rustfmt::skip]
macro_rules! value {
    (
        $(
            $(#[$($meta:meta)*] )?
            $name:ident
                $( ( $($t0:tt)* ) )?
                $( { $($t1:tt)* } )?
            ,
        )*

        => $i_name:ident
            $( ( $($it0:tt)* ) )?
            $( { $($it1:tt)* } )?
        ,
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Value {
            $(
                $name $( ( $($t0)* ) )? $( { $($t1)* } )?,
            )*
            $i_name $( ( $($it0)* ) )? $( { $($it1)* } )?
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
    };
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
    Macro(MacroCode),

    Type(ValueType),
    Pattern(Pattern),

    Module {
        exports: AHashMap<Spur, ValueKey>,
        types: Vec<CustomTypeKey>,
    },

    TriggerFunction(Id),

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
            Value::Macro(MacroCode::Normal { args, .. }) => format!(
                "({}) {{...}}",
                args.iter()
                    .map(|d| vm.resolve(&d.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Macro(MacroCode::Builtin(_)) => "<builtin fn>".to_string(),
            Value::TriggerFunction(_) => "!{{...}}".to_string(),
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

    pub fn invoke_static(&self, name: &str, vm: &mut Vm) -> Result<Value, RuntimeError> {
        todo!();
        // match self {
        //     Value::TypeIndicator(id) => match id {
        //         0 => String::invoke_static(name, vm),
        //         _ => todo!(),
        //     },
        //     _ => unreachable!(),
        // }
    }

    // pub fn invoke_self(&self, name: &str, vm: &mut Vm) -> Result<Value, RuntimeError> {
    //     Ok(match (self, name) {
    //         (Value::String(s), "length") => Value::Int(),

    //         Value::Builtins => {
    //             let b = Builtin::from_str(name).unwrap();

    //             Value::Macro(MacroCode::Builtin(Rc::new(move |args, vm, area| {
    //                 b.call(args, vm, area)
    //             })))
    //         }
    //         _ => todo!(),
    //     })
    // }
}
