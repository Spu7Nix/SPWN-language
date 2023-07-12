use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use ahash::AHashMap;
use delve::{EnumFromStr, EnumToStr, EnumVariantNames, VariantNames};
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::error::ErrorDiscriminants;
use super::vm::{FuncCoord, RuntimeResult, Vm};
use crate::compiling::bytecode::Constant;
use crate::compiling::compiler::{CustomTypeID, LocalTypeID};
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::ObjectKey;
use crate::interpreting::vm::ValueRef;
use crate::new_id_wrapper;
use crate::parsing::ast::{MacroArg, Vis, VisSource, VisTrait};
use crate::sources::{CodeArea, Spanned};
use crate::util::{ImmutCloneStr, ImmutCloneVec, ImmutStr, ImmutVec};

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltinFn(pub &'static fn(Vec<ValueRef>, &mut Vm, CodeArea) -> RuntimeResult<Value>);

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

        pub mod value_structs {
            use super::*;
            
            #[derive(Debug, PartialEq)]
            pub struct FieldGetter<'a, T> {
                value_ref: &'a ValueRef,
                getter: fn(&ValueRef) -> std::cell::Ref<'a, T>,
                getter_mut: fn(&ValueRef) -> std::cell::RefMut<'a, T>,
            }

            impl<'a, T> FieldGetter<'a, T> {
                pub fn borrow(&'a self) -> std::cell::Ref<'a, T> {
                    (self.getter)(self.value_ref)
                }
                pub fn borrow_mut(&'a self) -> std::cell::RefMut<'a, T> {
                    (self.getter_mut)(self.value_ref)
                }
                pub fn parent_area(&'a self) -> CodeArea {
                    self.value_ref.borrow().area.clone()
                }
            } 


            paste::paste! {
                $(
                    value! { 
                        @struct [<$name Getter>]<'a> 
                        $( 
                            ( 
                                $( FieldGetter<'a, $t0>, )* 
                            )
                        )? 
                        $( 
                            { 
                                $( $n: FieldGetter<'a, $t1>, )* 
                            } 
                        )? 
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

    // required cause structs with curly braces cant have semicolons
    (@struct $name:ident $(<$lt:lifetime>)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name $(<$lt> (std::marker::PhantomData<&$lt ()>) )?;
    };
    (@struct $name:ident $(<$lt:lifetime>)? ( $( $t0:ty, )* )) => {
        #[derive(Debug, PartialEq)]
        pub struct $name $(<$lt>)? ( $( pub $t0, )* );

   
        impl<'a> $name<'a> {
            pub fn area(&self) -> CodeArea {
                self.0.parent_area()
                
                // const VALUE_REF_SIZE: usize = std::mem::size_of::<ValueRef>();
                // const STRUCT_SIZE: usize = std::mem::size_of::<$name>();

                // type Equiv = [u8; STRUCT_SIZE];
                // let ptr = self as *const $name as *const Equiv;
                // unsafe {
                //     let read =
                //         &std::ptr::read(ptr)[(STRUCT_SIZE - VALUE_REF_SIZE)..]
                //             as *const [u8]
                //             // as *const [u8; VALUE_REF_SIZE]
                //             as *const ValueRef;

                //     (*read).borrow().area.clone()
                // }
            }
        }
    };
    (@struct $name:ident $(<$lt:lifetime>)? { $( $n:ident: $t1:ty, )* }) => {
        #[derive(Debug, PartialEq)]
        pub struct $name $(<$lt>)? { $( pub $n: $t1, )* }

        impl<'a> $name<'a> {
            pub fn area(&self) -> CodeArea {
                $(
                    return self.$n.parent_area();
                )*
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroData {
    pub func: FuncCoord,

    pub args: ImmutVec<Spanned<MacroArg<ValueRef, ()>>>,
    pub self_arg: Option<ValueRef>,

    pub captured: ImmutCloneVec<ValueRef>,

    pub is_method: bool,

    pub is_builtin: Option<BuiltinFn>,
}

value! {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ImmutCloneVec<char>),

    Array(Vec<ValueRef>),
    Dict(AHashMap<ImmutCloneVec<char>, VisSource<ValueRef>>),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range(i64, i64, usize), //start, end, step

    Maybe(Option<ValueRef>),

    #[default]
    Empty,

    Macro(MacroData),

    Type(ValueType),

    Module {
        exports: AHashMap<ImmutCloneVec<char>, ValueRef>,
        types: Vec<Vis<CustomTypeID>>,
    },

    TriggerFunction {
        group: Id,
        prev_context: Id,
    },

    Error(usize),

    //Object(AHashMap<u8, ObjParam>, ObjectType),
    ObjectKey(ObjectKey),

    Epsilon,

    //Iterator(IteratorData),

    Chroma {
        r: u8, g: u8, b: u8, a: u8,
    },

    => Instance {
        typ: CustomTypeID,
        items: AHashMap<ImmutCloneVec<char>, VisSource<ValueRef>>,
    },
}

impl ValueType {
    pub fn runtime_display(self, vm: &Vm) -> String {
        format!(
            "@{}",
            match self {
                Self::Custom(t) => vm.type_def_map[&t].name.iter().collect::<String>(),
                _ => <ValueType as Into<&str>>::into(self).into(),
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
            Constant::String(v) => Value::String(v.clone().into()),
            Constant::Type(v) => Value::Type(*v),
            Constant::Id(class, id) => match class {
                IDClass::Group => Value::Group(Id::Specific(*id)),
                IDClass::Channel => Value::Channel(Id::Specific(*id)),
                IDClass::Block => Value::Block(Id::Specific(*id)),
                IDClass::Item => Value::Item(Id::Specific(*id)),
            },
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
                        s.iter().collect::<String>(),
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

            Value::Macro(MacroData { args, .. }) => {
                format!("<{}-arg macro at {:?}>", args.len(), (self as *const _))
            },
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
                        s.iter().collect::<String>(),
                        k.borrow().value.runtime_display(vm)
                    ))
                    .join(", "),
                if types.iter().any(|p| p.is_pub()) {
                    format!(
                        "; {}",
                        types
                            .iter()
                            .filter(|p| p.is_pub())
                            .map(|p| ValueType::Custom(*p.value()).runtime_display(vm))
                            .join(", ")
                    )
                } else {
                    "".into()
                }
            ),

            // Value::Iterator(_) => "<iterator>".into(),
            // Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
            Value::Error(id) => format!("{} {{...}}", ErrorDiscriminants::VARIANT_NAMES[*id]),

            Value::Instance { typ, items } => format!(
                "@{}::{{ {} }}",
                vm.type_def_map[&typ].name.iter().collect::<String>(),
                items
                    .iter()
                    .map(|(s, v)| format!(
                        "{}: {}",
                        s.iter().collect::<String>(),
                        v.value().borrow().value.runtime_display(vm)
                    ))
                    .join(", ")
            ),
            Value::ObjectKey(_) => todo!(),
            // todo: iterator, object
        }
    }
}
