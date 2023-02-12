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
// pub trait IntoValue {
//     fn into_value(self) -> Value;
// }

pub trait GetMutArg<'a> {
    type Output;

    fn get_mut_arg(key: ValueKey, vm: &'a mut Vm) -> Self::Output;
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
pub struct Mut<'a, T: GetMutArg<'a>> {
    pub key: ValueKey,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: GetMutArg<'a>> Mut<'a, T> {
    pub fn get_mut(self, vm: &'a mut Vm) -> <T as GetMutArg<'a>>::Output {
        T::get_mut_arg(self.key, vm)
    }
}

// pub struct Mut<T: IntoValue> {
//     pub value: T,
//     pub key: ValueKey,
// }

// impl<T: IntoValue> std::ops::Deref for Mut<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }

// impl<T: IntoValue> std::ops::DerefMut for Mut<T> {
//     fn deref_mut(&mut self) -> &mut T {
//         &mut self.value
//     }
// }

// impl<T: IntoValue> Mut<T> {
//     pub fn update(self, vm: &mut Vm) {
//         vm.memory[self.key].value = self.value.into_value()
//     }

//     pub fn update_with_area(self, area: CodeArea, vm: &mut Vm) {
//         vm.memory[self.key].value = self.value.into_value();
//         vm.memory[self.key].area = area;
//     }
// }

////////////////////

impl<'a, T: GetMutArg<'a>> IntoArg<Mut<'a, T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, _: &mut Vm) -> Mut<'a, T> {
        Mut {
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
