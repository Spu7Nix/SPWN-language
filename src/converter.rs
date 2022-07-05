use crate::compiler::{Code, Instruction};

pub fn to_bytes(code: &Code) -> Vec<u8> {
    bincode::serialize(code).unwrap()
}

pub fn from_bytes(bytes: &Vec<u8>) -> Code {
    bincode::deserialize(bytes).unwrap()
}
