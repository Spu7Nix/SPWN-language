use std::cell::RefCell;
use std::rc::Rc;

use super::bytecode::{ConstID, FuncID, OpcodePos, OptRegister, Register, UnoptRegister};

pub type UnoptOpcode = Opcode<UnoptRegister>;
pub type OptOpcode = Opcode<OptRegister>;

#[derive(Debug, Clone, Copy)]
pub enum Opcode<R: Copy> {
    LoadConst { id: ConstID, to: R },
    CopyDeep { from: R, to: R },
    CopyMem { from: R, to: R },
    Plus { a: R, b: R, c: R },
    Jump { to: OpcodePos },
    FuncJump { to: FuncID },
    JumpIfFalse { check: R, to: OpcodePos },
    Ret,
    AllocArray { reg: R, len: u16 },
    PushArrayElem { elem: R, dest: R },
    AllocDict { reg: R, capacity: u16 },
    InsertDictElem { elem: R, dest: R, key: R },
}
