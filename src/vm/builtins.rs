use std::any::Any;
use std::io;
use std::io::Write;
use std::ops::Range;

use delve::{EnumDisplay, EnumFromStr};
use rand::seq::SliceRandom;
use rand::Rng;
use strum::EnumProperty;

use super::error::RuntimeError;
use super::interpreter::{ValueKey, Vm};
use super::value::{Value, ValueType};
use super::value_ops;
use crate::sources::CodeArea;

#[derive(delve::EnumDisplay, Clone, Debug)]
pub enum BuiltinValueType {
    #[delve(display = |v: &ValueType| format!("{v}"))]
    Atom(ValueType),
    #[delve(display = |v: &'static [BuiltinValueType]| v.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(", "))]
    List(&'static [BuiltinValueType]),
    None,
}

trait TypeOf {
    const TYPE: BuiltinValueType;
}

impl TypeOf for bool {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Bool);
}
impl TypeOf for f64 {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Float);
}
impl TypeOf for () {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Empty);
}
impl TypeOf for String {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::String);
}
impl TypeOf for i64 {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Int);
}
impl TypeOf for Array {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Array);
}
impl TypeOf for Value {
    const TYPE: BuiltinValueType = BuiltinValueType::None;
}
impl TypeOf for Range<i64> {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Range);
}
impl<T> TypeOf for Spread<T> {
    const TYPE: BuiltinValueType = BuiltinValueType::None;
}
impl<T: TypeOf> TypeOf for Option<T> {
    const TYPE: BuiltinValueType =
        BuiltinValueType::List(&[BuiltinValueType::Atom(ValueType::Empty), T::TYPE]);
}

trait IsOptional {
    const OPTIONAL: bool = false;
}
impl IsOptional for bool {}
impl IsOptional for f64 {}
impl IsOptional for () {}
impl IsOptional for String {}
impl IsOptional for i64 {}
impl IsOptional for Array {}
impl IsOptional for Value {}
impl IsOptional for Range<i64> {}
impl<T> IsOptional for Of<T> {}
impl<T> IsOptional for Spread<T> {
    const OPTIONAL: bool = true;
}
impl<T> IsOptional for Option<T> {
    const OPTIONAL: bool = true;
}

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
    fn to_value(self, vm: &mut Vm) -> Result<Value, RuntimeError>;
}

pub trait TOf<Types = ()> {
    fn get<T: 'static>(&self) -> Option<&T>;
}

impl ToValue for () {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::Empty)
    }
}
impl ToValue for i64 {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::Int(self))
    }
}
impl ToValue for f64 {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::Float(self))
    }
}
impl ToValue for String {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::String(self))
    }
}
impl ToValue for Value {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(self)
    }
}

impl<T: ToValue> ToValue for Result<T, RuntimeError> {
    fn to_value(self, vm: &mut Vm) -> Result<Value, RuntimeError> {
        match self {
            Ok(v) => Ok(v.to_value(vm)?),
            Err(e) => Err(e),
        }
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
impl NextValue<Value> for Vec<ValueKey> {
    type Output = Value;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        Some(vm.memory[self.pop()?].value.clone())
    }
}

impl NextValue<Spread<Value>> for Vec<ValueKey> {
    type Output = Spread<Value>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        let mut out = vec![];
        out.append(self);
        out.reverse();
        Some(Spread(
            out.iter().map(|k| vm.memory[*k].value.clone()).collect(),
        ))
    }
}

impl NextValue<ValueKey> for Vec<ValueKey> {
    type Output = ValueKey;

    fn next_value(&mut self, _: &mut Vm) -> Option<Self::Output> {
        self.pop()
    }
}

impl NextValue<Array> for Vec<ValueKey> {
    type Output = Array;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match &vm.memory[self.pop()?].value {
            Value::Array(a) => Some(a.iter().map(|k| vm.memory[*k].value.clone()).collect()),
            _ => None,
        }
    }
}

impl<T: std::fmt::Debug> NextValue<Option<T>> for Vec<ValueKey>
where
    Vec<ValueKey>: NextValue<T, Output = T>,
{
    type Output = Option<T>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        Some(if self.is_empty() {
            None
        } else {
            Some(<Vec<ValueKey> as NextValue<T>>::next_value(self, vm)?)
        })
    }
}

pub struct Of<Types>(Types);

macro_rules! tuple_impls {
    ( $( $name:ident )* ) => {
        impl<Fun, Res, $($name),*> BuiltinFn<0, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &Vm) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _vm))
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<1, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _vm))
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<2, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: IsOptional + TypeOf,
            )*
            Res: ToValue,
            Fun: Fn($($name,)*) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)*))
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<3, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &Vm, CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _vm, _area))
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<4, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm, CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _vm, _area))
            }
        }

        impl<Fun, Res, $($name),*> BuiltinFn<5, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                $(
                    if _args.is_empty() && !$name::OPTIONAL {
                        return Err(RuntimeError::TooFewBuiltinArguments {
                            call_area: _area.clone(),
                            call_stack: _vm.get_call_stack(),
                        })
                    }

                    let $name: $name = match _args.last() {
                        Some(a) => {
                            let v = &_vm.memory[*a];
                            let found = v.value.get_type();
                            let def_area = v.area.clone();

                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).ok_or_else(|| {
                                let expected = $name::TYPE;

                                RuntimeError::InvalidBuiltinArgumentType {
                                    call_area: _area.clone(),
                                    call_stack: _vm.get_call_stack(),
                                    def_area,
                                    expected,
                                    found,
                                }
                            })?
                        }
                        None => {
                            <Vec<ValueKey> as NextValue<$name>>::next_value(_args, _vm).unwrap()
                        }
                    };
                )*
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _area))
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

        impl< $($name: std::fmt::Debug,)* > NextValue<Of<( $(Option<$name>,)* )>> for Vec<ValueKey>
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
                            self.pop();
                            break '_a;
                        } else {
                            self.push(_v);
                        }
                    )*

                }

                if ( $( $name.is_none() && )* true ) {
                    return None
                }

                Some(Of(( $($name,)* )))
            }
        }

        impl< $($name,)* > TypeOf for Of<( $( Option<$name>,)* )>
        where $( $name: TypeOf, )*
        {
            const TYPE: BuiltinValueType = BuiltinValueType::List(&[ $($name::TYPE,)* ]);
        }
    };
}

pub trait BuiltinFn<const O: usize, Args, A = ()> {
    type Result;

    fn invoke(&self, args: &mut Args, vm: &mut Vm, area: CodeArea) -> Self::Result;
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

macro_rules! of {
    ($($t:ty),*) => {
        Of<($(Option<$t>),*)>
    };
}

///////////////////////////////////////////////////////////////////////

#[derive(Debug, EnumFromStr, EnumDisplay, PartialEq, Clone)]
#[delve(rename_variants = "snake_case")]
pub enum Builtin {
    Print,
    Println,
    Exit,
    Random,
    Version,
    Assert,
    AssertEq,
    Input,
}

impl Builtin {
    pub fn call(
        &self,
        args: &mut Vec<ValueKey>,
        vm: &mut Vm,
        area: CodeArea,
    ) -> Result<Value, RuntimeError> {
        match self {
            Self::Print => print.invoke(args, vm, area).to_value(vm),
            Self::Println => println.invoke(args, vm, area).to_value(vm),
            Self::Exit => exit.invoke(args, vm, area).to_value(vm),
            Self::Random => random.invoke(args, vm, area).to_value(vm),
            Self::Version => version.invoke(args, vm, area).to_value(vm),
            Self::Input => input.invoke(args, vm, area).to_value(vm),
            Self::Assert => assert.invoke(args, vm, area).to_value(vm),
            Self::AssertEq => assert_eq.invoke(args, vm, area).to_value(vm),
        }
    }
}

pub fn exit(_vm: &mut Vm) {
    // vm.contexts.yeet_current();
    // the goof (the sill)
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

pub fn random(value: of!(Range<i64>, Array, i64, f64)) -> Value {
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

pub fn input(prompt: Option<String>) -> String {
    let prompt = prompt.unwrap_or(String::new());

    print!("{prompt}");
    std::io::stdout().flush().unwrap();

    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("Couldn't read line");

    s.trim_end_matches(|p| matches!(p, '\n' | '\r')).into()
}

pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

pub fn assert(expr: bool, vm: &Vm, area: CodeArea) -> Result<(), RuntimeError> {
    if !expr {
        return Err(RuntimeError::AssertionFailed {
            area,
            call_stack: vm.get_call_stack(),
        });
    }

    Ok(())
}
pub fn assert_eq(a: Value, b: Value, vm: &Vm, area: CodeArea) -> Result<(), RuntimeError> {
    if !value_ops::equality(&a, &b, vm) {
        return Err(RuntimeError::EqAssertionFailed {
            area,
            left: a.runtime_display(vm),
            right: b.runtime_display(vm),
            call_stack: vm.get_call_stack(),
        });
    }

    Ok(())
}
