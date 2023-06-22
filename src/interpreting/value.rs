use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use delve::{EnumFromStr, EnumToStr, EnumVariantNames};
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::vm::Vm;
use crate::compiling::bytecode::Constant;
use crate::compiling::compiler::{CustomTypeID, LocalTypeID};
use crate::gd::ids::Id;
use crate::interpreting::vm::ValueRef;
use crate::new_id_wrapper;
use crate::parsing::ast::{VisSource, VisTrait};
use crate::sources::CodeArea;
use crate::util::{ImmutCloneStr, ImmutStr, ImmutVec};

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
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
        #[derive(Debug, Clone, PartialEq, Default)]
        pub enum Value {
            $(
                $(#[$($meta)*])?
                $name $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )?,
            )*
            $i_name $( ( $( $it0 ),* ) )? $( { $( $in: $it1 ,)* } )?,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default, EnumFromStr, EnumVariantNames, EnumToStr)]
        #[delve(rename_all = "snake_case")]
        pub enum ValueType {
            $(
                $( #[$($meta)* ])?
                $name,
            )*

            #[delve(skip)]
            Custom(CustomTypeID),
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

// impl ValueType {
//     pub fn runtime_display(self, vm: &Vm) -> String {
//         format!(
//             "@{}",
//             match self {
//                 Self::Custom(t) => vm.resolve(&vm.types[t].value.name),
//                 _ => <ValueType as Into<&str>>::into(self).into(),
//             }
//         )
//     }
// }

value! {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ImmutVec<char>),

    Array(Vec<ValueRef>),
    Dict(AHashMap<Spur, VisSource<ValueRef>>),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range(i64, i64, usize), //start, end, step

    Maybe(Option<ValueRef>),

    #[default]
    Empty,
    // Macro(MacroData),

    Type(ValueType),

    Module {
        exports: AHashMap<Spur, ValueRef>,
        types: Vec<(LocalTypeID, bool)>,
    },

    TriggerFunction {
        group: Id,
        prev_context: Id,
    },

    Error(usize),

    //Object(AHashMap<u8, ObjParam>, ObjectType),
    //ObjectKey(ObjectKey),

    Epsilon,

    //Iterator(IteratorData),

    Chroma {
        r: u8, g: u8, b: u8, a: u8,
    },

    => Instance {
        typ: CustomTypeID,
        items: AHashMap<Spur, VisSource<ValueRef>>,
    },
}

impl ValueType {
    pub fn runtime_display(self, vm: &Vm) -> String {
        format!(
            "@{}",
            match self {
                Self::Custom(t) => todo!(),
                _ => <ValueType as Into<&str>>::into(self),
            }
        )
    }
}

impl Value {
    pub fn from_const(_vm: &mut Vm, c: &Constant) -> Value {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::String(v) => Value::String(v.clone()),
        }
    }

    pub fn into_stored(self, area: CodeArea) -> StoredValue {
        StoredValue { value: self, area }
    }

    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("{:?}", s.iter().collect::<String>()),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|k| k.borrow().value.runtime_display(vm))
                    .join(", ")
            ),
            Value::Dict(d) => format!(
                "{{ {} }}",
                d.iter()
                    .map(|(s, v)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        v.value().borrow().value.runtime_display(vm)
                    ))
                    .join(", ")
            ),
            Value::Group(id) => id.fmt("g"),
            Value::Channel(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".to_string(),
            Value::Chroma { r, g, b, a } => format!("@chroma::rgb8({r}, {g}, {b}, {a})"),
            Value::Range(n1, n2, s) => {
                if *s == 1 {
                    format!("{n1}..{n2}")
                } else {
                    format!("{n1}..{s}..{n2}")
                }
            },
            Value::Maybe(o) => match o {
                Some(k) => format!("({})?", k.borrow().value.runtime_display(vm)),
                None => "?".into(),
            },
            Value::Empty => "()".into(),

            // Value::Macro(MacroData { args, .. }) => format!(
            //     "({}) {{...}}",
            //     args.iter()
            //         .map(|a| vm.resolve(&a.name().value))
            //         .collect::<Vec<_>>()
            //         .join(", ")
            // ),
            Value::TriggerFunction { .. } => "!{...}".to_string(),
            Value::Type(t) => t.runtime_display(vm),
            // Value::Object(map, typ) => format!(
            //     "{} {{ {} }}",
            //     match typ {
            //         ObjectType::Object => "obj",
            //         ObjectType::Trigger => "trigger",
            //     },
            //     map.iter()
            //         .map(|(s, k)| format!("{s}: {k:?}"))
            //         .collect::<Vec<_>>()
            //         .join(", ")
            // ),
            Value::Epsilon => "$.epsilon()".to_string(),
            Value::Module { exports, types } => format!(
                "module {{ {}{} }}",
                exports
                    .iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        k.borrow().value.runtime_display(vm)
                    ))
                    .join(", "),
                if !types.iter().any(|(_, p)| *p) {
                    format!(
                        "; {}",
                        types
                            .iter()
                            .filter(|(_, p)| !*p)
                            .map(|(t, _)| "todo")
                            .join(", ")
                    )
                } else {
                    "".into()
                }
            ),
            // Value::Instance { typ, items } => format!(
            //     "@{}::{{ {} }}",
            //     vm.resolve(&vm.types[*typ].value.name),
            //     items
            //         .iter()
            //         .map(|(s, (k, _))| format!(
            //             "{}: {}",
            //             vm.interner.borrow().resolve(s),
            //             vm.memory[*k].val.value.runtime_display(vm)
            //         ))
            //         .collect::<Vec<_>>()
            //         .join(", "),
            // ),
            // Value::Iterator(_) => "<iterator>".into(),
            // Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
            Value::Error(_) => todo!(),
            Value::Dict { .. } => todo!(),
            Value::Maybe { .. } => todo!(),
            Value::Instance { .. } => todo!(),
            Value::Module { .. } => todo!(),
            // todo: iterator, object
            _ => todo!(),
        }
    }
}
