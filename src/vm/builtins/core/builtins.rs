use std::hash::Hasher;
use std::time::SystemTime;

use crate::gd::gd_object::{GdObject, Trigger};
use crate::gd::ids::Id;
use crate::parsing::ast::ObjectType;
use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::Value;
use crate::vm::value_ops;

impl_type! {
    impl Builtins {
        Constants:

        Functions(vm, call_area):
        fn print(
            Builtins as self,
            args...: Value,
            end: String = {"\n"},
            sep: String = {" "},
        ) {
            print!(
                "{}{}",
                args
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.iter().collect(),
                        _ => v.runtime_display(vm),
                    })
                    .collect::<Vec<_>>()
                    .join(&sep.iter().collect::<String>()),
                end.iter().collect::<String>(),
            );

            Value::Empty
        }

        fn add(
            Builtins as self,
            Object(params, mode) as object,
            ignore_context: Bool = {false},
        ) {
            let obj = GdObject { params, mode };

            match mode {
                ObjectType::Object => {
                    if !*ignore_context && vm.contexts.group() != Id::Specific(0) {
                        return Err(RuntimeError::AddObjectInTriggerContext {
                            area: call_area,
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

            Value::Empty
        }

        fn epsilon(Builtins as self) -> Epsilon {
            Value::Epsilon
        }

        fn trigger_fn_context(Builtins as self) -> Group {
            Value::Group(vm.contexts.group())
        }

        fn assert(Builtins as self, value: Bool) {
            if !*value {
                return Err(RuntimeError::AssertionFailed { area: call_area, call_stack: vm.get_call_stack() });
            }
            Value::Empty
        }

        fn assert_eq(Builtins as self, left: Value, right: Value) {
            if !value_ops::equality(&left, &right, vm) {
                return Err(RuntimeError::AssertionFailed { area: call_area, call_stack: vm.get_call_stack() });
            }
            Value::Empty
        }

        fn hash(Builtins as self, value: ValueKey) -> Int {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            vm.hash_value(value, &mut hasher);
            Value::Int(unsafe { std::mem::transmute::<u64, i64>(hasher.finish()) })
        }

        fn time(Builtins as self) -> Int {
            match std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(time) => Value::Float(time.as_secs_f64()),
                Err(e) => {
                    // return Err(Runti) // not sure if there needs to be added a new error for this, idk
                    Value::Float(0.0)
                }
            }
        }

        fn random(Builtins as self, input: Array | Range | Empty = {()}) -> Float {
            use rand::prelude::*;

            let mut rng = rand::thread_rng();

            match input {
                InputValue::Array(array) =>
                    array.choose(&mut rng).map_or(Value::Maybe(None), |v| {
                        let value_key =  vm.deep_clone_key_insert(*v);
                        vm.memory[value_key].value.clone()
                    }),
                InputValue::Range(RangeDeref(start, end, step)) =>
                    Value::Int((start..end).step_by(step).choose(&mut rng).unwrap_or(0)),
                InputValue::Empty(_) =>
                    Value::Float(rng.gen::<f64>()), // 0.0..1.0
                _ =>
                    unreachable!(),
            }
        }
    }
}
