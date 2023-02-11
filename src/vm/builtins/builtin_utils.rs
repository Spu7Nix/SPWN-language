use crate::sources::CodeArea;
use crate::vm::interpreter::{RuntimeResult, ValueKey, Vm};
use crate::vm::value::Value;

pub trait Invoke<const N: usize, Args = ()> {
    fn invoke(&self, args: Vec<ValueKey>, vm: &mut Vm, area: CodeArea) -> RuntimeResult<Value>;
}

pub trait IntoArg<O> {
    fn into_arg(self, vm: &Vm) -> O;
}

pub struct Or<T>(T);
pub struct Spread<T>(Vec<T>);

impl<T> std::ops::Deref for Spread<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> IntoArg<Spread<T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, vm: &Vm) -> Spread<T> {
        match &vm.memory[self].value {
            Value::Array(arr) => Spread(arr.iter().map(|k| k.into_arg(vm)).collect()),
            _ => unreachable!(),
        }
    }
}

impl IntoArg<Value> for ValueKey {
    fn into_arg(self, vm: &Vm) -> Value {
        vm.memory[self].value.clone()
    }
}

impl IntoArg<ValueKey> for ValueKey {
    fn into_arg(self, _: &Vm) -> ValueKey {
        self
    }
}

macro_rules! tuple_macro {
    (@gen $($ident:ident)*) => {

        impl<$($ident,)*> IntoArg<Or<($(Option<$ident>,)*)>> for ValueKey
        where $( ValueKey: IntoArg<$ident> ),*
        {
            fn into_arg(self, _vm: &Vm) -> Or<($(Option<$ident>,)*)> {
                todo!()
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
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), &_vm);
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
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), &_vm);
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
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), &_vm);
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
