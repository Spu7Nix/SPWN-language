use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use crate::{
    compiler::{Code, Instruction},
    contexts::FullContext,
    error::RuntimeError,
    sources::CodeArea,
    value::{value_ops, Value},
};

new_key_type! {
    pub struct ValueKey;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub def_area: CodeArea,
}

pub struct Globals {
    pub memory: SlotMap<ValueKey, StoredValue>,

    pub contexts: FullContext,
}

pub fn execute(globals: &mut Globals, code: &Code, func: usize) -> Result<(), RuntimeError> {
    let mut stack: Vec<*mut StoredValue> = vec![];

    let mut i = 0;

    while i < code.instructions[func].len() {
        match &code.instructions[func][i] {
            Instruction::LoadConst(id) => {
                let area = code.get_instr_area(func, i).into_simple();
                let key = globals
                    .memory
                    .insert(code.constants.get(*id).clone().into_stored(area));
                stack.push(&mut globals.memory[key]);
            }
            Instruction::Plus => {
                let area = code.get_instr_area(func, i).into_simple();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let key = unsafe { globals.memory.insert(value_ops::plus(&*a, &*b, area)?) };
                stack.push(&mut globals.memory[key]);
            }
            Instruction::Minus => {
                let area = code.get_instr_area(func, i).into_simple();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let key = unsafe { globals.memory.insert(value_ops::minus(&*a, &*b, area)?) };
                stack.push(&mut globals.memory[key]);
            }
            Instruction::Mult => {
                let area = code.get_instr_area(func, i).into_simple();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let key = unsafe { globals.memory.insert(value_ops::mult(&*a, &*b, area)?) };
                stack.push(&mut globals.memory[key]);
            }
            Instruction::Div => {
                let area = code.get_instr_area(func, i).into_simple();
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let key = unsafe { globals.memory.insert(value_ops::div(&*a, &*b, area)?) };
                stack.push(&mut globals.memory[key]);
            }
            Instruction::PopTop => {}
            Instruction::MergeContexts => {}
            _ => todo!(),
        }
        i += 1
    }

    unsafe {
        println!(
            "stack: {}",
            stack
                .iter()
                .map(|s| format!("{:?}", (**s).value))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(())
}
