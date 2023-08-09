use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::gd::gd_object::{GdObject, TriggerObject};
use crate::gd::ids::Id;
use crate::interpreting::builtins::impl_type;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::Value;
use crate::interpreting::value_ops;
use crate::interpreting::vm::ValueRef;
use crate::parsing::ast::ObjectType;

impl_type! {
    impl Builtins {
        Constants:

        Functions(ctx, vm, program, area):

        /// g
        fn print(
            &Builtins{} as "self",
            ...args,
            String(end) as "end" = r#""\n""#,
            String(sep) as "sep" = r#"" ""#,
        ) {
            let mut ret = Multi::new_single(ctx, Ok(vec![]));
            let end = end.borrow().to_string();
            let sep = sep.borrow().to_string();

            for elem in args {
                ret = ret.try_flat_map(|ctx, v| {
                    let g = vm.runtime_display(ctx, elem, &area, program);

                    g.try_map(|ctx, new_elem| {
                        let mut v = v.clone();
                        v.push(new_elem);
                        (ctx, Ok(v))
                    })
                });
            }

            ret.try_map(|c, v| (c, {
                use itertools::Itertools;

                print!("{}{}", v.iter().join(&sep), end);

                Ok(ValueRef::new(Value::Empty.into_stored(area.clone())))
            }))
        }

        /// fghfddggfd
        fn add(
            &Builtins{} as "self",
            Object {
                params,
                typ,
            } as "object",
            Bool(ignore_context) as "ignore_context" = "false",
        ) {
            let typ = *typ.borrow();

            let obj = GdObject {
                params: params.borrow().clone(),
                mode: typ,
            };

            match typ {
                ObjectType::Object => {
                    if !*ignore_context.borrow() && ctx.group != Id::Specific(0) {
                        return Multi::new_single(ctx, Err(RuntimeError::AddObjectInTriggerContext {
                            area,
                            call_stack: vm.get_call_stack(),
                        }));
                    }
                    vm.objects.push(obj)
                }
                ObjectType::Trigger => vm.triggers.push(
                    TriggerObject {
                        obj,
                        order: vm.trigger_order_count.next(),
                    }
                    .apply_context(ctx.group),
                ),
            }

            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// dfdf
        fn epsilon(&Builtins{} as "self") -> "@epsilon" {
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Epsilon.into_stored(area))))
        }

        /// dfdf
        fn trigger_fn_context(&Builtins{} as "self") -> "@group" {
            let context_g = Value::Group(ctx.group);
            Multi::new_single(ctx, Ok(ValueRef::new(context_g.into_stored(area))))
        }

        /// dfdf
        fn assert(&Builtins{} as "self", Bool(value) as "value") {
            if !*value.borrow() {
                return Multi::new_single(ctx, Err(RuntimeError::AssertionFailed { area, call_stack: vm.get_call_stack() }))
            }
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// gg
        fn assert_eq(&Builtins{} as "self", left, right) {
            if !value_ops::equality(&left.borrow().value, &right.borrow().value) {
                return Multi::new_single(ctx, Err(RuntimeError::EqAssertionFailed { area, call_stack: vm.get_call_stack() }));
            }
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// gfg
        fn hash(&Builtins{} as "self", &value) -> "@int" {
            let mut state = DefaultHasher::default();
            vm.hash_value(value, &mut state);
            let hash = state.finish();

            Multi::new_single(ctx, Ok(ValueRef::new(Value::Int(hash as i64).into_stored(area))))
        }
    }
}
