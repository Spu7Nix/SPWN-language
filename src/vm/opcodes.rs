use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};

pub type Register = u8;
pub type ConstID = u16;
pub type JumpPos = u16;
pub type AllocSize = u16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    LoadConst {
        dest: Register,
        id: ConstID,
    },
    // LoadBuiltin {},

    // Call {},
    AllocArray {
        size: AllocSize,
        dest: Register,
    },
    AllocDict {
        size: AllocSize,
        dest: Register,
    },

    Push {
        elem: Register,
        dest: Register,
    },

    Add {
        left: Register,
        right: Register,
        dest: Register,
    },
    Sub {
        left: Register,
        right: Register,
        dest: Register,
    },
    Mult {
        left: Register,
        right: Register,
        dest: Register,
    },
    Div {
        left: Register,
        right: Register,
        dest: Register,
    },
    Pow {
        left: Register,
        right: Register,
        dest: Register,
    },

    Eq {
        left: Register,
        right: Register,
        dest: Register,
    },
    Neq {
        left: Register,
        right: Register,
        dest: Register,
    },

    Gt {
        src: Register,
        dest: Register,
    },
    Lt {
        src: Register,
        dest: Register,
    },
    Gte {
        src: Register,
        dest: Register,
    },
    Lte {
        src: Register,
        dest: Register,
    },

    And {
        left: Register,
        right: Register,
        dest: Register,
    },
    Or {
        left: Register,
        right: Register,
        dest: Register,
    },
    Not {
        src: Register,
        dest: Register,
    },
    ShifLeft {
        src: Register,
        dest: Register,
    },
    ShiftRight {
        src: Register,
        dest: Register,
    },

    Jump {
        to: JumpPos,
    },
    JumpIfFalse {
        src: Register,
        to: JumpPos,
    },

    Ret {
        src: Register,
    },

    YeetContext,
    EnterArrowStatement {
        skip_to: JumpPos,
    },
}

impl Serialize for Opcode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Safety:
        // opcodes will always be u32 or less
        serializer.serialize_u32(unsafe { std::mem::transmute::<_, u32>(*self) })
    }
}

impl<'de> Deserialize<'de> for Opcode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u32(OpcodeVisitor)
    }
}

struct OpcodeVisitor;

impl<'de> Visitor<'de> for OpcodeVisitor {
    type Value = Opcode;

    fn expecting(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        panic!("idk")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // Safety:
        // who is manually writing bytecode
        Ok(unsafe { std::mem::transmute::<_, Opcode>(value) })
    }
}
