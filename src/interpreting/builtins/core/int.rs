use crate::interpreting::builtins::impl_type;
use crate::interpreting::multi::Multi;
use crate::interpreting::value::Value;
use crate::interpreting::vm::{DeepClone, ValueRef};

impl_type! {
    impl Int {
        Constants:

        Functions:


        /// fghfddggfd
        fn max(Int(slf) as "self", other: Int | Float) {
            other! {
                Int(n) => {
                    if *slf.borrow() > *n.borrow() {
                        return Multi::new_single(ctx, Ok(slf.get_ref().clone()))
                    }
                    return Multi::new_single(ctx, Ok(n.get_ref().clone()))
                },
                Float(n) => {
                    if *slf.borrow() as f64 > *n.borrow() {
                        return Multi::new_single(ctx, Ok(slf.get_ref().clone()))
                    }
                    return Multi::new_single(ctx, Ok(n.get_ref().clone()))
                }
            }
            unreachable!()
        }

    }
}
