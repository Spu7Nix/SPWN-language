use crate::interpreting::builtins::impl_type;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::Value;
use crate::interpreting::vm::{DeepClone, ValueRef};

impl_type! {
    impl Array {
        Constants:

        Functions(ctx, vm, program, area):


        /// fghfddggfd
        fn push(&mut Array(slf) as "self", elem) {
            slf.borrow_mut().push(vm.deep_clone_ref(elem));

            Multi::new_single(ctx, Ok(ValueRef::new(Value::Empty.into_stored(area))))
        }

        /// fgdfgdfg
        fn is_empty(&Array(slf) as "self") {
            let is_empty = slf.borrow().is_empty();
            Multi::new_single(ctx, Ok(ValueRef::new(Value::Bool(is_empty).into_stored(area))))
        }

    }
}
