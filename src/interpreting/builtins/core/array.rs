use crate::interpreting::builtins::impl_type;
use crate::interpreting::value::Value;

impl_type! {
    impl Array {
        Constants:
        /// tEst
        const A = Int(2);

        Functions:
        /// dsf "fuck"
        fn push(&mut Array(v) as "self", bink) {
            v.borrow_mut().push(bink.clone());
        }

        /// dsf "fuck"
        fn peniscum(...sinky: Int) {

            for v in &sinky {
                println!("{}", v.0.borrow());
            }
            // v.borrow_mut().push(bink.clone());
        }
    }
}
