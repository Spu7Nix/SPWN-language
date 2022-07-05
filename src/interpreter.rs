use std::collections::HashMap;

use slotmap::{new_key_type, SlotMap};

use crate::{
    compiler::{Code, Instruction},
    contexts::FullContext,
    sources::CodeArea,
    value::Value,
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
    memory: SlotMap<ValueKey, StoredValue>,

    contexts: FullContext,
}

pub fn execute(globals: &mut Globals, code: &Code, func: usize) {
    // let mut stack = vec![];

    for i in &code.instructions[func] {
        match i {
            Instruction::LoadConst(id) => todo!(),
            Instruction::Plus => todo!(),
            Instruction::Minus => todo!(),
            Instruction::Mult => todo!(),
            Instruction::Div => todo!(),
            Instruction::Mod => todo!(),
            Instruction::Pow => todo!(),
            Instruction::Eq => todo!(),
            Instruction::NotEq => todo!(),
            Instruction::Greater => todo!(),
            Instruction::GreaterEq => todo!(),
            Instruction::Lesser => todo!(),
            Instruction::LesserEq => todo!(),
            Instruction::Assign => todo!(),
            Instruction::Negate => todo!(),
            Instruction::LoadVar(_) => todo!(),
            Instruction::SetVar(_, _) => todo!(),
            Instruction::LoadType(_) => todo!(),
            Instruction::BuildArray(_) => todo!(),
            Instruction::PushEmpty => todo!(),
            Instruction::PopTop => todo!(),
            Instruction::Jump(_) => todo!(),
            Instruction::JumpIfFalse(_) => todo!(),
            Instruction::ToIter => todo!(),
            Instruction::IterNext(_) => todo!(),
            Instruction::DeriveScope => todo!(),
            Instruction::PopScope => todo!(),
            Instruction::BuildDict(_) => todo!(),
            Instruction::Return => todo!(),
            Instruction::Continue => todo!(),
            Instruction::Break => todo!(),
            Instruction::MakeMacro(_) => todo!(),
            Instruction::PushAnyPattern => todo!(),
            Instruction::MakeMacroPattern(_) => todo!(),
            Instruction::Index => todo!(),
            Instruction::Call(_) => todo!(),
            Instruction::SaveContexts => todo!(),
            Instruction::ReviseContexts => todo!(),
            Instruction::MergeContexts => todo!(),
            Instruction::PushNone => todo!(),
            Instruction::WrapMaybe => todo!(),
        }
    }
}
