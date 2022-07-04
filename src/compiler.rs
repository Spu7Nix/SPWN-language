use crate::value::Value;

pub struct Code {
    pub constants: Vec<Value>,
    pub instructions: Vec<Vec<Instruction>>,
}

pub enum Instruction {
    LoadConst(u32),

    Plus,
    Minus,
    Mult,
    Div,
    // other shit soon
}
