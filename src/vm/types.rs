use std::marker::PhantomData;
use std::{any::Any, sync::Arc};

use ahash::AHashMap;

use super::error::RuntimeError;
use super::interpreter::Globals;
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

pub type AttributeFunction<T, R> = fn(&mut Globals, &T) -> R;
pub type MethodFunction = fn(&mut Globals, &[ValueKey]) -> Result<ValueKey, RuntimeError>;

// pub struct BuiltinFunction {
//     func: BuiltinPtr,
//     is_method: bool, // takes self as first argument if true
//                      // arguments and shit
// }

pub type BuiltinFunctions = Arc<dyn Any>;

pub struct TypeBuilder<T> {
    typ: ValueType,
    members: AHashMap<String, ValueKey>,
    phantom: PhantomData<T>,
}

impl<T> TypeBuilder<T>
where
    T: 'static,
{
    pub fn new(typ: ValueType) -> Self {
        Self {
            typ,
            members: AHashMap::new(),
            phantom: PhantomData,
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

    fn add(&mut self, globals: &mut Globals, name: &'static str, v: Value) {
        let v = v.into_stored(CodeArea::internal());
        let k = globals.memory.insert(v);
        self.members.insert(name.into(), k);
    }

    pub fn add_member(
        mut self,
        globals: &mut Globals,
        name: &'static str,
        f: AttributeFunction<T, impl ToValueResult + 'static>,
    ) -> Self {
        let key = globals.builtins.insert(Arc::new(f));
        let v = Value::Macro(Macro::Builtin { func_ptr: key });

        self.add(globals, name, v);

        self
    }

    pub fn add_method(
        mut self,
        globals: &mut Globals,
        name: &'static str,
        f: MethodFunction,
    ) -> Self {
        let key = globals.builtins.insert(Arc::new(f));
        let v = Value::Macro(Macro::Builtin { func_ptr: key });

        self.add(globals, name, v);

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
macro_rules! method {
    {
        $name:ident
        (
            $(
                $args:tt
            )*
        ) => $body:expr
    } => {(
        stringify!($name:ident),
        |globals, args| {
            let mut args = args;

            method!($($args)*);
            // $(
            //     let $arg = args.remove(0).unwrap();
            //     // pattern check
            // )*
            // $body
        })
    };

    {
        this,
        $(
            $args:ident
            $(:
                $(mut)?
                $(
                    $type:ident
                )|*
            )?
        ),*
    } => {
        let this = globals.memory[args[0]].expect_value_type(ValueType::Instance);

    };

    {
        mut this,
        $(
            $args:ident
            $(:
                $(mut)?
                $(
                    $type:ident
                )|*
            )?
        ),*
    } => {

    }
}
