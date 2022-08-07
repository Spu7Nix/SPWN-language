use std::any::Any;
use std::marker::PhantomData;

use ahash::AHashMap;

use super::{
    interpreter::ValueKey,
    value::{Macro, Value},
};

//* Type:
// all send + sync
// from struct?
// constructor
// fields
// methods (static + instance separate)?
// type checking
// default values
// all methods with areas and globals

pub struct Type {
    pub name: String,
    pub members: AHashMap<String, ValueKey>,
}

pub struct Instance {
    pub ty: Type,
    pub fields: AHashMap<String, ValueKey>,
}

// pub struct TypeBuilder<T> {
//     name: &'static str,
//     members: AHashMap<String, ValueKey>,
//     phantom: PhantomData<T>,
// }

// impl<T> TypeBuilder<T> {
//     pub fn named(name: &'static str) -> Self {
//         Self {
//             name,
//             members: AHashMap::new(),
//             phantom: PhantomData,
//         }
//     }
// }
