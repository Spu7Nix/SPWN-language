use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;

use ahash::AHashMap;
use delve::{FieldNames, ModifyField, VariantNames};
use eager::*;
use lasso::Spur;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::error::RuntimeError;
use super::interpreter::{FuncCoord, RuntimeResult, ValueKey, Visibility, Vm};
use super::pattern::ConstPattern;
// use super::pattern::Pattern;
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
pub struct BuiltinFn(
    pub &'static (dyn Fn(Vec<ValueKey>, &mut Vm, CodeArea) -> RuntimeResult<Value>),
);

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
    pub args: Vec<MacroArg<Spanned<Spur>, ValueKey, ConstPattern>>,
    pub self_arg: Option<ValueKey>,
}

impl PartialEq for MacroData {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
} // 🙂

#[derive(Debug, Clone, PartialEq)]
pub enum IteratorData {
    Array {
        array: ValueKey,
        index: usize,
    },
    String {
        string: ValueKey,
        index: usize,
    },
    Range {
        range: (i64, i64, usize),
        index: usize,
    },
    Dictionary {
        map: ValueKey,
        keys: Vec<Spur>,
        index: usize,
    },
    Custom(ValueKey),
}

impl IteratorData {
    pub fn next(&self, vm: &Vm, area: CodeArea) -> Option<StoredValue> {
        match self {
            IteratorData::Array { array, index } => {
                match &vm.memory[*array].value {
                    Value::Array(values) => values.get(*index).map(|k| vm.memory[*k].clone()),
                    _ => todo!(), // maybe add error here incase its mutated???
                }
            }
            IteratorData::Range { range, index } => {
                let v = if range.1 >= range.0 {
                    (range.0..range.1).nth(*index * range.2)
                } else {
                    let l = (range.0 - range.1) as usize - 1;
                    if l >= *index * range.2 {
                        ((range.1 + 1)..(range.0 + 1)).nth(l - *index * range.2)
                    } else {
                        None
                    }
                };
                v.map(|v| StoredValue {
                    value: Value::Int(v),
                    area,
                })
            }
            IteratorData::String { string, index } => {
                match &vm.memory[*string].value {
                    Value::String(s) => s.get(*index).map(|c| StoredValue {
                        value: Value::String(vec![*c]),
                        area,
                    }),
                    _ => todo!(), // maybe add error here incase its mutated???
                }
            }
            // dict string TODO
            _ => unreachable!(),
        }
    }

    pub fn increment(&mut self) {
        match self {
            IteratorData::Array { index, .. } => *index += 1,
            IteratorData::Range { index, .. } => *index += 1,
            IteratorData::String { index, .. } => *index += 1,
            IteratorData::Dictionary { index, .. } => *index += 1,
            // dict string TODO
            _ => unreachable!(),
        }
    }
}

eager_macro_rules! { $eager_2
    #[macro_export]
    macro_rules! _destructure_names {
        ( ($t1:ty) ) => {
            (a)
        };
        ( ($t1:ty, $t2:ty) ) => {
            (a, b)
        };
        ( ($t1:ty, $t2:ty, $t3:ty) ) => {
            (a, b, c)
        };
        ( ($t1:ty, $t2:ty, $t3:ty, $t4:ty) ) => {
            (a, b, c, d)
        };
        ( { $( $n:ident: $t1:ty ,)* } ) => {
            {$($n,)*}
        }
    }
}
#[doc(hidden)]
pub use _destructure_names as destructure_names;

#[rustfmt::skip]
macro_rules! value {
    (
        $_:tt
        
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

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
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

        eager_macro_rules!{ $eager_1
            #[macro_export]
            macro_rules! _gen_wrapper {
                $(
                    (
                        $name
                    ) => {
                        $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )?
                    };

                    (
                        $v:vis struct $stname:ident : $_( $_($m:ident)? & $name )? $_(*$name)?
                    ) => {
                        $v struct $stname<'a> 
                            $( ( $( $_(&'a $_($m)?)? $t0, )* std::marker::PhantomData<&'a ()>); )?
                            $( { $( $n: $_(&'a $_($m)?)? $t1 ,)* _pd: std::marker::PhantomData<&'a ()> } )?
                    };

                    (
                        @destruct $name
                    ) => {
                        eager! {
                            destructure_names! { $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ,)* } )? }
                        }
                    };
                )*

                (
                    $v:vis enum $ename:ident: $_( $tys:ident) |* $_(|)?; $_( $extra:ident( $_($t:tt)* ) ),* $_(,)?
                ) => {
                    use eager::*;
                    eager! {
                        $v enum $ename {
                            $_ (
                                $tys gen_wrapper!( $tys ),
                            )*

                            $_(
                                $extra ( $_($t)* ),
                            )*
                        }
                    }
                };
            }
        }
        #[doc(hidden)]
        pub use _gen_wrapper as gen_wrapper;

        pub mod type_aliases {
            use super::*;

            pub trait TypeAliasDefaultThisIsNecessaryLOLItsSoThatItHasADefaultAndThenTheDirectImplInBuiltinUtilsOverwritesIt {
                fn get_override_fn(&self, name: &'static str) -> Option<BuiltinFn> {
                    None
                }
                fn get_override_const(&self, name: &'static str) -> Option<Constant> {
                    None
                }
            }

            $(
                pub struct $name;
                impl TypeAliasDefaultThisIsNecessaryLOLItsSoThatItHasADefaultAndThenTheDirectImplInBuiltinUtilsOverwritesIt for $name {}
            )*

            impl ValueType {
                pub fn get_override_fn(self, name: &'static str) -> Option<BuiltinFn> {
                    match self {
                        $(
                            Self::$name => type_aliases::$name.get_override_fn(name),
                        )*
                        _ => unreachable!(),
                    }
                }
                pub fn get_override_const(self, name: &'static str) -> Option<Constant> {
                    match self {
                        $(
                            Self::$name => type_aliases::$name.get_override_const(name),
                        )*
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
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
    $
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Vec<char>),

    Array(Vec<ValueKey>),
    Dict(AHashMap<Spur, (ValueKey, Visibility)>),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range(i64, i64, usize), //start, end, step

    Maybe(Option<ValueKey>),

    #[default]
    Empty,
    Macro(MacroData),

    Type(ValueType),

    Module {
        exports: AHashMap<Spur, ValueKey>,
        types: Vec<(CustomTypeKey, bool)>,
    },

    TriggerFunction {
        group: Id,
        prev_context: Id,
    },

    Object(AHashMap<u8, ObjParam>, ObjectType),

    Epsilon,

    Iterator(IteratorData),

    => Instance {
        typ: CustomTypeKey,
        items: AHashMap<Spur, (ValueKey, Visibility)>,
    },
}

impl Value {
    pub fn from_const(c: &Constant, vm: &mut Vm, area: &CodeArea) -> Self {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::String(v) => Value::String(v.chars().collect()),
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
            Constant::Array(arr) => Value::Array(
                arr.iter()
                    .map(|c| {
                        let value = Value::from_const(c, vm, area);
                        vm.memory.insert(StoredValue {
                            value,
                            area: area.clone(),
                        })
                    })
                    .collect(),
            ),
            Constant::Dict(m) => Value::Dict(
                m.iter()
                    .map(|(s, c)| {
                        let value = Value::from_const(c, vm, area);
                        (
                            vm.intern(s),
                            (
                                vm.memory.insert(StoredValue {
                                    value,
                                    area: area.clone(),
                                }),
                                Visibility::Public,
                            ),
                        )
                    })
                    .collect(),
            ),
            Constant::Maybe(o) => Value::Maybe(o.clone().map(|c| {
                let value = Value::from_const(&c, vm, area);
                vm.memory.insert(StoredValue {
                    value,
                    area: area.clone(),
                })
            })),
            Constant::Builtins => Value::Builtins,
            Constant::Empty => Value::Empty,
            Constant::Instance(t, m) => Value::Instance {
                typ: *t,
                items: m
                    .iter()
                    .map(|(s, c)| {
                        let value = Value::from_const(c, vm, area);
                        (
                            vm.intern(s),
                            (
                                vm.memory.insert(StoredValue {
                                    value,
                                    area: area.clone(),
                                }),
                                Visibility::Public,
                            ),
                        )
                    })
                    .collect(),
            },
        }
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
                    .map(|k| vm.memory[*k].value.runtime_display(vm))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Dict(d) => format!(
                "{{ {} }}",
                d.iter()
                    .map(|(s, (k, _))| format!(
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
                if !types.iter().any(|(_, p)| *p) {
                    format!(
                        "; {}",
                        types
                            .iter()
                            .filter(|(_, p)| !*p)
                            .map(|(t, _)| format!("@{}", vm.resolve(&vm.types[*t].value.name)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    "".into()
                }
            ),
            Value::Instance { typ, items } => format!(
                "@{}::{{ {} }}",
                vm.resolve(&vm.types[*typ].value.name),
                items
                    .iter()
                    .map(|(s, (k, _))| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        vm.memory[*k].value.runtime_display(vm)
                    ))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Value::Iterator(_) => "<iterator>".into(),
        }
    }
}
