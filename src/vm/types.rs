use ahash::AHashMap;

use super::context::FullContext;
use super::error::RuntimeError;
use super::interpreter::{Globals, TypeMember};
use super::value::ValueType;
use super::{
    interpreter::{TypeKey, ValueKey},
    value::{Macro, Value},
};

#[derive(Clone)]
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
pub type MethodFunction =
    fn(&mut Globals, &mut FullContext, ValueKey, &[ValueKey]) -> Result<Value, RuntimeError>;

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

// this macro is acting as pretty much a match expression but for the tokens
// each "variant" is a kind of argument (mutable, non mutable, no type, etc)
#[macro_export]
macro_rules! method_arg_type {
    // mutable, no type specified
    // ex: `..., mut el => {}`
    (
        //globals..key.......args......count
        [$g:ident, $k:ident, $a:ident, $c:ident]
        mut $arg:ident@
    ) => {
        let $arg = &mut $g.memory[$a[$c]].value;
    };

    // mutable, 1 or more types specified
    // ex: `..., mut el: A => {}`
    // ex: `..., mut el: A | B | C => {}`
    (
        [$g:ident, $k:ident, $a:ident, $c:ident]
        mut $arg:ident@$($argty:ty)|*
    ) => {};

    // non mutable, no type specified
    // ex: `..., el => {}`
    (
        [$g:ident, $k:ident, $a:ident, $c:ident]
        $arg:ident@
    ) => {
        let $arg = &$g.memory[$a[$c]].value;
    };

    // non mutable, 1 or more types specified
    // ex: `..., el: A => {}`
    // ex: `..., el: A | B | C => {}`
    (
        [$g:ident, $k:ident, $a:ident, $c:ident]
        $arg:ident@$($argty:ty)|*
    ) => {
        // TODO: multiple types
        // TODO: fix references
        $(
            let $arg: &$argty = $crate::vm::from_value::FromValue::from_value(&$g.memory[$a[$c]].value)?;
        )*
    };

    // "reference" (ValueKey)
    // ex: `..., ref el => {}`
    (
        //.............................count
        [$g:ident, $k:ident, $a:ident, $c:ident]
        ref $arg:ident@
    ) => {
        let $arg = $a[$c];
    };
}

// the different variations of arguments
// all start with some form of `this`
#[macro_export]
macro_rules! method_args {
    // |mut this: X| { }
    //  ^
    {
        //globals..key.......args
        [$g:ident, $c:ident, $k:ident, $a:ident]
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let $this: &mut $typ = $crate::vm::from_value::FromValue::from_value_mut(&mut $globals.memory[$k].value)?;
        $body
    }};

    // this: X => { }
    {
        [$g:ident, $c:ident, $k:ident, $a:ident]
        $globals:ident, |$this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let $this: $typ = $crate::vm::from_value::FromValue::from_value($a[0])?;
        $body
    }};

    // |mut this: X, mut y: Y, z: Z| { }
    //  ^
    {
        [$g:ident, $c:ident, $k:ident, $a:ident]
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ty )|*
                )?
            ),+
        | $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            let $this: &mut $typ = $crate::vm::from_value::FromValue::from_value_mut(&mut $globals.memory[$k].value)?;

            let mut count = 0;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                //return Err($crate::vm::error::RuntimeError::)
            }
            if count > $a.len() {
                // return Err($crate::vm::error::RuntimeError::TooManyArguments {
                //     expected: $a.len(),
                //     provided: count,
                //     call_area: ,
                //     func_area: $crate::sources::CodeArea::internal(),
                // })
            }

            // reset count so now we can increment again to get arg index
            count = 0;

            $(
                $crate::method_arg_type!(
                    [$globals, $k, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };

    // |this: X, mut y: Y, z: Z| { }
    {
        [$g:ident, $c:ident, $k:ident, $a:ident]
        $globals:ident, $context:ident, |$this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ty )|*
                )?
            ),+
        | $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            let $this: $typ = $crate::vm::from_value::FromValue::from_value($globals.memory[$k].value)?;

            let mut count = 0;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                return Err(todo!())
            }
            if count > $a.len() {
                return Err(todo!())
            }

            // reset count so now we can increment again to get arg index
            count = 0;

            $(
                $crate::method_arg_type!(
                    [$globals, $k, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };

    // |mut y: Y, z: Z| { }
    {
        [$g:ident, $c:ident, $k:ident, $a:ident]
        $globals:ident, $context:ident,
        |$(
            $($arg:ident)+
            $(
                : $( $argty:ty )|*
            )?
        ),+| $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            let mut count = 0;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                return Err(todo!())
            }
            if count > $a.len() {
                return Err(todo!())
            }

            // reset count so now we can increment again to get arg index
            count = 0;

            $(
                $crate::method_arg_type!(
                    [$globals, $k, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };
}

#[macro_export]
macro_rules! method {
    // capture all tokens as "base" macro, just to generate closure
    ($($all:tt)+) => {
        |globals, context, _key, _args| {
            $crate::vm::to_value::ToValueResult::try_to_value(
                $crate::method_args!(
                    [globals, context, _key, _args]
                    $($all)+
                )
            )
        }
    };
}
