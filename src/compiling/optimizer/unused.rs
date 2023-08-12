use ahash::AHashSet;

use crate::compiling::bytecode::{UnoptBytecode, UnoptFunction, UnoptRegister};
use crate::compiling::opcodes::{FuncID, Opcode};

fn get_all_used(func: &UnoptFunction, code: &UnoptBytecode) -> AHashSet<UnoptRegister> {
    let mut used = AHashSet::new();

    for opcode in func.opcodes.iter() {
        for r in opcode.get_used_regs() {
            used.insert(*r);
        }
        #[allow(clippy::single_match)]
        match opcode.value {
            Opcode::Call { call, .. } => {
                used.extend(code.call_exprs[*call as usize].get_regs().copied())
            },
            _ => {},
        }
    }

    used
}

pub fn optimize(code: &mut UnoptBytecode, func_id: FuncID) -> bool {
    let mut changed = false;

    let used = get_all_used(&code.functions[*func_id as usize], &*code);

    let func = &mut code.functions[*func_id as usize];

    func.captured_regs.retain(|(_, r)| {
        if !used.contains(r) {
            // println!("ahulynman egaga");
            changed = true;
            return false;
        }
        true
    });

    changed
}
