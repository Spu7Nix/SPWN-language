use std::fmt::Display;

use super::bytecode::{Bytecode, RegNum, Register, UnoptBytecode};
use super::opcodes::{Opcode, OpcodePos};
use crate::compiling::opcodes::FuncID;

pub mod register;
mod util;

pub fn optimize_code(code: &mut UnoptBytecode) {
    #[allow(clippy::never_loop)]
    loop {
        let mut changed = false;

        // for func in 0..code.functions.len() {
        //     changed |= register::optimize(code, func.into())
        // }

        fn visit(code: &mut UnoptBytecode, func: FuncID, changed: &mut bool) {
            for &child in code.functions[*func as usize].child_funcs.clone().iter() {
                visit(code, child, changed)
            }

            println!("jw {}", func);
            // if func == 1 {
            *changed |= register::optimize(code, func);
            // }
            // *changed |= redundancy::optimize(&mut (*code).functions[func as usize]);
            // *changed |= dead_code::optimize(&mut (*code).functions[func as usize]);
        }

        visit(code, FuncID(0), &mut changed);

        // if !changed {
        break;
        // }
    }
    // for func in &mut code.functions {
    //     println!("{:#?}", func);
    // }
}
