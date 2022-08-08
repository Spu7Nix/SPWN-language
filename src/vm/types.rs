use std::marker::PhantomData;
use std::{any::Any, sync::Arc};

use ahash::AHashMap;
use slotmap::SlotMap;

use crate::sources::CodeArea;

use super::error::RuntimeError;

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

// type @group

pub struct CustomType {
    pub name: String,
    //pub members: AHashMap<String, ValueKey>,
}

// impl CustomType {
//     pub fn get_member(&self, name: String) -> ValueKey {
//         self.members[&name]
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub typ: TypeKey,
    pub fields: AHashMap<String, ValueKey>,
}

type BuiltinPtr = fn(&mut Globals, &[ValueKey]) -> Result<ValueKey, RuntimeError>;

pub struct BuiltinFunction {
    func: BuiltinPtr,
    is_method: bool, // takes self as first argument if true
                     // arguments and shit
}

pub struct TypeBuilder {
    typ: ValueType,
    members: AHashMap<String, ValueKey>,
}

impl TypeBuilder {
    pub fn new(typ: ValueType) -> Self {
        Self {
            typ,
            members: AHashMap::new(),
        }
    }

    pub fn add_member<V>(mut self, globals: &mut Globals, name: &'static str, v: V) -> Self
    where
        V: ToValue,
    {
        let v = v.to_value().into_stored(CodeArea::unknown());
        let k = globals.memory.insert(v);
        self.members.insert(name.into(), k);
        self
    }

    pub unsafe fn add_method(
        self,
        globals: &mut Globals,
        name: &'static str,
        f: BuiltinPtr,
    ) -> Self {
        let key = globals.builtins.insert(f);

        let v = Value::Macro(Macro::Builtin { func_ptr: key });
        self.add_member(globals, name, v)
    }

    pub fn finish_type(self, globals: &mut Globals) {
        globals
            .type_members
            .entry(self.typ)
            .or_default()
            .extend(self.members);
    }
}

#[macro_export]
macro_rules! method {
    {
        $name:ident
        ($($arg:ident: $pat:expr),*) => $body:expr
    } => {
        stringify!($name:ident), BuiltinFunction {
            func: |globals, args| {
                let mut args = args;
                $(
                    let $arg = args.remove(0).unwrap();
                    // pattern check
                )*
                $body
            },
            is_method: true,
        }
    };
}
