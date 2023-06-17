use std::cell::RefCell;
use std::rc::Rc;

use super::bytecode::{ConstID, FuncID, OpcodePos, Register, UnoptRegister};

pub type UnoptOpcode = Opcode<UnoptRegister>;

#[derive(Debug)]
pub enum Opcode<R: Copy> {
    LoadConst { id: ConstID, to: R },
    CopyDeep { from: R, to: R },
    CopyMem { from: R, to: R },
    Plus { a: R, b: R, c: R },
    Jump { to: OpcodePos },
    FuncJump { to: FuncID },
    JumpIfFalse { check: R, to: OpcodePos },
    Ret,
}
