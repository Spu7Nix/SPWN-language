use std::marker::PhantomData;
use std::{any::Any, sync::Arc};

use ahash::AHashMap;

use super::error::RuntimeError;
use super::interpreter::{Globals, TypeMember};
use super::to_value::ToValueResult;
use super::value::ValueType;
use super::{
    interpreter::{TypeKey, ValueKey},
    value::{Macro, Value},
};
use crate::sources::CodeArea;

pub struct CustomType {
    pub name: String,
    //pub members: AHashMap<String, ValueKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub typ: TypeKey,
    pub fields: AHashMap<String, ValueKey>,
}

pub type AttributeFunction = fn(&Globals, ValueKey) -> Value;
pub type MethodFunction = fn(&mut Globals, &[ValueKey]) -> Result<ValueKey, RuntimeError>;

// pub struct BuiltinFunction {
//     func: BuiltinPtr,
//     is_method: bool, // takes self as first argument if true
//                      // arguments and shit
// }

pub enum BuiltinFunction {
    Attribute(AttributeFunction),
    Method(MethodFunction),
}

pub struct TypeBuilder {
    typ: ValueType,
    members: AHashMap<String, TypeMember>,
}

impl TypeBuilder {
    pub fn new(typ: ValueType) -> Self {
        Self {
            typ,
            members: AHashMap::new(),
        }
    }

    // pub fn add_member<V>(mut self, globals: &mut Globals, name: &'static str, v: V) -> Self
    // where
    //     V: ToValue,
    // {
    //     let v = v.to_value().into_stored(CodeArea::unknown());
    //     let k = globals.memory.insert(v);
    //     self.members.insert(name.into(), k);
    //     self
    // }

    pub fn add_member(
        mut self,
        globals: &mut Globals,
        name: &'static str,
        f: AttributeFunction,
    ) -> Self {
        let key = globals.builtins.insert(BuiltinFunction::Attribute(f));

        self.members.insert(name.into(), TypeMember::Builtin(key));

        self
    }

    pub fn add_method(
        mut self,
        globals: &mut Globals,
        name: &'static str,
        f: MethodFunction,
    ) -> Self {
        let key = globals.builtins.insert(BuiltinFunction::Method(f));

        self.members.insert(name.into(), TypeMember::Builtin(key));

        self
    }

    pub fn finish_type(self, globals: &mut Globals) {
        globals
            .type_members
            .entry(self.typ)
            .or_default()
            .extend(self.members);

        // let k = globals.types.insert(CustomType {
        //     name: self.typ.to_str(globals).into(),
        //     members: self.members,
        // });

        // globals.type_keys.insert(self.typ.to_str(globals).into(), k);
    }
}

#[macro_export]
macro_rules! attr {
    (
        $globals:ident, $this:pat => $body:expr
    ) => {
        |$globals: &Globals, this: ValueKey| -> Value {
            let val = &$globals.memory[this].value;
            match val {
                $this => ToValueResult::try_to_value($body).unwrap(),
                _ => unreachable!(),
            }
        }
    };
}

/*

attr! {
    globals, Value::Array(this) => this.len()
}
*/

#[macro_export]
macro_rules! method_arg {
    ($globals:ident, $args:ident) => {
        &$globals.memory[*$args.next().unwrap()].value
    };

    (mut $globals:ident, $args:ident) => {
        &mut $globals.memory[*$args.next().unwrap()].value
    };

    (key $globals:ident, $args:ident) => {
        *$args.next().unwrap()
    };
}

#[macro_export]
macro_rules! method {
    {
        $globals:ident,
        $(
            $(#$mut:ident)? $arg:pat
        ),*
        => $body:expr
    } => {
        |$globals, args| {
            let mut args = args.iter().rev();

            match ($(method_arg!($($mut)? $globals, args)),*){
                ($($arg),*) => Ok({
                    let a = ToValueResult::try_to_value($body).unwrap();
                    $globals.memory.insert(a.into_stored(CodeArea::internal()))
                }),
                _ => return Err(todo!()),
            }
        }
    };
}
