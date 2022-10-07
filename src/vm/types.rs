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

pub type AttributeFunction = fn(&Globals, &mut FullContext, ValueKey) -> Value;

#[derive(Clone, Copy)]
pub struct MethodFunction {
    pub func: fn(&mut Globals, &mut FullContext, &[ValueKey]) -> Result<Value, RuntimeError>,
    pub arg_count: usize,
    pub is_static: bool,
    // arg types, etc
}

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

        globals
            .builtins_by_name
            .insert(format!("{}::{}", self.typ.to_str(globals), name), key);

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

        globals
            .builtins_by_name
            .insert(format!("{}::{}", self.typ.to_str(globals), name), key);

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

// this macro is acting as pretty much a match expression but for the tokens
// each "variant" is a kind of argument (mutable, non mutable, no type, etc)
#[macro_export]
macro_rules! method_arg_type {
    // mutable, no type specified
    // ex: `..., mut el => {}`
    (
        //globals..key.......args......count
        [$g:ident, $a:ident, $c:ident]
        mut $arg:ident@
    ) => {
        let $arg = &mut $g.memory[$a[$c]].value;
    };

    // mutable, 1 or more types specified
    // ex: `..., mut el: A => {}`
    // ex: `..., mut el: A | B | C => {}`
    (
        [$g:ident, $a:ident, $c:ident]
        mut $arg:ident@$($argty:ident)|*
    ) => {
        let v = &mut $g.memory[$a[$c]].value;

        $crate::method_arg_type!(@mut@ $arg v $($argty,)*);
    };

    // non mutable, no type specified
    // ex: `..., el => {}`
    (
        [$g:ident, $a:ident, $c:ident]
        $arg:ident@
    ) => {
        let $arg = &$g.memory[$a[$c]].value;
    };

    // non mutable, 1 or more types specified
    // ex: `..., el: A => {}`
    // ex: `..., el: A | B | C => {}`
    (
        [$g:ident, $a:ident, $c:ident]
        $arg:ident@$($argty:ident)|*
    ) => {
        let v = &$g.memory[$a[$c]].value;

        $crate::method_arg_type!(@@$arg v $($argty,)*);
    };

    // "reference" (ValueKey)
    // ex: `..., ref el => {}`
    (
        //.............................count
        [$g:ident, $a:ident, $c:ident]
        ref $arg:ident@
    ) => {
        let $arg = $a[$c];
    };

    //////////////////////////////////////////////////////

    // multiple types type checking
    (@$($mut:ident)?@ $arg:ident $v:ident $t1:ident, $($trest:ident),+,) => {
        if !(match $v {
            $crate::vm::value::Value::$t1(..) => true,
            $(
                $crate::vm::value::Value::$trest(..) => true,
            )*
            _ => false,
        }) {
            // TODO: error
        }

        let $arg = &$($mut)?*$v;
    };

    // single type type checking
    (@@ $arg:ident $v:ident $type1:ty,) => {
        let $arg: &$type1 = $crate::vm::from_value::FromValue::from_value($v)?;
    };

    // single mut type type checking
    (@mut@ $arg:ident $v:ident $type1:ty,) => {
        let $arg: &mut $type1 = $crate::vm::from_value::FromValue::from_value_mut($v)?;
    };
}

// the different variations of arguments
// all start with some form of `this`
#[macro_export]
macro_rules! method_args {
    // |mut this: X| { }
    //  ^
    {
        //globals...context...key.......args
        [$g:ident, $c:ident, $a:ident]
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let slf = $a.get(0).unwrap();//.map_err(|_| ) // TODO: error (missing self)

        let $this: &mut $typ = $crate::vm::from_value::FromValue::from_value_mut(&mut $globals.memory[*slf].value)?;
        $body
    }};

    // this: X => { }
    {
        [$g:ident, $c:ident, $a:ident]
        $globals:ident, $context:ident, |$this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let slf = $a.get(0).unwrap();//.map_err(|_| ) // TODO: error (missing self)

        let $this: &$typ = $crate::vm::from_value::FromValue::from_value(&$globals.memory[*slf].value)?;
        $body
    }};

    // |mut this: X, mut y: Y, z: Z| { }
    //  ^
    {
        [$g:ident, $c:ident, $a:ident]
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ident )|*
                )?
            ),+
        | $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            let slf = $a.get(0).unwrap();//.map_err(|_| ) // TODO: error (missing self)

            let $this: &mut $typ = $crate::vm::from_value::FromValue::from_value_mut(&mut $globals.memory[*slf].value)?;

            let mut count = 0;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                // TODO: error
            }
            if count > $a.len() {
                // TODO: error
            }

            // reset count so now we can increment again to get arg index
            count = 1;

            $(
                $crate::method_arg_type!(
                    [$globals, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };

    // |this: X, mut y: Y, z: Z| { }
    {
        [$g:ident, $c:ident, $a:ident]
        $globals:ident, $context:ident,
        |$this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ident )|*
                )?
            ),+
        | $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            let slf = $a.get(0).unwrap();//.map_err(|_| ) // TODO: error (missing self)

            let $this: $typ = $crate::vm::from_value::FromValue::from_value($globals.memory[*slf].value)?;

            let mut count = 0;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                // TODO: error
            }
            if count > $a.len() {
                // TODO: error
            }

            // reset count so now we can increment again to get arg index
            count = 1;

            $(
                $crate::method_arg_type!(
                    [$globals, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };

    // |mut y: Y, z: Z| { }
    {
        [$g:ident, $c:ident, $a:ident]
        $globals:ident, $context:ident,
        |$(
            $($arg:ident)+
            $(
                : $( $argty:ident )|*
            )?
        ),+| $body:block
    } => {
        #[allow(unused_assignments)]
        {
            let $globals = $g;
            let $context = $c;

            // skip first argument ("self" argument, but since this is a static method it doesnt actually point to anything useful)
            let mut count = 1;
            $(
                // useless stringify to get the token to repeat so count increases
                stringify!($($arg)*,);
                count += 1;
            )+

            if count < $a.len() {
                // TODO: error
            }
            if count > $a.len() {
                // TODO: error
            }

            // skip first argument ("self" argument, but since this is a static method it doesnt actually point to anything useful)
            count = 1;

            $(
                $crate::method_arg_type!(
                    [$globals, $a, count]
                    $($arg)+ @ $($($argty)|*)?
                );
                count += 1;
            )+

            $body
        }
    };
}

#[macro_export]
macro_rules! count_args {
    {
        $globals:ident, $context:ident, |
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ty )|*
                )?
            ),+
        | $body:block
    } => {
        {let mut count = 0;
        $(
            stringify!($($arg)+,);
            count += 1;
        )+
        count}
    }

}

#[macro_export]
macro_rules! is_static {

    {
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty| $body:block
    } => {false};

    {
        $globals:ident, $context:ident, |$this:ident: $typ:ty| $body:block
    } => {false};

    {
        $globals:ident, $context:ident,
        |$this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ident )|*
                )?
            ),+
        | $body:block
    } => { false };

    {
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty,
            $(
                $($arg:ident)+
                $(
                    : $( $argty:ident )|*
                )?
            ),+
        | $body:block
    } => { false };

    {
        $globals:ident, $context:ident,
        |$(
            $($arg:ident)+
            $(
                : $( $argty:ident )|*
            )?
        ),+| $body:block
    } => { true }

}

#[macro_export]
macro_rules! method {
    // capture all tokens as "base" macro, just to generate closure
    ($($all:tt)+) => {
        MethodFunction {
            func: |globals, context, args| {
                $crate::vm::to_value::ToValueResult::try_to_value(
                    $crate::method_args!(
                        [globals, context, args]
                        $($all)+
                    )
                )
            },
            arg_count: $crate::count_args!($($all)+),
            is_static: $crate::is_static!($($all)+),
        }
    };
}

#[macro_export]
macro_rules! attr_args {
    // |mut this: X| { }
    //  ^
    {
        //globals...context...key.......args
        [$g:ident, $c:ident, $k:ident]
        $globals:ident, $context:ident, |mut $this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let $this: &mut $typ = $crate::vm::from_value::FromValue::from_value_mut(&mut $globals.memory[$k].value).unwrap(); // shouldnt be possible to error here
        $body
    }};

    // this: X => { }
    {
        [$g:ident, $c:ident, $k:ident]
        $globals:ident, $context:ident, |$this:ident: $typ:ty| $body:block
    } => {{
        let $globals = $g;
        let $context = $c;

        let $this: &$typ = $crate::vm::from_value::FromValue::from_value(&$globals.memory[$k].value).unwrap();
        $body
    }};
}

#[macro_export]
macro_rules! attr {
    // capture all tokens as "base" macro, just to generate closure
    ($($all:tt)+) => {
        |globals, context, key| {
            $crate::vm::to_value::ToValue::to_value(
                $crate::attr_args!(
                    [globals, context, key]
                    $($all)+
                )
            )
        }
    };
}
