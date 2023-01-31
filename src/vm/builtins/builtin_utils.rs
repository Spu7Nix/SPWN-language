use std::any::Any;
use std::ops::Range;

use crate::sources::CodeArea;
use crate::vm::error::RuntimeError;
use crate::vm::interpreter::{ValueKey, Vm};
use crate::vm::value::{MacroCode, Value, ValueType};

#[derive(delve::EnumDisplay, Clone, Debug)]
pub enum BuiltinValueType {
    #[delve(display = |v: &ValueType| format!("{v}"))]
    Atom(ValueType),
    #[delve(display = |v: &'static [BuiltinValueType]| v.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(", "))]
    List(&'static [BuiltinValueType]),
    None,
}

// returns the respective spwn type name of rust types
pub trait TypeOf {
    const TYPE: BuiltinValueType;
}

// returns whether the value is optional in arguments or not
pub trait IsOptional {
    const OPTIONAL: bool = false;
}

// returns the spwn type name for a custom builtin type
pub trait TypeName {
    const NAME: &'static str;
}

// functions to call rust functions / get struct members from within spwn
pub trait BuiltinType {
    fn invoke_static(name: &str, vm: &mut Vm) -> Result<Value, RuntimeError>;
    fn invoke_self(&self, name: &str, vm: &mut Vm) -> Result<Value, RuntimeError> {
        unimplemented!()
    }
}

// gets a value of a given type within a tuple
pub trait TOf<Args = ()> {
    fn get<T: 'static>(&self) -> Option<&T>;
}

pub trait NextValue<T> {
    type Output;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output>;
}

pub trait ToValue<const O: usize = 0, A = ()> {
    fn to_value(self, vm: &mut Vm) -> Result<Value, RuntimeError>;
}

pub trait Invoke<const O: usize, Args, A = ()> {
    type Result;

    fn invoke_fn(&self, args: &mut Args, vm: &mut Vm, area: CodeArea) -> Self::Result;
}

pub struct Of<Types>(Types);

pub struct Spread<T>(Vec<T>);

impl<T> std::ops::Deref for Spread<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! of {
    ($($t:ty),*) => {
        $crate::vm::builtins::builtin_utils::Of<($(Option<$t>),*)>
    };
}

macro_rules! inner_fn {
    (@invoke $($name:ident)*, $self:ident, $args:ident, $vm:ident, $area:ident) => {
        $(
            if $args.is_empty() && !$name::OPTIONAL {
                return Err(RuntimeError::TooFewBuiltinArguments {
                    call_area: $area.clone(),
                    call_stack: $vm.get_call_stack(),
                })
            }

            let $name: $name = match $args.last() {
                Some(a) => {
                    let v = &$vm.memory[*a];
                    let found = v.value.get_type();
                    let def_area = v.area.clone();

                    <Vec<ValueKey> as NextValue<$name>>::next_value($args, $vm).ok_or_else(|| {
                        let expected = $name::TYPE;

                        RuntimeError::InvalidBuiltinArgumentType {
                            call_area: $area.clone(),
                            call_stack: $vm.get_call_stack(),
                            def_area,
                            expected,
                            found,
                        }
                    })?
                }
                None => {
                    <Vec<ValueKey> as NextValue<$name>>::next_value($args, $vm).unwrap()
                }
            };
        )*

        if !$args.is_empty() {
            return Err(RuntimeError::TooManyBuiltinArguments {
                call_area: $area.clone(),
                call_stack: $vm.get_call_stack(),
            })
        }
    };

    (@to_value $self:ident) => {
        Ok(Value::Macro(MacroCode::Builtin(std::rc::Rc::new(move |_args, _vm, _area| {
            $self.invoke_fn(_args, _vm, _area).to_value(_vm)
        }))))
    }
}

macro_rules! tuple_impls {
    ( $( $name:ident )* ) => {
        impl<Fun, Res, $($name),*> Invoke<0, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &Vm) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( $($name,)* _vm))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<0, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)* &Vm) -> Res + Invoke<0, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, Res, $($name),*> Invoke<1, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( $($name,)* _vm))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<1, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm) -> Res + Invoke<1, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, Res, $($name),*> Invoke<2, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: IsOptional + TypeOf,
            )*
            Res: ToValue,
            Fun: Fn($($name,)*) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( $($name,)*))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<2, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)*) -> Res + Invoke<2, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, Res, $($name),*> Invoke<3, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &Vm, CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( $($name,)* _vm, _area))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<3, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)* &Vm, CodeArea) -> Res + Invoke<3, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, Res, $($name),*> Invoke<4, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm, CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                if !_args.is_empty() {
                    return Err(RuntimeError::TooManyBuiltinArguments {
                        call_area: _area.clone(),
                        call_stack: _vm.get_call_stack(),
                    })
                }
                Ok((self)( $($name,)* _vm, _area))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<4, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)* &mut Vm, CodeArea) -> Res + Invoke<4, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, Res, $($name),*> Invoke<5, Vec<ValueKey>, ($($name,)*)> for Fun
        where
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn($($name,)* CodeArea) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( $($name,)* _area))
            }
        }

        impl<Fun, Res, $($name,)*> ToValue<5, ($($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn($($name,)* CodeArea) -> Res + Invoke<5, Vec<ValueKey>, ($($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, This, Res, $($name),*> Invoke<6, Vec<ValueKey>, (This, $($name,)*)> for Fun
        where
            Vec<ValueKey>: NextValue<This, Output = This>,
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            Fun: Fn(This, $($name,)*) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                let _this: This = <Vec<ValueKey> as NextValue<This>>::next_value(_args, _vm).unwrap();
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( _this, $($name,)*))
            }
        }

        impl<Fun, This, Res, $($name,)*> ToValue<6, (This, $($name,)*)> for Fun
        where
            Res: ToValue,
            Fun: Fn(This, $($name,)*) -> Res + Invoke<6, Vec<ValueKey>, (This, $($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, This, Res, $($name),*> Invoke<7, Vec<ValueKey>, (This, $($name,)*)> for Fun
        where
            Vec<ValueKey>: NextValue<This, Output = This>,
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            for<'a> Fun: Fn(&'a This, $($name,)*) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                let _this: This = <Vec<ValueKey> as NextValue<This>>::next_value(_args, _vm).unwrap();
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( &_this, $($name,)*))
            }
        }

        impl<Fun, This, Res, $($name,)*> ToValue<7, (This, $($name,)*)> for Fun
        where
            Res: ToValue,
            for<'a> Fun: Fn(&'a This, $($name,)*) -> Res + Invoke<7, Vec<ValueKey>, (This, $($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        impl<Fun, This, Res, $($name),*> Invoke<8, Vec<ValueKey>, (This, $($name,)*)> for Fun
        where
            Vec<ValueKey>: NextValue<This, Output = This>,
            $(
                Vec<ValueKey>: NextValue<$name, Output = $name>,
                $name: TypeOf + IsOptional,
            )*
            Res: ToValue,
            for<'a> Fun: Fn(&'a mut This, $($name,)*) -> Res
        {
            type Result = Result<Res, RuntimeError>;

            fn invoke_fn(&self, _args: &mut Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> Self::Result {
                let mut _this: This = <Vec<ValueKey> as NextValue<This>>::next_value(_args, _vm).unwrap();
                inner_fn!(@invoke $($name)*, self, _args, _vm, _area);
                Ok((self)( &mut _this, $($name,)*))
            }
        }

        impl<Fun, This, Res, $($name,)*> ToValue<8, (This, $($name,)*)> for Fun
        where
            Res: ToValue,
            for<'a> Fun: Fn(&'a mut This, $($name,)*) -> Res + Invoke<8, Vec<ValueKey>, (This, $($name,)*), Result = Result<Res, RuntimeError>> + 'static,
        {
            fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                inner_fn!(@to_value self)
            }
        }

        ////////////////////////////

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

macro_rules! impl_for_t {
    (
        $(  $ty:ty $(: $en:ident)? ),*
    ) => {
        $(
            impl IsOptional for $ty {}

            $(
                impl ToValue for $ty {
                    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
                        Ok(Value::$en(self))
                    }
                }

                impl TypeOf for $ty {
                    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::$en);
                }

                impl NextValue<$ty> for Vec<ValueKey> {
                    type Output = $ty;

                    // TODO: optimise this
                    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
                        match &vm.memory[self.pop()?].value {
                            Value::$en(_v) => Some(_v.clone()),
                            _ => None,
                        }
                    }
                }
            )?
        )*
    };
}

impl_for_t! {
    (),
    bool: Bool,
    String: String,
    Value,
    Vec<Value>,
    Range<i64>,
    f64: Float,
    i64: Int
}

// special cases of each trait
impl<T> IsOptional for Spread<T> {
    const OPTIONAL: bool = true;
}
impl<T> IsOptional for Option<T> {
    const OPTIONAL: bool = true;
}
impl<T> IsOptional for Of<T> {}

impl ToValue for Value {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(self)
    }
}
impl ToValue for usize {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::Int(self as i64))
    }
}
impl ToValue for () {
    fn to_value(self, _: &mut Vm) -> Result<Value, RuntimeError> {
        Ok(Value::Empty)
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

// `TypeOf for ()` is implemented in the `tuple_impls` macro (empty tuple)
impl TypeOf for Value {
    const TYPE: BuiltinValueType = BuiltinValueType::None;
}
impl<T> TypeOf for Spread<T> {
    const TYPE: BuiltinValueType = BuiltinValueType::None;
}
impl<T: TypeOf> TypeOf for Option<T> {
    const TYPE: BuiltinValueType =
        BuiltinValueType::List(&[BuiltinValueType::Atom(ValueType::Empty), T::TYPE]);
}
impl TypeOf for Range<i64> {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Range);
}
impl TypeOf for Vec<Value> {
    const TYPE: BuiltinValueType = BuiltinValueType::Atom(ValueType::Array);
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
impl NextValue<Vec<Value>> for Vec<ValueKey> {
    type Output = Vec<Value>;

    fn next_value(&mut self, vm: &mut Vm) -> Option<Self::Output> {
        match &vm.memory[self.pop()?].value {
            Value::Array(a) => Some(a.iter().map(|k| vm.memory[*k].value.clone()).collect()),
            _ => None,
        }
    }
}
impl<T> NextValue<Option<T>> for Vec<ValueKey>
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
