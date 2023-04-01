use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;
use std::path::PathBuf;

use ahash::AHashMap;
use delve::{FieldNames, ModifyField, VariantNames};
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
use crate::gd::object_keys::ObjectKey;
use crate::parsing::ast::{MacroArg, ObjectType, Spanned};
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

impl std::hash::Hash for BuiltinFn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const dyn Fn(Vec<ValueKey>, &mut Vm, CodeArea) -> RuntimeResult<Value>)
            .hash(state);
    }
}

impl PartialEq for BuiltinFn {
    fn eq(&self, other: &Self) -> bool {
        (self.0 as *const dyn Fn(Vec<ValueKey>, &mut Vm, CodeArea) -> RuntimeResult<Value>)
            .eq(&(other.0
                as *const dyn Fn(Vec<ValueKey>, &mut Vm, CodeArea) -> RuntimeResult<Value>))
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct MacroData {
    pub target: MacroTarget,
    pub args: Vec<MacroArg<Spanned<Spur>, ValueKey, ConstPattern>>,
    pub self_arg: Option<ValueKey>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
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
            },
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
            },
            IteratorData::String { string, index } => {
                match &vm.memory[*string].value {
                    Value::String(s) => s.get(*index).map(|c| StoredValue {
                        value: Value::String(vec![*c]),
                        area,
                    }),
                    _ => todo!(), // maybe add error here incase its mutated???
                }
            },
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

        pub mod value_structs {
            use super::*;
            paste::paste! {
                $(
                    pub struct [<$name Getter>](pub ValueKey);

                    value! { @struct [<$name Deref>] $( ( $( $t0, )* ) )? $( { $( $n: $t1, )* } )? }
                    value! { @struct [<$name Ref>]<'a> $( ( $( &'a $t0, )* ) )? $( { $( $n: &'a $t1, )* } )? }
                    value! { @struct [<$name MutRef>]<'a> $( ( $( &'a mut $t0, )* ) )? $( { $( $n: &'a mut $t1, )* } )? }

                    impl [<$name Getter>] {
                        pub fn get_ref<'a>(&self, vm: &'a Vm) -> [<$name Ref>]<'a> {
                            match &vm.memory[self.0].value {
                                value! { @match (a, b, c, d) [Value::$name] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? }
                                    => value! { @match (a, b, c, d) [[<$name Ref>]] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? + std::marker::PhantomData },
                                _ => panic!("ValueKey does not point to value of correct type")
                            }
                        }
                        pub fn get_mut_ref<'a>(&self, vm: &'a mut Vm) -> [<$name MutRef>]<'a> {
                            match &mut vm.memory[self.0].value {
                                value! { @match (a, b, c, d) [Value::$name] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? }
                                    => value! { @match (a, b, c, d) [[<$name MutRef>]] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? + std::marker::PhantomData },
                                _ => panic!("ValueKey does not point to value of correct type")
                            }
                        }
                    }

                    impl From<Value> for [<$name Deref>] {
                        fn from(v: Value) -> Self {
                            match v {
                                value! { @match (a, b, c, d) [Value::$name] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? }
                                    => value! { @match (a, b, c, d) [[<$name Deref>]] $( ( $( $t0 ),* ) )? $( { $( $n: $t1 ),* } )? },
                                _ => panic!("ValueKey does not point to value of correct type")
                            }
                        }
                    }
                )*
            }
        }

        pub mod type_aliases {
            use super::*;

            pub trait TypeAliasDefaultThisIsNecessaryLOLItsSoThatItHasADefaultAndThenTheDirectImplInBuiltinUtilsOverwritesIt {
                fn get_override_fn(&self, _: &str) -> Option<BuiltinFn> {
                    None
                }
            }

            $(
                pub struct $name;
                impl TypeAliasDefaultThisIsNecessaryLOLItsSoThatItHasADefaultAndThenTheDirectImplInBuiltinUtilsOverwritesIt for $name {}
            )*

            impl ValueType {
                pub fn get_override_fn(self, name: &str) -> Option<BuiltinFn> {
                    match self {
                        $(
                            Self::$name => type_aliases::$name.get_override_fn(name),
                        )*
                        _ => unreachable!(),
                    }
                }
            }
        }
    };

    (@struct $name:ident $(<$lt:lifetime>)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name $(<$lt> (std::marker::PhantomData<&$lt ()>) )?;
    };
    (@struct $name:ident $(<$lt:lifetime>)? ( $( $t0:ty, )* )) => {
        #[derive(Debug, PartialEq)]
        pub struct $name $(<$lt>)? ( $( pub $t0, )* $(std::marker::PhantomData<&$lt ()>)? );

        ::defile::item! {
            value! { #try_deref $name, $( @$t0 ),* }
        }
    };
    (@struct $name:ident $(<$lt:lifetime>)? { $( $n:ident: $t1:ty, )* }) => {
        #[derive(Debug, PartialEq)]
        pub struct $name $(<$lt>)? { $( pub $n: $t1, )* $(_pd: std::marker::PhantomData<&$lt ()>)? }
    };

    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] $(+ $($extra:tt)*)? ) => { $($path)* $(( $($extra)* ))? };
    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] ($t1:ty) $(+ $extra:expr)? ) => { $($path)* ($a $(, $extra )? ) };
    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] ($t1:ty, $t2:ty) $(+ $extra:expr)? ) => { $($path)* ($a, $b $(, $extra )? ) };
    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] ($t1:ty, $t2:ty, $t3:ty) $(+ $extra:expr)? ) => { $($path)* ($a, $b, $c $(, $extra )? ) };
    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] ($t1:ty, $t2:ty, $t3:ty, $t4:ty) $(+ $extra:expr)? ) => { $($path)* ($a, $b, $c, $d $(, $extra )? ) };
    (@match ($a:ident, $b:ident, $c:ident, $d:ident) [$($path:tt)*] { $( $n:ident: $t1:ty ),* } $(+ $extra:expr)? ) => { $($path)* { $($n),* $(, _pd: $extra )? } };

    (#try_deref $name:ident, &$lt:lifetime mut $t:ty) => {
        impl std::ops::Deref for $name<'_> {
            type Target = $t;

            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl std::ops::DerefMut for $name<'_> {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    };
    (#try_deref $name:ident, &$lt:lifetime $t:ty) => {
        impl std::ops::Deref for $name<'_> {
            type Target = $t;

            fn deref(&self) -> &Self::Target { &self.0 }
        }
    };
    (#try_deref $name:ident, $t:ty) => {
        impl std::ops::Deref for $name {
            type Target = $t;

            fn deref(&self) -> &Self::Target { &self.0 }
        }
    };
    (#try_deref $($_:tt)*) => {};
}

impl ValueType {
    pub fn runtime_display(self, vm: &Vm) -> String {
        format!(
            "@{}",
            match self {
                Self::Custom(t) => vm.resolve(&vm.types[t].value.name),
                _ => <ValueType as Into<&str>>::into(self).into(),
            }
        )
    }
}

value! {
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

    Error(usize),

    Object(AHashMap<u8, ObjParam>, ObjectType),
    ObjectKey(ObjectKey),

    Epsilon,

    Iterator(IteratorData),

    Chroma {
        r: u8, g: u8, b: u8, a: u8,
    },

    Path(PathBuf),

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
            },
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
            Value::Chroma { r, g, b, a } => format!("@chroma::rgb8({r}, {g}, {b}, {a})"),
            Value::Range(n1, n2, s) => {
                if *s == 1 {
                    format!("{n1}..{n2}")
                } else {
                    format!("{n1}..{s}..{n2}")
                }
            },
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
            Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
            Value::Path(path) => path.to_str().unwrap_or("<path>").to_string(),
            Value::Error(_) => todo!(),
        }
    }
}
