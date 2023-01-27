use std::any::Any;
use std::ops::Range;

use delve::{EnumDisplay, EnumFromStr};
use rand::seq::SliceRandom;
use rand::Rng;

use super::interpreter::{ValueKey, Vm};
use super::value::Value;

type Array = Vec<Value>;
pub struct Spread<T>(Vec<T>);

impl<T> std::ops::Deref for Spread<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait NextValue<T> {
    type Output;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output>;
}

trait ToValue {
    fn to_value(self, vm: &mut Vm) -> Value;
}

pub trait TOf<Types = ()> {
    fn get<T: 'static>(&self) -> Option<&T>;
}

impl ToValue for () {
    fn to_value(self, _: &mut Vm) -> Value {
        Value::Empty
    }
}
impl ToValue for i64 {
    fn to_value(self, _: &mut Vm) -> Value {
        Value::Int(self)
    }
}
impl ToValue for f64 {
    fn to_value(self, _: &mut Vm) -> Value {
        Value::Float(self)
    }
}
impl ToValue for Value {
    fn to_value(self, _: &mut Vm) -> Value {
        self
    }
}

impl NextValue<f64> for Vec<ValueKey> {
    type Output = f64;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match vm.memory[self.pop()?].value {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }
}
impl NextValue<i64> for Vec<ValueKey> {
    type Output = i64;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match vm.memory[self.pop()?].value {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }
}
impl NextValue<bool> for Vec<ValueKey> {
    type Output = bool;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match vm.memory[self.pop()?].value {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
}
impl NextValue<String> for Vec<ValueKey> {
    type Output = String;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match &vm.memory[self.pop()?].value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}
impl NextValue<Range<i64>> for Vec<ValueKey> {
    type Output = Range<i64>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match vm.memory[self.pop()?].value {
            Value::Range(start, end, _) => Some(start..end),
            _ => None,
        }
    }
}

impl NextValue<Spread<Value>> for Vec<ValueKey> {
    type Output = Spread<Value>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        Some(Spread(
            self.iter().map(|k| vm.memory[*k].value.clone()).collect(),
        ))
    }
}

impl NextValue<ValueKey> for Vec<ValueKey> {
    type Output = ValueKey;

    fn next_value(&mut self, _: &mut Vm) -> Option<Self::Output> {
        self.pop()
    }
}

impl NextValue<Vec<Value>> for Vec<ValueKey> {
    type Output = Vec<Value>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match &vm.memory[self.pop()?].value {
            Value::Array(a) => Some(a.iter().map(|k| vm.memory[*k].value.clone()).collect()),
            _ => None,
        }
    }
}
// impl NextValue<Vec<ValueKey>> for Vec<ValueKey> {
//     type Output = Vec<ValueKey>;

//     fn next_value(&mut self, _: &mut Vm) -> Option<Self::Output> {
//         self.reverse();
//         Some(self.to_vec())
//     }
// }

impl<T> NextValue<Option<T>> for Vec<ValueKey>
where
    Vec<ValueKey>: NextValue<T, Output = T>,
{
    type Output = Option<T>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        Some(<Vec<ValueKey> as NextValue<T>>::next_value(self, vm))
    }
}

pub struct Of<Types>(Types);

macro_rules! tuple_impls {
    ( $( $name:ident )* ) => {
        impl<Fun, Res, $($name),*> BuiltinFn<0, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &Vm) -> Res
        {
            type Result = Res;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm) -> Self::Result {
                // len check
                $(
                    #[allow(non_snake_case)]
                    let $name: $name = <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap();
                )*
                (self)( $($name,)* _vm)
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<1, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm) -> Res
        {
            type Result = Res;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm) -> Self::Result {
                // len check
                $(
                    #[allow(non_snake_case)]
                    let $name: $name = <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap();
                )*
                (self)( $($name,)* _vm)
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<2, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
            )*
            Res: ToValue,
            Fun: Fn($($name,)*) -> Res
        {
            type Result = Res;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm) -> Self::Result {

                // len check
                $(
                    #[allow(non_snake_case)]
                    let $name: $name = <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap();
                )*
                (self)( $($name,)*)
            }
        }

        impl< $($name: 'static,)*> TOf<( $($name,)* )> for Of<( $(Option<$name>,)* )> {
            fn get<Type: 'static>(&self) -> Option<&Type> {
                #[allow(non_snake_case)]
                let ($($name,)*) = &self.0;
                $(
                    if let Some(_v) = $name {
                        return (_v as &dyn Any).downcast_ref::<Type>();
                    }
                )*
                None
            }
        }

        impl< $($name,)* > NextValue<Of<( $(Option<$name>,)* )>> for Vec<ValueKey>
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
            )*
        {
            type Output = Of<( $(Option<$name>,)* )>;

            fn next_value(&mut self, _vm: &mut Vm) -> Option<Self::Output> {
                let _v = *self.last().unwrap();

                $(
                    #[allow(non_snake_case)]
                    #[allow(unused_assignments)]
                    let mut $name: Option<$name> = None;
                )*

                '_a: {
                    $(
                        $name = <Vec<ValueKey> as NextValue<$name>>::next_value(self, _vm);

                        if $name.is_some() {
                            break '_a;
                        } else {
                            self.push(_v);
                        }
                    )*
                }

                Some(Of(( $($name,)* )))
            }
        }
    };
}

pub trait BuiltinFn<const O: usize, Args, A = ()> {
    type Result;

    fn invoke(&self, args: &mut Args, vm: &mut Vm) -> Self::Result;
}

macro_rules! tuple_impl_all {
    ( $first:ident $( $name:ident )* ) => {
        tuple_impls!( $first $( $name )* );

        tuple_impl_all!( $( $name )* );
    };

    () => {
        tuple_impls!();
    };
}

tuple_impl_all! { A B C D }

///////////////////////////////////////////////////////////////////////

#[derive(Debug, EnumFromStr, EnumDisplay, PartialEq, Clone)]
#[delve(rename_variants = "snake_case")]
pub enum Builtin {
    Print,
    Println,
    Exit,
    Random,
    //TriggerFnContext,
}

impl Builtin {
    pub fn call(&self, args: &mut Vec<ValueKey>, vm: &mut Vm) -> Value {
        match self {
            Self::Print => {
                print.invoke(args, vm);
            }
            Self::Println => {
                println.invoke(args, vm);
            }
            Self::Exit => {
                exit.invoke(args, vm);
            }
            Self::Random => {
                return random.invoke(args, vm).to_value(vm);
            }
        }

        ().to_value(vm)
    }
}

pub fn exit() {
    std::process::exit(0);
}

pub fn print(values: Spread<Value>, vm: &Vm) {
    print!(
        "{}",
        values
            .iter()
            .map(|v| v.runtime_display(vm))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
pub fn println(values: Spread<Value>, vm: &Vm) {
    println!(
        "{}",
        values
            .iter()
            .map(|v| v.runtime_display(vm))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

pub fn random(value: Of<(Option<Range<i64>>, Option<Array>, Option<i64>, Option<f64>)>) -> Value {
    if let Some(range) = value.get::<Range<i64>>() {
        return Value::Int(rand::thread_rng().gen_range(range.clone()));
    }
    if let Some(values) = value.get::<Array>() {
        // TODO: handle empty array !!!!
        return values.choose(&mut rand::thread_rng()).unwrap().clone();
    }
    if let Some(n) = value.get::<i64>() {
        return Value::Int(rand::thread_rng().gen_range(0..*n));
    }
    if let Some(n) = value.get::<f64>() {
        return Value::Float(rand::thread_rng().gen_range(0.0..*n));
    }

    unreachable!()
}

pub fn version() -> String {
    include_str!("../VERSION").into()
}
