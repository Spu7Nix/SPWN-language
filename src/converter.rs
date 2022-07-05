use crate::compiler::{Code, Instruction};

pub fn opcode_id(i: &Instruction) -> u8 {
    match i {
        Instruction::LoadConst(_) => 1,
        Instruction::Plus => 2,
        Instruction::Minus => 3,
        Instruction::Mult => 4,
        Instruction::Div => 5,
        Instruction::Mod => 6,
        Instruction::Pow => 7,
        Instruction::Eq => 8,
        Instruction::NotEq => 9,
        Instruction::Greater => 10,
        Instruction::GreaterEq => 11,
        Instruction::Lesser => 12,
        Instruction::LesserEq => 13,
        Instruction::Assign => 14,
        Instruction::Negate => 15,
        Instruction::LoadVar(_) => 16,
        Instruction::SetVar(_, _) => 17,
        Instruction::LoadType(_) => 18,
        Instruction::BuildArray(_) => 19,
        Instruction::PushEmpty => 20,
        Instruction::PopTop => 21,
        Instruction::Jump(_) => 22,
        Instruction::JumpIfFalse(_) => 23,
        Instruction::ToIter => 24,
        Instruction::IterNext(_) => 25,
        Instruction::DeriveScope => 26,
        Instruction::PopScope => 27,
        Instruction::BuildDict(_) => 28,
        Instruction::Return => 29,
        Instruction::Continue => 30,
        Instruction::Break => 31,
        Instruction::MakeMacro(_) => 32,
        Instruction::PushAnyPattern => 33,
        Instruction::MakeMacroPattern(_) => 34,
        Instruction::Index => 35,
        Instruction::Call(_) => 36,
        Instruction::TriggerFuncCall => 37,
        Instruction::SaveContexts => 38,
        Instruction::ReviseContexts => 39,
        Instruction::MergeContexts => 40,
        Instruction::PushNone => 41,
        Instruction::WrapMaybe => 42,
        Instruction::PushContextGroup => 43,
        Instruction::PopContextGroup => 44,
        Instruction::PushTriggerFnValue => 45,
        Instruction::TypeDef(_) => 46,
        Instruction::Impl(_) => 47,
        Instruction::Instance(_) => 48,
    }
}

pub fn to_bytes(code: &Code) -> Vec<u8> {
    let mut bytes = vec![];

    for func in &code.instructions {
        for i in func {
            match i {
                Instruction::LoadConst(id)
                | Instruction::LoadVar(id)
                | Instruction::LoadType(id)
                | Instruction::BuildArray(id)
                | Instruction::Jump(id)
                | Instruction::JumpIfFalse(id)
                | Instruction::IterNext(id)
                | Instruction::BuildDict(id)
                | Instruction::MakeMacro(id)
                | Instruction::MakeMacroPattern(id)
                | Instruction::Call(id)
                | Instruction::TypeDef(id)
                | Instruction::Impl(id)
                | Instruction::Instance(id) => {
                    bytes.push(opcode_id(i));
                    let [a, b] = id.to_be_bytes();
                    bytes.push(a as u8);
                    bytes.push(b as u8);
                }
                Instruction::SetVar(id, m) => {
                    bytes.push(opcode_id(i));
                    let [a, b] = id.to_be_bytes();
                    bytes.push(a as u8);
                    bytes.push(b as u8);
                    bytes.push(*m as u8);
                }
                Instruction::Plus
                | Instruction::Minus
                | Instruction::Mult
                | Instruction::Div
                | Instruction::Mod
                | Instruction::Pow
                | Instruction::Eq
                | Instruction::NotEq
                | Instruction::Greater
                | Instruction::GreaterEq
                | Instruction::Lesser
                | Instruction::LesserEq
                | Instruction::Assign
                | Instruction::Negate
                | Instruction::PushEmpty
                | Instruction::PopTop
                | Instruction::ToIter
                | Instruction::DeriveScope
                | Instruction::PopScope
                | Instruction::Return
                | Instruction::Continue
                | Instruction::Break
                | Instruction::PushAnyPattern
                | Instruction::Index
                | Instruction::TriggerFuncCall
                | Instruction::SaveContexts
                | Instruction::ReviseContexts
                | Instruction::MergeContexts
                | Instruction::PushNone
                | Instruction::WrapMaybe
                | Instruction::PushContextGroup
                | Instruction::PopContextGroup
                | Instruction::PushTriggerFnValue => {
                    bytes.push(opcode_id(i));
                }
            }
        }
        bytes.push(255);
    }

    bytes
}
