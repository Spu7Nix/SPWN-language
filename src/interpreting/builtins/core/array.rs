use crate::compiling::bytecode::{CallExpr, Register};
use crate::interpreting::builtins::impl_type;
use crate::interpreting::value::Value;

impl_type! {
    impl Array {
        Constants:
        /// tEst
        const A = Int(2);

        Functions(vm, program, area):
        /// dsf "fuck"
        fn push(&mut Array(v) as "self", bink) {
            v.borrow_mut().push(bink.clone());
        }

        /// dsf "fuck"
        fn peniscum(m: Macro) {
            {
                let m = m.clone();
                vm.call_macro(
                    |vm| vm.get_reg_ref(Register(0u8)),
                    &CallExpr {
                        dest: Register(255u8),
                        positional: Box::new([]),
                        named: Box::new([])
                    },
                    program,
                    area
                )?;
            }
            // vm.run_macro(&CallExpr {base, dest, positional, named}, program, area);
            // vm;

            // for v in &sinky {
            //     println!("{}", v.0.borrow());
            // }
            // v.borrow_mut().push(bink.clone());
        }
    }
}
