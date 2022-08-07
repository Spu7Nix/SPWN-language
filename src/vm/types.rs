use std::marker::PhantomData;
use std::{any::Any, sync::Arc};

use ahash::AHashMap;

use super::from_value::FromValueList;
use super::interpreter::Globals;
use super::value::ValueType;
use super::{
    interpreter::{TypeKey, ValueKey},
    value::{Macro, Value},
};

use super::to_value::{ToValue, ToValueResult};

//* Type:
// all send + sync
// from struct?
// constructor
// fields
// methods (static + instance separate)?
// type checking
// default values
// all methods with areas and globals

pub struct CustomType {
    pub name: String,
    pub members: AHashMap<String, ValueKey>,
    //internal: Option<&'a mut BuiltinType>,
}

impl CustomType {
    pub fn get_member(&self, name: String) -> ValueKey {
        self.members[name]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub typ: TypeKey,
    pub fields: AHashMap<String, ValueKey>,
}

pub struct BuiltinType;

// pub struct RustType<'a, T, R1, R2, A>
// where
//     R1: ToValue,
//     R2: ToValueResult,
//     A: FromValueList,
// {
//     globals: &'a mut Globals,
//     members: AHashMap<String, Arc<dyn Fn(&T, &mut Globals) -> R1>>,
//     methods: AHashMap<String, Arc<dyn Fn(A, &mut Globals) -> R2>>,
// }

// impl RustType {
//     pub fn new() -> Self {}

//     pub fn get_member() {}
//     pub fn call_method() {}
// }

pub struct TypeBuilder<T, R1, R2, A>
where
    R1: ToValue,
    R2: ToValueResult,
    A: FromValueList,
{
    name: &'static str,
    members: AHashMap<String, Arc<dyn Fn(&T, &mut Globals) -> R1>>,
    methods: AHashMap<String, Arc<dyn Fn(A, &mut Globals) -> R2>>,
    phantom: PhantomData<T>,
}

impl<T, R1, R2, A> TypeBuilder<T, R1, R2, A>
where
    R1: ToValue,
    R2: ToValueResult,
    A: FromValueList,
{
    pub fn named(name: &'static str) -> Self {
        Self {
            name,
            members: AHashMap::new(),
            methods: AHashMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn add_member<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn(&T, &mut Globals) -> R1 + 'static,
    {
        self.members.insert(name.into(), Arc::new(f));
    }

    pub fn add_method<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn(A, &mut Globals) -> R2 + 'static,
    {
        self.methods.insert(name.into(), Arc::new(f));
    }

    // pub fn finish_type(&self, globals: &mut Globals) -> CustomType {
    //     CustomType {
    //         name: self.name,
    //         members: AHashMap::new(),
    //     }
    // }
}
