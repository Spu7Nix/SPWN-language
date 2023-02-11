use super::builtin_utils::{Invoke, Spread};
use crate::vm::error::RuntimeError;
use crate::vm::interpreter::{ValueKey, Vm};
use crate::vm::value::arg_aliases::{ABuiltins, AInt, AString};
use crate::vm::value::Value;

macro_rules! or {
    ( $($t:ty)|* ) => {
        Or<( $( Option<$t> ),* )>
    };
}

use paste::paste;

use crate::vm::value::{BuiltinFn, ValueType};

macro_rules! override_names {
    ($($name:ident : [$($func:ident,)*],)*) => {
        paste! {
            impl ValueType {
                pub fn get_override(self, name: &str) -> Option<BuiltinFn> {
                    match self {
                        $(
                            Self::$name => {
                                match name {
                                    $(
                                        stringify!($func) => Some(BuiltinFn(std::rc::Rc::new(|args: Vec<ValueKey>, vm, area| {
                                            overrides::[<$name:snake>]::$func.invoke(args, vm, area)
                                        }))),
                                    )*
                                    _ => None
                                }
                            },
                        )*
                        _ => None,
                    }
                }
            }
        }
    };
}

override_names! {
    Builtins: [
        print,
        // add,
        // epsilon,
        // trigger_fn_context,
    ],
    // Float: [
    //     sin,
    // ],
}

mod overrides {
    use super::*;
    use crate::gd::gd_object::{GdObject, Trigger};
    use crate::gd::ids::Id;
    use crate::parsing::ast::ObjectType;
    use crate::sources::CodeArea;
    use crate::vm::interpreter::RuntimeResult;
    use crate::vm::value::arg_aliases::*;

    pub mod builtins {
        use super::*;

        pub fn print(
            _self: ABuiltins,
            values: Spread<Value>,
            AString(end): AString,
            AString(sep): AString,
            vm: &mut Vm,
        ) -> RuntimeResult<Value> {
            print!(
                "{}{}",
                values
                    .iter()
                    .map(|v| v.runtime_display(vm))
                    .collect::<Vec<_>>()
                    .join(&sep),
                end,
            );

            Ok(Value::Empty)
        }

        pub fn add(
            _self: ABuiltins,
            AObject(params, mode): AObject,
            ignore_context: ABool,
            vm: &mut Vm,
            area: CodeArea,
        ) -> RuntimeResult<Value> {
            let obj = GdObject { params, mode };

            match mode {
                ObjectType::Object => {
                    if !ignore_context.0 && vm.contexts.group() != Id::Specific(0) {
                        return Err(RuntimeError::AddObjectInTriggerContext {
                            area,
                            call_stack: vm.get_call_stack(),
                        });
                    }
                    vm.objects.push(obj)
                }
                ObjectType::Trigger => vm.triggers.push(
                    Trigger {
                        obj,
                        order: vm.trigger_order_count.next(),
                    }
                    .apply_context(vm.contexts.group()),
                ),
            }

            Ok(Value::Empty)
        }

        pub fn epsilon(_self: ABuiltins) -> RuntimeResult<Value> {
            Ok(Value::Epsilon)
        }

        pub fn trigger_fn_context(_self: ABuiltins, vm: &mut Vm) -> RuntimeResult<Value> {
            Ok(Value::Group(vm.contexts.group()))
        }
    }

    pub mod float {
        use super::*;

        pub fn sin(AFloat(slf): AFloat) -> RuntimeResult<Value> {
            Ok(Value::Float(slf.sin()))
        }
    }

    pub mod array {
        use super::*;

        // pub fn push(slf: &mut AArray, elem: ValueKey, vm: &mut Vm) -> RuntimeResult<Value> {
        //     Ok(Value::Empty)
        // }
    }
}
