use slotmap::{new_key_type, SlotMap};

use crate::compiling::bytecode::Bytecode;

use super::{opcodes::Opcode, value::Value};

new_key_type! {
    pub struct ValueKey;
}

struct Vm<'a> {
    registers: [ValueKey; 255],

    memory: SlotMap<ValueKey, Value>,

    program: &'a Bytecode,
    // sp: usize,
    // pc: usize,
}

impl<'a> Vm<'a> {
    pub fn deep_clone_value(&mut self, k: ValueKey) -> ValueKey {
        let value = match self.memory[k].clone() {
            Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| self.deep_clone_value(v)).collect())
            }
            v => v,
        };
        self.memory.insert(value)
    }

    pub fn run_func(&mut self, func: usize) {
        let opcodes = &self.program.functions[func].opcodes;

        let mut i = 0_usize;

        while i < opcodes.len() {
            match &opcodes[i] {
                Opcode::LoadConst { dest, id } => {
                    self.registers[*dest as usize] = self
                        .memory
                        .insert(Value::from_const(&self.program.consts[*id as usize]))
                }
                Opcode::Copy { from, to } => {
                    self.registers[*to as usize] =
                        self.deep_clone_value(self.registers[*from as usize])
                }
                Opcode::Print { reg } => {
                    println!("{:?}", self.registers[*reg as usize])
                }
                Opcode::AllocArray { size, dest } => {
                    self.registers[*dest as usize] = self
                        .memory
                        .insert(Value::Array(Vec::with_capacity(*size as usize)))
                }
                Opcode::AllocDict { size, dest } => todo!(),
                Opcode::PushArrayElem { elem, dest } => {
                    let push = self.deep_clone_value(self.registers[*elem as usize]);
                    match &mut self.memory[self.registers[*dest as usize]] {
                        Value::Array(v) => v.push(push),
                        _ => panic!("sholdnt happe!!!n!!! Real ........"),
                    }
                }
                Opcode::PushDictElem { elem, key, dest } => todo!(),
                Opcode::Add { left, right, dest } => todo!(),
                Opcode::Sub { left, right, dest } => todo!(),
                Opcode::Mult { left, right, dest } => todo!(),
                Opcode::Div { left, right, dest } => todo!(),
                Opcode::Mod { left, right, dest } => todo!(),
                Opcode::Pow { left, right, dest } => todo!(),
                Opcode::ShiftLeft { left, right, dest } => todo!(),
                Opcode::ShiftRight { left, right, dest } => todo!(),
                Opcode::BinOr { left, right, dest } => todo!(),
                Opcode::BinAnd { left, right, dest } => todo!(),
                Opcode::AddEq { left, right } => todo!(),
                Opcode::SubEq { left, right } => todo!(),
                Opcode::MultEq { left, right } => todo!(),
                Opcode::DivEq { left, right } => todo!(),
                Opcode::ModEq { left, right } => todo!(),
                Opcode::PowEq { left, right } => todo!(),
                Opcode::ShiftLeftEq { left, right } => todo!(),
                Opcode::ShiftRightEq { left, right } => todo!(),
                Opcode::BinAndEq { left, right } => todo!(),
                Opcode::BinOrEq { left, right } => todo!(),
                Opcode::BinNotEq { left, right } => todo!(),
                Opcode::Not { src, dest } => todo!(),
                Opcode::Negate { src, dest } => todo!(),
                Opcode::BinNot { src, dest } => todo!(),
                Opcode::Eq { left, right, dest } => todo!(),
                Opcode::Neq { left, right, dest } => todo!(),
                Opcode::Gt { left, right, dest } => todo!(),
                Opcode::Lt { left, right, dest } => todo!(),
                Opcode::Gte { left, right, dest } => todo!(),
                Opcode::Lte { left, right, dest } => todo!(),
                Opcode::Range { left, right, dest } => todo!(),
                Opcode::In { left, right, dest } => todo!(),
                Opcode::As { left, right, dest } => todo!(),
                Opcode::Is { left, right, dest } => todo!(),
                Opcode::And { left, right, dest } => todo!(),
                Opcode::Or { left, right, dest } => todo!(),
                Opcode::Jump { to } => todo!(),
                Opcode::JumpIfFalse { src, to } => todo!(),
                Opcode::Ret { src } => todo!(),
                Opcode::WrapMaybe { src, dest } => todo!(),
                Opcode::LoadNone { dest } => todo!(),
                Opcode::LoadEmpty { dest } => todo!(),
                Opcode::Index { from, dest, index } => todo!(),
                Opcode::Member { from, dest, member } => todo!(),
                Opcode::Associated { from, dest, name } => todo!(),
                Opcode::YeetContext => todo!(),
                Opcode::EnterArrowStatement { skip_to } => todo!(),
                Opcode::LoadBuiltins { dest } => todo!(),
                Opcode::Export { src } => todo!(),
            }
        }
    }
}
