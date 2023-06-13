use crate::compiling::bytecode::Bytecode;
use crate::interpreting::opcodes::UnoptRegister;

pub fn fold_constants(bytecode: &mut Bytecode<UnoptRegister>) {
    for function in &bytecode.functions {}
}
