use super::bytecode::Bytecode;
use crate::interpreting::opcodes::{Opcode, UnoptRegister};

mod util;

mod dead_code;
mod redundancy;
mod registers;

pub fn optimize_code(code: &mut Bytecode<UnoptRegister>) {
    loop {
        let mut changed = false;

        for func in &mut code.functions {
            changed |= registers::optimize(func);
            // changed |= redundancy::optimize(func);
            // changed |= dead_code::optimize(func);
        }

        if !changed {
            break;
        }
    }
}
