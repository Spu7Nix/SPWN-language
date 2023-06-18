use std::hash::Hash;

use derive_more::{Deref, DerefMut};

use super::opcodes::{Opcode, OptOpcode};
use crate::util::{Digest, ImmutStr, ImmutVec};

#[derive(Clone, Debug, derive_more::From)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ImmutStr),
}

impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Constant::Int(v) => v.hash(state),
            Constant::Float(v) => v.to_bits().hash(state),
            Constant::Bool(v) => v.hash(state),
            Constant::String(v) => v.hash(state),
        }
    }
}
impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0.to_bits() == r0.to_bits(),
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl Eq for Constant {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut)]
pub struct Register<T: Copy>(pub T);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstID(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpcodePos(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncID(pub u16);

pub type UnoptRegister = Register<usize>;
pub type OptRegister = Register<u8>;

pub struct Bytecode<R: Copy> {
    pub source_hash: Digest,
    pub version: ImmutStr,
    constants: ImmutVec<Constant>,

    opcodes: ImmutVec<Opcode<R>>,
}
