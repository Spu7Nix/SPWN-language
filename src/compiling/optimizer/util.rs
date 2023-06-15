use std::fmt::Display;

use ahash::AHashSet;

use crate::compiling::bytecode::Function;
use crate::interpreting::opcodes::{Opcode, OpcodePos, UnoptRegister};

impl Function<UnoptRegister> {
    pub fn remove_opcodes(&mut self, indexes: &AHashSet<OpcodePos>) {
        let mut new = vec![];
        let mut new_spans = vec![];

        let mut new_count = 0 as OpcodePos;
        let mut remap = Vec::with_capacity(self.opcodes.len());

        for (i, (opcode, span)) in self.opcodes.iter().zip(&self.opcode_spans).enumerate() {
            if indexes.contains(&(i as OpcodePos)) {
                remap.push(new_count);
            } else {
                new.push(*opcode);
                new_spans.push(*span);
                remap.push(new_count);
                new_count += 1;
            }
        }
        for opcode in &mut new {
            for jump in opcode.get_jumps() {
                *jump = remap[*jump as usize];
            }
        }

        self.opcodes = new;
        self.opcode_spans = new_spans;
    }
}

impl<T: Display + Copy> Opcode<T> {
    // getting successive opcodes to given opcode
    pub fn get_successors(&self, idx: OpcodePos, len: usize) -> Vec<OpcodePos> {
        let mut successors = match *self {
            Opcode::Jump { to } => return vec![to],
            Opcode::JumpIfFalse { src, to } => vec![to],
            Opcode::JumpIfEmpty { src, to } => vec![to],
            Opcode::UnwrapOrJump { src, to } => vec![to],
            Opcode::Ret { src, module_ret } => return vec![],
            Opcode::PushContextGroup { src } => todo!(),
            Opcode::PopGroupStack { fn_reg } => todo!(),
            Opcode::YeetContext => return vec![],
            Opcode::EnterArrowStatement { skip_to } => vec![skip_to],
            Opcode::Export { src } => todo!(),
            Opcode::Throw { err } => return vec![],
            Opcode::StartTryCatch { id, reg } => todo!(),
            _ => vec![],
        };
        if (idx as usize) < len - 1 {
            successors.push(idx + 1);
        }
        successors
    }

    pub fn get_key_change_regs(self) -> Vec<T> {
        match self {
            Opcode::Index { dest, .. }
            | Opcode::Member { dest, .. }
            | Opcode::Associated { dest, .. } => vec![dest],
            _ => vec![],
        }
    }

    pub fn get_read(self) -> Vec<T> {
        match self {
            Opcode::LoadConst { dest, id } => vec![],
            Opcode::Copy { from, to } => vec![from],
            Opcode::Dbg { reg } => vec![reg],
            Opcode::Call { base, args, dest } => vec![base, args],
            Opcode::AllocArray { size, dest } => vec![],
            Opcode::AllocDict { size, dest } => vec![],
            Opcode::AllocObject { size, dest } => vec![],
            Opcode::AllocTrigger { size, dest } => vec![],
            Opcode::PushArrayElem { elem, dest } => vec![dest, elem],
            Opcode::PushDictElem { elem, key, dest } => vec![dest, key, elem],
            Opcode::PushArrayElemByKey { elem, dest } => vec![elem, dest],
            Opcode::PushDictElemByKey { elem, key, dest } => vec![elem, dest, key],
            Opcode::MakeDictElemPrivate { dest, key } => vec![dest, key],
            Opcode::PushObjectElemKey {
                elem,
                obj_key,
                dest,
            } => vec![elem, dest],
            Opcode::PushObjectElemUnchecked {
                elem,
                obj_key,
                dest,
            } => vec![elem, dest],
            Opcode::CreateMacro { id, dest } => vec![],
            Opcode::PushMacroArg { name, dest, is_ref } => vec![name, dest],
            Opcode::SetMacroArgDefault { src, dest } => vec![dest, src],
            Opcode::SetMacroArgPattern { id, dest } => vec![dest],
            Opcode::PushMacroSpreadArg { name, dest } => vec![dest, name],
            Opcode::Add { left, right, dest } => vec![left, right],
            Opcode::Sub { left, right, dest } => vec![left, right],
            Opcode::Mult { left, right, dest } => vec![left, right],
            Opcode::Div { left, right, dest } => vec![left, right],
            Opcode::Mod { left, right, dest } => vec![left, right],
            Opcode::Pow { left, right, dest } => vec![left, right],
            Opcode::ShiftLeft { left, right, dest } => vec![left, right],
            Opcode::ShiftRight { left, right, dest } => vec![left, right],
            Opcode::BinOr { left, right, dest } => vec![left, right],
            Opcode::BinAnd { left, right, dest } => vec![left, right],
            Opcode::AddEq { left, right } => vec![left, right],
            Opcode::SubEq { left, right } => vec![left, right],
            Opcode::MultEq { left, right } => vec![left, right],
            Opcode::DivEq { left, right } => vec![left, right],
            Opcode::ModEq { left, right } => vec![left, right],
            Opcode::PowEq { left, right } => vec![left, right],
            Opcode::ShiftLeftEq { left, right } => vec![left, right],
            Opcode::ShiftRightEq { left, right } => vec![left, right],
            Opcode::BinAndEq { left, right } => vec![left, right],
            Opcode::BinOrEq { left, right } => vec![left, right],
            Opcode::Not { src, dest } => vec![src],
            Opcode::Negate { src, dest } => vec![src],
            Opcode::Eq { left, right, dest } => vec![left, right],
            Opcode::Neq { left, right, dest } => vec![left, right],
            Opcode::Gt { left, right, dest } => vec![left, right],
            Opcode::Lt { left, right, dest } => vec![left, right],
            Opcode::Gte { left, right, dest } => vec![left, right],
            Opcode::Lte { left, right, dest } => vec![left, right],
            Opcode::Range { left, right, dest } => vec![left, right],
            Opcode::In { left, right, dest } => vec![left, right],
            Opcode::As { left, right, dest } => vec![left, right],
            Opcode::And { left, right, dest } => vec![left, right],
            Opcode::Or { left, right, dest } => vec![left, right],
            Opcode::Jump { to } => vec![],
            Opcode::JumpIfFalse { src, to } => vec![src],
            Opcode::JumpIfEmpty { src, to } => vec![src],
            Opcode::UnwrapOrJump { src, to } => vec![src],
            Opcode::WrapIterator { src, dest } => vec![src],
            Opcode::IterNext { src, dest } => vec![src],
            Opcode::Ret { src, module_ret } => vec![src],
            Opcode::WrapMaybe { src, dest } => vec![src],
            Opcode::LoadNone { dest } => vec![],
            Opcode::LoadEmpty { dest } => vec![],
            Opcode::LoadEpsilon { dest } => vec![],
            Opcode::LoadEmptyDict { dest } => vec![],
            Opcode::LoadArbitraryId { class, dest } => vec![],
            Opcode::TypeOf { src, dest } => vec![src],
            Opcode::PushContextGroup { src } => vec![src],
            Opcode::PopGroupStack { fn_reg } => vec![fn_reg],
            Opcode::MakeTriggerFunc { src, dest } => vec![src],
            Opcode::CallTriggerFunc { func } => vec![func],
            Opcode::Index { base, dest, index } => vec![base, index],
            Opcode::Member { from, dest, member } => vec![from, member],
            Opcode::TypeMember { from, dest, member } => vec![from, member],
            Opcode::Associated { from, dest, name } => vec![from, name],
            Opcode::YeetContext => vec![],
            Opcode::EnterArrowStatement { skip_to } => vec![],
            Opcode::LoadBuiltins { dest } => vec![],
            Opcode::Export { src } => vec![src],
            Opcode::Import { src, dest } => vec![],
            Opcode::Throw { err } => vec![err],
            Opcode::CreateInstance { base, dict, dest } => vec![base, dict],
            Opcode::Impl { base, dict } => vec![base, dict],
            Opcode::Overload { array, op } => vec![array],
            Opcode::MakeByteArray { reg } => vec![reg],
            Opcode::StartTryCatch { id, reg } => todo!(),
        }
    }

    pub fn get_write(self) -> Vec<T> {
        match self {
            Opcode::LoadConst { dest, id } => vec![dest],
            Opcode::Copy { from, to } => vec![to],
            Opcode::Dbg { reg } => vec![],
            Opcode::Call { base, args, dest } => vec![dest],
            Opcode::AllocArray { size, dest } => vec![dest],
            Opcode::AllocDict { size, dest } => vec![dest],
            Opcode::AllocObject { size, dest } => vec![dest],
            Opcode::AllocTrigger { size, dest } => vec![dest],
            Opcode::PushArrayElem { elem, dest } => vec![dest],
            Opcode::PushDictElem { elem, key, dest } => vec![dest],
            Opcode::PushArrayElemByKey { elem, dest } => vec![dest],
            Opcode::PushDictElemByKey { elem, key, dest } => vec![dest],
            Opcode::MakeDictElemPrivate { dest, key } => vec![dest],
            Opcode::PushObjectElemKey {
                elem,
                obj_key,
                dest,
            } => vec![dest],
            Opcode::PushObjectElemUnchecked {
                elem,
                obj_key,
                dest,
            } => vec![dest],
            Opcode::CreateMacro { id, dest } => vec![dest],
            Opcode::PushMacroArg { name, dest, is_ref } => vec![dest],
            Opcode::SetMacroArgDefault { src, dest } => vec![dest],
            Opcode::SetMacroArgPattern { id, dest } => vec![dest],
            Opcode::PushMacroSpreadArg { name, dest } => vec![dest],
            Opcode::Add { left, right, dest } => vec![dest],
            Opcode::Sub { left, right, dest } => vec![dest],
            Opcode::Mult { left, right, dest } => vec![dest],
            Opcode::Div { left, right, dest } => vec![dest],
            Opcode::Mod { left, right, dest } => vec![dest],
            Opcode::Pow { left, right, dest } => vec![dest],
            Opcode::ShiftLeft { left, right, dest } => vec![dest],
            Opcode::ShiftRight { left, right, dest } => vec![dest],
            Opcode::BinOr { left, right, dest } => vec![dest],
            Opcode::BinAnd { left, right, dest } => vec![dest],
            Opcode::AddEq { left, right } => vec![left],
            Opcode::SubEq { left, right } => vec![left],
            Opcode::MultEq { left, right } => vec![left],
            Opcode::DivEq { left, right } => vec![left],
            Opcode::ModEq { left, right } => vec![left],
            Opcode::PowEq { left, right } => vec![left],
            Opcode::ShiftLeftEq { left, right } => vec![left],
            Opcode::ShiftRightEq { left, right } => vec![left],
            Opcode::BinAndEq { left, right } => vec![left],
            Opcode::BinOrEq { left, right } => vec![left],
            Opcode::Not { src, dest } => vec![dest],
            Opcode::Negate { src, dest } => vec![dest],
            Opcode::Eq { left, right, dest } => vec![dest],
            Opcode::Neq { left, right, dest } => vec![dest],
            Opcode::Gt { left, right, dest } => vec![dest],
            Opcode::Lt { left, right, dest } => vec![dest],
            Opcode::Gte { left, right, dest } => vec![dest],
            Opcode::Lte { left, right, dest } => vec![dest],
            Opcode::Range { left, right, dest } => vec![dest],
            Opcode::In { left, right, dest } => vec![dest],
            Opcode::As { left, right, dest } => vec![dest],
            Opcode::And { left, right, dest } => vec![dest],
            Opcode::Or { left, right, dest } => vec![dest],
            Opcode::Jump { to } => vec![],
            Opcode::JumpIfFalse { src, to } => vec![],
            Opcode::JumpIfEmpty { src, to } => vec![],
            Opcode::UnwrapOrJump { src, to } => vec![],
            Opcode::WrapIterator { src, dest } => vec![dest],
            Opcode::IterNext { src, dest } => vec![dest],
            Opcode::Ret { src, module_ret } => vec![],
            Opcode::WrapMaybe { src, dest } => vec![dest],
            Opcode::LoadNone { dest } => vec![dest],
            Opcode::LoadEmpty { dest } => vec![dest],
            Opcode::LoadEpsilon { dest } => vec![dest],
            Opcode::LoadEmptyDict { dest } => vec![dest],
            Opcode::LoadArbitraryId { class, dest } => vec![dest],
            Opcode::TypeOf { src, dest } => vec![dest],
            Opcode::PushContextGroup { src } => vec![],
            Opcode::PopGroupStack { fn_reg } => vec![],
            Opcode::MakeTriggerFunc { src, dest } => vec![dest],
            Opcode::CallTriggerFunc { func } => vec![],
            Opcode::Index { base, dest, index } => vec![dest],
            Opcode::Member { from, dest, member } => vec![dest],
            Opcode::TypeMember { from, dest, member } => vec![dest],
            Opcode::Associated { from, dest, name } => vec![dest],
            Opcode::YeetContext => vec![],
            Opcode::EnterArrowStatement { skip_to } => vec![],
            Opcode::LoadBuiltins { dest } => vec![dest],
            Opcode::Export { src } => vec![],
            Opcode::Import { src, dest } => vec![dest],
            Opcode::Throw { err } => vec![],
            Opcode::CreateInstance { base, dict, dest } => vec![dest],
            Opcode::Impl { base, dict } => vec![],
            Opcode::Overload { array, op } => vec![],
            Opcode::MakeByteArray { reg } => vec![reg],
            Opcode::StartTryCatch { id, reg } => todo!(),
        }
    }
}
