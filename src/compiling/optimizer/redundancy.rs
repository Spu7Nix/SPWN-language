use std::fmt::Display;

use ahash::AHashSet;

use crate::compiling::bytecode::Function;
use crate::interpreting::opcodes::{Opcode, OpcodePos, UnoptRegister};

pub fn optimize(func: &mut Function<UnoptRegister>) -> bool {
    let mut to_remove = AHashSet::new();

    for (i, opcode) in func.opcodes.iter().enumerate() {
        match opcode {
            Opcode::Copy { from, to } => {
                if from == to {
                    to_remove.insert(i as OpcodePos);
                }
            },
            _ => (),
        }
    }

    func.remove_opcodes(&to_remove);

    !to_remove.is_empty()
}

impl<T: Display + Copy> Opcode<T> {
    pub fn get_jumps(&mut self) -> Vec<&mut OpcodePos> {
        match self {
            Opcode::Jump { to } => vec![to],
            Opcode::JumpIfFalse { to, .. } => vec![to],
            Opcode::JumpIfEmpty { to, .. } => vec![to],
            Opcode::UnwrapOrJump { to, .. } => vec![to],

            Opcode::EnterArrowStatement { skip_to } => vec![skip_to],
            Opcode::StartTryCatch { .. } => todo!(),
            _ => vec![],
        }
    }
}
