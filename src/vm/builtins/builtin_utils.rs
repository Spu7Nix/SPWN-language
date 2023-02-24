use std::marker::PhantomData;

use crate::sources::CodeArea;
use crate::vm::interpreter::{RuntimeResult, ValueKey, Vm};
use crate::vm::value::Value;

pub trait Invoke<const N: usize, Args = ()> {
    fn invoke(&self, args: Vec<ValueKey>, vm: &mut Vm, area: CodeArea) -> RuntimeResult<Value>;
}

pub trait IntoArg<O> {
    fn into_arg(self, vm: &mut Vm) -> O;
}

pub trait GetMutRefArg<'a> {
    type Output;

    fn get_mut_ref_arg(key: ValueKey, vm: &'a mut Vm) -> Self::Output;
}
pub trait GetRefArg<'a> {
    type Output;

    fn get_ref_arg(key: ValueKey, vm: &'a Vm) -> Self::Output;
}

pub struct Or<T>(T);
pub struct Spread<T>(Vec<T>);

impl<T> std::ops::Deref for Spread<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct Ref<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> {
    key: ValueKey,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> Ref<'a, T> {
    pub fn get_mut_ref(&self, vm: &'a mut Vm) -> <T as GetMutRefArg<'a>>::Output {
        T::get_mut_ref_arg(self.key, vm)
    }

    pub fn get_ref(&self, vm: &'a Vm) -> <T as GetRefArg<'a>>::Output {
        T::get_ref_arg(self.key, vm)
    }
}

////////////////////

impl<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> IntoArg<Ref<'a, T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, _: &mut Vm) -> Ref<'a, T> {
        Ref {
            key: self,
            _phantom: PhantomData,
        }
    }
}

impl<T> IntoArg<Spread<T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, vm: &mut Vm) -> Spread<T> {
        match &vm.memory[self].value {
            Value::Array(arr) => Spread(arr.clone().iter().map(|k| k.into_arg(vm)).collect()),
            _ => unreachable!(),
        }
    }
}

impl IntoArg<Value> for ValueKey {
    fn into_arg(self, vm: &mut Vm) -> Value {
        vm.memory[self].value.clone()
    }
}

impl IntoArg<ValueKey> for ValueKey {
    fn into_arg(self, _: &mut Vm) -> ValueKey {
        self
    }
}

impl IntoArg<CodeArea> for ValueKey {
    fn into_arg(self, vm: &mut Vm) -> CodeArea {
        vm.memory[self].area.clone()
    }
}

macro_rules! tuple_macro {
    (@gen $($ident:ident)*) => {

        impl<$($ident,)*> IntoArg<Or<($(Option<$ident>,)*)>> for ValueKey
        where $( ValueKey: IntoArg<$ident> ),*
        {
            fn into_arg(self, _vm: &mut Vm) -> Or<($(Option<$ident>,)*)> {
                todo!()
            }
        }

        impl<$($ident,)*> IntoArg<($($ident,)*)> for ValueKey
        where $( ValueKey: IntoArg<$ident> ),*
        {
            #[allow(clippy::unused_unit)]
            fn into_arg(self, vm: &mut Vm) -> ($($ident,)*) {
                (
                    $({
                        stringify!($ident);
                        self.into_arg(vm)
                    },)*
                )
            }
        }

        impl<Fun, $($ident,)*> Invoke<0, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)* &mut Vm, CodeArea) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* _vm, _area )
            }
        }

        impl<Fun, $($ident,)*> Invoke<1, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)* &mut Vm) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* _vm, )
            }
        }

        impl<Fun, $($ident,)*> Invoke<2, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)*) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* )
            }
        }
    };

    ($first:ident $( $name:ident )* ) => {
        tuple_macro!( @gen $first $( $name )* );

        tuple_macro!( $( $name )* );
    };

    () => {
        tuple_macro!(@gen);
    };
}

tuple_macro! {A B C D}

//     impl @error {
//         const TYPE_MISMATCH = Int(0);

//         < #[deprecated] >
//         fn poo(
//           #[self]
//             &self / &mut self,

//             Thing(...)
//             r: mut ref Thing
//             r: ref Thing
//             r where Area(a) Key(k),

//             where Area(...) Key(...) Value(...)
//
//             _: Range(start, end, step) | String where CodeArea(a) ValueKey(k)

//r: &String | &Int

//             r: &mut Range @ range_area,
//             r: &Range,

//             a: Range(start, end, step),

//             raw k,
//         ) {
//             if let Some(v) = r.is::<Range>() {
//                 //dfdfdf
//             }
//             if let Some(v) = r.is::<String>() {
//                 / fdfd fd f
//             }

//             match vm.memory.get_mut(key) {
//                 Range(ref mut start, ref mut end, ref mut step)
//             }

//             r.get_mut_ref()
//         }?
//     }
// }

#[rustfmt::skip]
macro_rules! builtin_impl {
    (
        $(#[doc = $impl_doc:literal])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl @$builtin:ident {
            $(
                $(<{ $($fn_raw:tt)* }>)?
                $(#[$fn_doc:meta])?
                fn $fn_name:ident (
                    $(
                        $(
                            $var:ident $(: $(
                                $(mut ref $mut_ref_variant:ident)?
                                $(ref $ref_variant:ident)?
                            )|+)?
                        )?

                        $(
                            _: $(
                                $variant:ident $(($($tok1:tt)*))? $({$($tok2:tt)*})?
                            )|+
                        )?

                        $(where $($extra:ident($bind:ident))+)?

                        ,
                    )*
                ) $(-> $ret_type:ty)? $code:block
            )*
        }
    ) => {
        #[cfg(test)]
        pub mod $builtin {

            #[test]
            pub fn core_gen() {
                //let (slf, line, col) = (file!(), line!(), column!());
                let path = std::path::PathBuf::from(format!("{}{}{}", $crate::CORE_PATH, stringify!($builtin), ".spwn"));
                let out = indoc::formatdoc!(r#"
                        {impl_raw}
                        #[doc("{impl_doc}")]
                        impl @<> {{
                            #[doc(...)]
                            <>

                            #[doc(...)]
                            <>
                        }}
                    "#, 
                    impl_raw = stringify!($($($impl_raw)*)?),
                    // todo: it no unindent
                    impl_doc = indoc::concatdoc!($($impl_doc, "\n"),*),
                );

                std::fs::write(path, &out).unwrap();
            }
        }
    };
}

builtin_impl! {
    /// foo bar
    /// # Example:
    /// ```
    ///     fn test() {}
    /// ```
    #[raw( #[deprecated] )]
    impl @string {
    }
}
