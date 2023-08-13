use std::rc::Rc;

use crate::interpreting::builtins::impl_type;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::Value;
use crate::interpreting::vm::{DeepClone, ValueRef};
use crate::parsing::ast::{VisSource, VisTrait};

impl_type! {
    impl Dict {
        Constants:

        Functions(ctx, vm, program, area):

        /// fdf
        fn get_or_insert(&mut Dict(slf) as "self", String(key) as "key", &value) -> "_" {
            let mut slf = slf.borrow_mut();
            let v = slf.entry(Rc::clone(&*key.borrow())).or_insert(VisSource::Public(value.clone()));

            let v = vm.deep_clone_ref((*v).value());

            Multi::new_single(ctx, Ok(v))
        }

        /// dfdfdf
        fn insert(&mut Dict(slf) as "self", String(key) as "key", &value) {
            slf.borrow_mut().insert(Rc::clone(&*key.borrow()), VisSource::Public(value.clone()));

            Multi::new_single(ctx, Ok(Value::Empty.into_value_ref(area)))
        }

    }
}
