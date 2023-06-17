use std::hash::Hash;

#[derive(Clone, Debug)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Box<str>),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Register<T: Copy>(pub T);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstID(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpcodePos(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncID(pub u16);

pub type UnoptRegister = Register<usize>;

impl<T: Copy> core::ops::Deref for Register<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Copy> core::ops::DerefMut for Register<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl core::ops::Deref for ConstID {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for ConstID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl core::ops::Deref for OpcodePos {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for OpcodePos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl core::ops::Deref for FuncID {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for FuncID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
