use super::bytecode::Bytecode;
use crate::interpreting::opcodes::{Opcode, UnoptRegister};

mod util;

mod dead_code;
mod redundancy;
mod registers;

pub fn optimize_code(code: &mut Bytecode<UnoptRegister>) {
    #[allow(clippy::never_loop)]
    loop {
        let mut changed = false;

        fn visit(code: &mut Bytecode<UnoptRegister>, func: u16, changed: &mut bool) {
            for child in code.functions[func as usize].inner_funcs.clone() {
                let id = child;
                visit(code, id, changed)
            }

            println!("jw {}", func);
            *changed |= registers::optimize(code, func);
            // *changed |= redundancy::optimize(&mut (*code).functions[func as usize]);
            // *changed |= dead_code::optimize(&mut (*code).functions[func as usize]);
        }

        visit(code, 0, &mut changed);

        // if !changed {
        break;
        // }
    }
    // for func in &mut code.functions {
    //     println!("{:#?}", func);
    // }
}
