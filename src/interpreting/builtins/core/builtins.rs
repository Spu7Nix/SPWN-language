use crate::gd::gd_object::{GdObject, TriggerObject};
use crate::gd::ids::Id;
use crate::interpreting::builtins::impl_type;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::Value;
use crate::interpreting::vm::ValueRef;
use crate::parsing::ast::ObjectType;

impl_type! {
    impl Builtins {
        Constants:

        Functions(ctx, vm, program, area):


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

    }
}
