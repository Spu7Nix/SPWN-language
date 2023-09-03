use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use ahash::AHashMap;
use delve::{EnumFromStr, EnumToStr, EnumVariantNames, VariantNames};
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::context::{CloneMap, Context};
use super::multi::Multi;
use super::vm::{FuncCoord, Program, RuntimeResult, Vm};
use crate::compiling::bytecode::Constant;
use crate::compiling::compiler::{CustomTypeID, LocalTypeID};
use crate::gd::gd_object::ObjParam;
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::ObjectKey;
use crate::interpreting::vm::ValueRef;
use crate::new_id_wrapper;
use crate::parsing::ast::{MacroArg, ObjectType, Vis, VisSource, VisTrait};
use crate::sources::{CodeArea, Spanned};
use crate::util::{ImmutCloneStr, ImmutCloneStr32, ImmutCloneVec, ImmutStr, ImmutVec, String32};

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct BuiltinFn(
    pub  fn(
        Vec<ValueRef>,
        Context,
        &mut Vm,
        &Rc<Program>,
        CodeArea,
        bool,
    ) -> Multi<RuntimeResult<ValueRef>>,
);

pub trait BuiltinClosure: Debug {
    fn shallow_clone(&self) -> Rc<RefCell<dyn BuiltinClosure>>;
    fn get_mut_refs<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut ValueRef> + 'a>;

    fn deep_clone_inner_refs(
        &mut self,
        vm: &Vm,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) {
        for v in self.get_mut_refs() {
            *v = v.deep_clone_checked(vm, map, context_split);
        }
    }

    fn call(
        &mut self,
        args: Vec<ValueRef>,
        context: Context,
        vm: &mut Vm,
        program: &Rc<Program>,
        area: CodeArea,
    ) -> Multi<RuntimeResult<ValueRef>>;
}

// macro_rules! builtin- {
//     () => {

//     };
// }

// #[derive(Clone)]
// pub struct BuiltinClosure(
//     pub  Rc<
//         RefCell<
//             dyn FnMut(
//                 Vec<ValueRef>,
//                 Context,
//                 &mut Vm,
//                 &Rc<Program>,
//                 CodeArea,
//             ) -> Multi<RuntimeResult<ValueRef>>,
//         >,
//     >,
// );

// impl BuiltinClosure {
//     pub fn new<F>(f: F) -> Self
//     where
//         F: FnMut(
//                 Vec<ValueRef>,
//                 Context,
//                 &mut Vm,
//                 &Rc<Program>,
//                 CodeArea,
//             ) -> Multi<RuntimeResult<ValueRef>>
//             + 'static,
//     {
//         Self(Rc::new(RefCell::new(f)))
//     }
// }

// impl Debug for BuiltinClosure {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "BuiltinClosure at {:?}", Rc::as_ptr(&self.0))
//     }
// }

// impl PartialEq for BuiltinClosure {
//     #[allow(clippy::vtable_address_comparisons)]
//     fn eq(&self, other: &Self) -> bool {
//         Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
//     }
// }

// impl Hash for BuiltinClosure {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         Rc::as_ptr(&self.0).hash(state);
//     }
// }

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
                $(
                    (
                        $tuple_typ:ty
                    )
                )?
                $(
                    {
                        $(
                            $(#[$($field_meta:meta)*] )?
                            $n:ident: $t1:ty ,
                        )*
                    }
                )?
            ,
        )*

        => $i_name:ident { $( $i_field:ident: $i_typ:ty ,)* }
        ,
    ) => {
        #[derive(Debug, Clone, PartialEq, Default, allow_until::AllowUntil)]
        pub enum Value {
            $(
                $(#[$($meta)*])?
                $name
                $(
                    (
                        $tuple_typ
                    )
                )?
                $(
                    {
                        $(
                            $(#[$($field_meta)*] )?
                            $n: $t1 ,
                        )*
                    }
                )?,
            )*
            $i_name { $( $i_field: $i_typ ,)* },
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

            #[derive(Debug, PartialEq, Clone)]
            pub struct FieldGetter<'a, T, const MUT: bool> {
                value_ref: &'a ValueRef,
                getter: fn(&'a ValueRef) -> std::cell::Ref<'a, T>,
                getter_mut: fn(&'a ValueRef) -> std::cell::RefMut<'a, T>,
            }

            impl<'a, T, const MUT: bool> FieldGetter<'a, T, MUT> {
                pub fn parent_area(&'a self) -> CodeArea {
                    self.value_ref.borrow().area.clone()
                }
                pub fn get_ref(&'a self) -> &'a ValueRef {
                    self.value_ref
                }
                pub fn borrow(&'a self) -> std::cell::Ref<'a, T> {
                    (self.getter)(self.value_ref)
                }
            }

            impl<'a, T> FieldGetter<'a, T, true> {
                pub fn borrow_mut(&'a self) -> std::cell::RefMut<'a, T> {
                    (self.getter_mut)(self.value_ref)
                }
            }


            paste::paste! {
                $(
                    value! {
                        @struct [<$name Getter>]<'a, const MUT: bool>
                        $(
                            (
                                FieldGetter<'a, $tuple_typ, MUT>,
                            )
                        )?
                        $(
                            {
                                $( $n: FieldGetter<'a, $t1, MUT>, )*
                            }
                        )?
                    }

                    $(
                        impl<'a, const MUT: bool> [<$name Getter>]<'a, MUT> {
                            pub fn area(&'a self) -> CodeArea {
                                stringify!($tuple_typ);
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
                            pub fn get_ref(&'a self) -> &'a ValueRef {
                                self.0.get_ref()
                            }
                        }
                    )?

                    $(
                        impl<'a, const MUT: bool> [<$name Getter>]<'a, MUT> {
                            #[allow(unreachable_code)]
                            pub fn area(&'a self) -> CodeArea {
                                $(
                                    return self.$n.parent_area();
                                )*
                            }
                            #[allow(unreachable_code)]
                            pub fn get_ref(&'a self) -> &'a ValueRef {
                                $(
                                    return self.$n.get_ref();
                                )*
                            }
                        }
                    )?

                    impl<'a, const MUT: bool> [<$name Getter>]<'a, MUT> {
                        pub const fn make_from(vref: &'a ValueRef) -> Self {
                            value! { @make
                                Self
                                $(
                                    (
                                        FieldGetter::<'a, $tuple_typ, MUT> {
                                            value_ref: vref,
                                            getter: |vr: &ValueRef| {
                                                std::cell::Ref::map(vr.borrow(), |v| {
                                                    match &v.value {
                                                        Value::$name(field) => field,
                                                        _ => panic!("wrong GUNGLY TYPE DUM BASS")
                                                    }
                                                })
                                            },
                                            getter_mut: |vr: &ValueRef| {
                                                std::cell::RefMut::map(vr.borrow_mut(), |v| {
                                                    match &mut v.value {
                                                        Value::$name(field) => field,
                                                        _ => panic!("wrong GUNGLY TYPE DUM BASS")
                                                    }
                                                })
                                            }
                                        }
                                    )
                                )?
                                $(
                                    {
                                        $(
                                            $n: FieldGetter::<'a, $t1, MUT> {
                                                value_ref: vref,
                                                getter: |vr: &ValueRef| {
                                                    std::cell::Ref::map(vr.borrow(), |v| {
                                                        match &v.value {
                                                            Value::$name { $n, .. } => $n,
                                                            _ => panic!("wrong GUNGLY TYPE DUM BASS")
                                                        }
                                                    })
                                                },
                                                getter_mut: |vr: &ValueRef| {
                                                    std::cell::RefMut::map(vr.borrow_mut(), |v| {
                                                        match &mut v.value {
                                                            Value::$name { $n, .. } => $n,
                                                            _ => panic!("wrong GUNGLY TYPE DUM BASS")
                                                        }
                                                    })
                                                }
                                            },
                                        )*
                                    }
                                )?
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
                        _ => None,
                    }
                }
            }
        }
    };

    // required cause structs with curly braces cant have semicolons
    (@struct $name:ident <$lt:lifetime $(, const $const_generic:ident : $ty:ty)?>) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name <$lt $(, const $const_generic : $ty)?> (std::marker::PhantomData<&$lt ()>);
    };
    (@struct $name:ident <$lt:lifetime $(, const $const_generic:ident : $ty:ty)?> ( $( $t0:ty, )* )) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name <$lt $(, const $const_generic : $ty)?> ( $( pub $t0, )* );
    };
    (@struct $name:ident <$lt:lifetime $(, const $const_generic:ident : $ty:ty)?> { $( $n:ident: $t1:ty, )* }) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name <$lt $(, const $const_generic : $ty)?> { $( pub $n: $t1, )* }
    };

    (@make $name:ident) => {
        $name(std::marker::PhantomData)
    };
    (@make $name:ident $($t:tt)*) => {
        $name $($t)*
    };
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum MacroData {
//     Spwn {
//         func: FuncCoord,

//         args: ImmutVec<Spanned<MacroArg<ValueRef, ()>>>,
//         self_arg: Option<ValueRef>,

//         captured: ImmutVec<ValueRef>,

//         is_method: bool,

//         is_builtin: Option<BuiltinFn>,
//     },
//     FullyRust {
//         func: BuiltinFn,
//         args: ImmutVec<Spanned<MacroArg<ValueRef, ()>>>,
//     }
// }

#[derive(Debug, Clone)]
pub enum MacroTarget {
    Spwn {
        func: FuncCoord,
        is_builtin: Option<BuiltinFn>,
        captured: ImmutVec<ValueRef>,
    },
    FullyRust {
        fn_ptr: Rc<RefCell<dyn BuiltinClosure>>,
        args: ImmutVec<ImmutStr>,
        spread_arg: Option<u8>,
    },
}

impl PartialEq for MacroTarget {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Spwn {
                    func: l_func,
                    is_builtin: l_is_builtin,
                    captured: l_captured,
                },
                Self::Spwn {
                    func: r_func,
                    is_builtin: r_is_builtin,
                    captured: r_captured,
                },
            ) => l_func == r_func && l_is_builtin == r_is_builtin && l_captured == r_captured,
            (
                Self::FullyRust {
                    fn_ptr: l_fn_ptr,
                    args: l_args,
                    spread_arg: l_spread_arg,
                },
                Self::FullyRust {
                    fn_ptr: r_fn_ptr,
                    args: r_args,
                    spread_arg: r_spread_arg,
                },
            ) => Rc::ptr_eq(l_fn_ptr, r_fn_ptr) && l_args == r_args && l_spread_arg == r_spread_arg,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroData {
    pub target: MacroTarget,

    pub defaults: ImmutVec<Option<ValueRef>>,
    pub self_arg: Option<ValueRef>,

    pub is_method: bool,
    pub is_unsafe: bool,
}

value! {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ImmutCloneStr32),

    Array(Vec<ValueRef>),
    Dict(AHashMap<ImmutCloneStr32, VisSource<ValueRef>>),

    Group(Id),
    Channel(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range {
        start: i64,
        end: i64,
        step: i64,
    }, //start, end, step

    Maybe(Option<ValueRef>),

    #[default]
    Empty,

    Macro(MacroData),
    Iterator(MacroData),

    Type(ValueType),

    Module {
        exports: AHashMap<ImmutCloneStr32, ValueRef>,
        types: Vec<Vis<CustomTypeID>>,
    },

    TriggerFunction {
        group: Id,
        prev_context: Id,
    },

    Error(usize),

    Object {
        params: AHashMap<u8, ObjParam>,
        typ: ObjectType,
    },
    ObjectKey(ObjectKey),

    Epsilon,

    Chroma {
        r: u8, g: u8, b: u8, a: u8,
    },

    => Instance {
        typ: CustomTypeID,
        items: AHashMap<ImmutCloneStr32, VisSource<ValueRef>>,
    },
}

impl ValueType {
    pub fn runtime_display(self, vm: &Vm) -> String {
        format!(
            "@{}",
            match self {
                Self::Custom(t) => vm.type_def_map[&t].name.as_ref().to_string(),
                _ => <ValueType as Into<&str>>::into(self).into(),
            }
        )
    }
}

impl Value {
    pub fn from_const(vm: &mut Vm, c: &Constant, area: CodeArea) -> Value {
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
            Constant::Array(arr) => Value::Array(
                arr.iter()
                    .map(|c| {
                        let value = Value::from_const(vm, c, area.clone());
                        value.into_value_ref(area.clone())
                    })
                    .collect(),
            ),
            Constant::Dict(m) => Value::Dict(
                m.iter()
                    .map(|(s, c)| {
                        let value = Value::from_const(vm, c, area.clone());
                        // let g = s.into_utfstring();
                        (
                            s.clone().into_utfstring().into(),
                            VisSource::Public(value.into_value_ref(area.clone())),
                        )
                    })
                    .collect(),
            ),
            Constant::Maybe(o) => Value::Maybe(o.clone().map(|c| {
                let value = Value::from_const(vm, &c, area.clone());
                value.into_value_ref(area.clone())
            })),
            Constant::Builtins => Value::Builtins,
            Constant::Empty => Value::Empty,
            Constant::Instance { typ, items } => Value::Instance {
                typ: *typ,
                items: items
                    .iter()
                    .map(|(s, c)| {
                        let value = Value::from_const(vm, c, area.clone());
                        (
                            s.clone().into_utfstring().into(),
                            VisSource::Public(value.into_value_ref(area.clone())),
                        )
                    })
                    .collect(),
            },
        }
    }

    pub fn into_stored(self, area: CodeArea) -> StoredValue {
        StoredValue { value: self, area }
    }

    pub fn into_value_ref(self, area: CodeArea) -> ValueRef {
        ValueRef::new(self.into_stored(area))
    }
}
