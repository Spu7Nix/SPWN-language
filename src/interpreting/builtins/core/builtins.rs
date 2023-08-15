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
use crate::util::{Str32, String32};

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

                Ok(Value::Empty.into_value_ref(area.clone()))
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

            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// dfdf
        fn epsilon(&Builtins{} as "self") -> "@epsilon" {
            Multi::new_single(ctx, Ok(Value::Epsilon.into_value_ref(area)))
        }

        /// dfdf
        fn trigger_fn_context(&Builtins{} as "self") -> "@group" {
            let context_g = Value::Group(ctx.group);
            Multi::new_single(ctx, Ok(context_g.into_value_ref(area)))
        }

        /// dfdf
        fn assert(&Builtins{} as "self", Bool(value) as "value") {
            if !*value.borrow() {
                return Multi::new_single(ctx, Err(RuntimeError::AssertionFailed { area, call_stack: vm.get_call_stack() }))
            }
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// gg
        fn assert_eq(&Builtins{} as "self", left, right) {
            if !value_ops::equality(&left.borrow().value, &right.borrow().value) {
                return Multi::new_single(ctx, Err(RuntimeError::EqAssertionFailed { area, call_stack: vm.get_call_stack() }));
            }
            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

        /// gfg
        fn hash(&Builtins{} as "self", &value) -> "@int" {
            let mut state = DefaultHasher::default();
            vm.hash_value(value, &mut state);
            let hash = state.finish();

            Multi::new_single(ctx, Ok(Value::Int(hash as i64).into_value_ref(area)))
        }

        /// gfg
        fn version(&Builtins{} as "self") -> "[@int, @int, @int, @int?]" {
            let semver = semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("BUG: invalid semver format");

            let v =
                vec![
                    Value::Int(semver.major as i64).into_value_ref(area.clone()),
                    Value::Int(semver.minor as i64).into_value_ref(area.clone()),
                    Value::Int(semver.patch as i64).into_value_ref(area.clone()),
                    {
                        if semver.pre == semver::Prerelease::EMPTY {
                            Value::Maybe(None).into_value_ref(area.clone())
                        } else {
                            let beta = semver.pre.split('.').nth(1).expect("BUG: missing beta version");
                            Value::Maybe(
                                Some(
                                    Value::Int(
                                        beta.parse::<i64>().expect("BUG: invalid beta number")
                                    ).into_value_ref(area.clone())
                                )
                            ).into_value_ref(area.clone())
                        }
                    }
                ];

            Multi::new_single(ctx, Ok(Value::Array(v).into_value_ref(area)))
        }

        /// gfg
        fn args(&Builtins{} as "self") -> "@string[]" {
            let args: Vec<ValueRef> = vm.trailing_args.iter().map(
                |s| {
                    Value::String(
                        String32::from_str(s.as_str()).into()
                    ).into_value_ref(area.clone())
                }
            ).collect();

            Multi::new_single(ctx, Ok(Value::Array(args).into_value_ref(area)))
        }
    }
}
