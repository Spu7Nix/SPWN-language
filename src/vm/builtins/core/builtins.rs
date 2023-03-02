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
        const bulgaria = Int(3);

        Functions(vm, call_area):
        fn print(
            Builtins as self,
            ...args,
            end: String = r#""\n""#,
            sep: String = r#"" ""#,
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
            ignore_context: Bool,
        ) {
            let obj = GdObject { params, mode };

            match mode {
                ObjectType::Object => {
                    if !ignore_context.0 && vm.contexts.group() != Id::Specific(0) {
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

        fn assert(Builtins as self, value: Bool, is: Bool) {
            if value != is {
                return Err(RuntimeError::AssertionFailed { area: call_area, call_stack: vm.get_call_stack() });
            }
            Value::Empty
        }

        fn assert_eq(Builtins as self, left, right) {
            if !value_ops::equality(left, right, vm) {
                return Err(RuntimeError::AssertionFailed { area: call_area, call_stack: vm.get_call_stack() });
            }
            Value::Empty
        }
    }
}
