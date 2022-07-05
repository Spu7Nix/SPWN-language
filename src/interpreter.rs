use std::collections::HashMap;

use slotmap::{new_key_type, SlotMap};

use crate::{
    compiler::{Code, Instruction},
    contexts::FullContext,
    sources::CodeArea,
    value::{value_ops, Value},
};

new_key_type! {
    pub struct ValueKey;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub def_area: CodeArea,
}

pub struct Globals {
    pub memory: SlotMap<ValueKey, StoredValue>,

    pub contexts: FullContext,
}

// just a test

// pub fn execute(globals: &mut Globals, code: &Code, func: usize) {
//     let mut stack: Vec<*mut StoredValue> = vec![];

//     let mut i = 0;

//     while i < code.instructions[func].len() {
//         match &code.instructions[func][i] {
//             Instruction::LoadConst(id) => {
//                 let area = code.get_instr_area(func, i).into_simple();
//                 let key = globals
//                     .memory
//                     .insert(code.constants.get(*id).clone().into_stored(area));
//                 stack.push(&mut globals.memory[key]);
//             }
//             Instruction::Plus => {
//                 let area = code.get_instr_area(func, i).into_simple();
//                 let b = stack.pop().unwrap();
//                 let a = stack.pop().unwrap();
//                 let key = unsafe { globals.memory.insert(value_ops::plus(&*a, &*b, area)) };
//                 stack.push(&mut globals.memory[key]);
//             }
//             Instruction::Minus => {
//                 let area = code.get_instr_area(func, i).into_simple();
//                 let b = stack.pop().unwrap();
//                 let a = stack.pop().unwrap();
//                 let key = unsafe { globals.memory.insert(value_ops::minus(&*a, &*b, area)) };
//                 stack.push(&mut globals.memory[key]);
//             }
//             Instruction::Mult => {
//                 let area = code.get_instr_area(func, i).into_simple();
//                 let b = stack.pop().unwrap();
//                 let a = stack.pop().unwrap();
//                 let key = unsafe { globals.memory.insert(value_ops::mult(&*a, &*b, area)) };
//                 stack.push(&mut globals.memory[key]);
//             }
//             Instruction::Div => {
//                 let area = code.get_instr_area(func, i).into_simple();
//                 let b = stack.pop().unwrap();
//                 let a = stack.pop().unwrap();
//                 let key = unsafe { globals.memory.insert(value_ops::div(&*a, &*b, area)) };
//                 stack.push(&mut globals.memory[key]);
//             }
//             Instruction::Mod => todo!(),
//             Instruction::Pow => todo!(),
//             Instruction::Eq => todo!(),
//             Instruction::NotEq => todo!(),
//             Instruction::Greater => todo!(),
//             Instruction::GreaterEq => todo!(),
//             Instruction::Lesser => todo!(),
//             Instruction::LesserEq => todo!(),
//             Instruction::Assign => todo!(),
//             Instruction::Negate => todo!(),
//             Instruction::LoadVar(_) => todo!(),
//             Instruction::SetVar(_, _) => todo!(),
//             Instruction::LoadType(_) => todo!(),
//             Instruction::BuildArray(_) => todo!(),
//             Instruction::PushEmpty => todo!(),
//             Instruction::PopTop => {
//                 // stack.pop();
//             }
//             Instruction::Jump(_) => todo!(),
//             Instruction::JumpIfFalse(_) => todo!(),
//             Instruction::ToIter => todo!(),
//             Instruction::IterNext(_) => todo!(),
//             Instruction::DeriveScope => todo!(),
//             Instruction::PopScope => todo!(),
//             Instruction::BuildDict(_) => todo!(),
//             Instruction::Return => todo!(),
//             Instruction::Continue => todo!(),
//             Instruction::Break => todo!(),
//             Instruction::MakeMacro(_) => todo!(),
//             Instruction::PushAnyPattern => todo!(),
//             Instruction::MakeMacroPattern(_) => todo!(),
//             Instruction::Index => todo!(),
//             Instruction::Call(_) => todo!(),
//             Instruction::SaveContexts => todo!(),
//             Instruction::ReviseContexts => todo!(),
//             Instruction::MergeContexts => {}
//             Instruction::PushNone => todo!(),
//             Instruction::WrapMaybe => todo!(),
//             Instruction::PushContextGroup => todo!(),
//             Instruction::PopContextGroup => todo!(),
//             Instruction::PushTriggerFnValue => todo!(),
//         }
//         i += 1
//     }

//     unsafe {
//         println!(
//             "stack: {}",
//             stack
//                 .iter()
//                 .map(|s| format!("{:?}", (**s).value))
//                 .collect::<Vec<_>>()
//                 .join(", ")
//         );
//     }
// }
