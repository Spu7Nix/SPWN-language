use crate::interpreting::builtins::impl_type;

impl_type! {
    impl Array {
        /// dsf "fuck" 😨
        fn push(& Array(v) as "self", elem) {
            v.borrow_mut().push(elem)
        }
    }
}
