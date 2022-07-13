use std::marker::PhantomData;
use std::any::Any;

use ahash::AHashMap;

use super::value::{Macro, Value};

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
    constructor: Option<Macro>,
    methods: AHashMap<String, Macro>
}

impl Type {

}

pub struct Instance {
    ty: Type,
    fields: AHashMap<String, Value>
}

pub struct TypeBuilder<T> 
where
    T: Any + Send + Sync
{
    phantom: PhantomData<T>,
}
