use std::fmt::Display;

use super::bytecode::{Bytecode, RegNum, Register, UnoptBytecode};
use super::opcodes::{Opcode, OpcodePos};

pub mod register;
mod util;

pub fn optimize_code(code: &mut UnoptBytecode) {
    #[allow(clippy::never_loop)]
    loop {
        let mut changed = false;

        for func in 0..code.functions.len() {
            changed |= register::optimize(code, func.into())
        }

        // fn visit(code: &mut Bytecode<UnoptRegister>, func: u16, changed: &mut bool) {
        //     for child in code.functions[func as usize].inner_funcs.clone() {
        //         let id = child;
        //         visit(code, id, changed)
        //     }

        //     println!("jw {}", func);
        //     // if func == 1 {
        //     //*changed |= registers::optimize(code, func);
        //     // }
        //     // *changed |= redundancy::optimize(&mut (*code).functions[func as usize]);
        //     // *changed |= dead_code::optimize(&mut (*code).functions[func as usize]);
        // }

        // visit(code, 0, &mut changed);

        // if !changed {
        break;
        // }
    }
    // for func in &mut code.functions {
    //     println!("{:#?}", func);
    // }
}
