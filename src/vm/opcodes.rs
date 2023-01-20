use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};

use delve::{EnumDisplay, EnumFields, EnumToStr, EnumVariantNames};

pub type Register = u8;
pub type ConstID = u16;
pub type JumpPos = u16;
pub type AllocSize = u16;

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, EnumDisplay, EnumToStr, EnumFields, EnumVariantNames,
)]
#[delve(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Opcode {
    LoadConst {
        dest: Register,
        id: ConstID,
    },

    #[delve(display = |f: &Register, t: &Register| format!("R{f} -> R{t}"))]
    Copy {
        from: Register,
        to: Register,
    },
    #[delve(display = |reg: &Register| format!("print R{reg}"))]
    Print {
        reg: Register,
    },
    // LoadBuiltin {},

    // Call {},
    #[delve(display = |s: &AllocSize, d: &Register| format!("[...; {s}] -> R{d}"))]
    AllocArray {
        size: AllocSize,
        dest: Register,
    },
    #[delve(display = |s: &AllocSize, d: &Register| format!("{{...; {s}}} -> R{d}"))]
    AllocDict {
        size: AllocSize,
        dest: Register,
    },

    #[delve(display = |e: &Register, d: &Register| format!("R{d}.push R{e}"))]
    PushArrayElem {
        elem: Register,
        dest: Register,
    },
    #[delve(display = |e: &Register, k: &Register, d: &Register| format!("R{d}.insert R{k}: R{d}"))]
    PushDictElem {
        elem: Register,
        key: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} + R{b} -> R{x}"))]
    Add {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} - R{b} -> R{x}"))]
    Sub {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} * R{b} -> R{x}"))]
    Mult {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} / R{b} -> R{x}"))]
    Div {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} % R{b} -> R{x}"))]
    Mod {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} ^ R{b} -> R{x}"))]
    Pow {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} << R{b} -> R{x}"))]
    ShiftLeft {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} >> R{b} -> R{x}"))]
    ShiftRight {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} | R{b} -> R{x}"))]
    BinOr {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} & R{b} -> R{x}"))]
    BinAnd {
        left: Register,
        right: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register| format!("R{a} += R{b}"))]
    AddEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} -= R{b}"))]
    SubEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} *= R{b}"))]
    MultEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} /= R{b}"))]
    DivEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} %= R{b}"))]
    ModEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} ^= R{b}"))]
    PowEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} <<= R{b}"))]
    ShiftLeftEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} >>= R{b}"))]
    ShiftRightEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} &= R{b}"))]
    BinAndEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} |= R{b}"))]
    BinOrEq {
        left: Register,
        right: Register,
    },
    #[delve(display = |a: &Register, b: &Register| format!("R{a} ~= R{b}"))]
    BinNotEq {
        left: Register,
        right: Register,
    },

    #[delve(display = |s: &Register, d: &Register| format!("!R{s} -> R{d}"))]
    Not {
        src: Register,
        dest: Register,
    },
    #[delve(display = |s: &Register, d: &Register| format!("-R{s} -> R{d}"))]
    Negate {
        src: Register,
        dest: Register,
    },
    #[delve(display = |s: &Register, d: &Register| format!("~R{s} -> R{d}"))]
    BinNot {
        src: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} == R{b} -> R{x}"))]
    Eq {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} != R{b} -> R{x}"))]
    Neq {
        left: Register,
        right: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} > R{b} -> R{x}"))]
    Gt {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} < R{b} -> R{x}"))]
    Lt {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} >= R{b} -> R{x}"))]
    Gte {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} <= R{b} -> R{x}"))]
    Lte {
        left: Register,
        right: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a}..R{b} -> R{x}"))]
    Range {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} in R{b} -> R{x}"))]
    In {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} as R{b} -> R{x}"))]
    As {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} is R{b} -> R{x}"))]
    Is {
        left: Register,
        right: Register,
        dest: Register,
    },

    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} && R{b} -> R{x}"))]
    And {
        left: Register,
        right: Register,
        dest: Register,
    },
    #[delve(display = |a: &Register, b: &Register, x: &Register| format!("R{a} || R{b} -> R{x}"))]
    Or {
        left: Register,
        right: Register,
        dest: Register,
    },

    #[delve(display = |to: &JumpPos| format!("to {to}"))]
    Jump {
        to: JumpPos,
    },
    #[delve(display = |s: &Register, to: &JumpPos| format!("if !R{s}, to {to}"))]
    JumpIfFalse {
        src: Register,
        to: JumpPos,
    },

    #[delve(display = |s: &Register| format!("return R{s}"))]
    Ret {
        src: Register,
    },

    #[delve(display = |s: &Register, d: &Register| format!("R{s}? -> R{d}"))]
    WrapMaybe {
        src: Register,
        dest: Register,
    },
    #[delve(display = |d: &Register| format!("? -> R{d}"))]
    LoadNone {
        dest: Register,
    },

    #[delve(display = |d: &Register| format!("() -> R{d}"))]
    LoadEmpty {
        dest: Register,
    },

    #[delve(display = |f: &Register, d: &Register, i: &Register| format!("R{f}[R{i}] -> R{d}"))]
    Index {
        from: Register,
        dest: Register,
        index: Register,
    },
    #[delve(display = |f: &Register, d: &Register, i: &Register| format!("R{f}.R{i} -> R{d}"))]
    Member {
        from: Register,
        dest: Register,
        member: Register,
    },
    #[delve(display = |f: &Register, d: &Register, i: &Register| format!("R{f}::R{i} -> R{d}"))]
    Associated {
        from: Register,
        dest: Register,
        name: Register,
    },

    #[delve(display = || format!("yeet"))]
    YeetContext,
    #[delve(display = |to: &JumpPos| format!("-> to {to}"))]
    EnterArrowStatement {
        skip_to: JumpPos,
    },

    #[delve(display = |d: &Register| format!("$ -> R{d}"))]
    LoadBuiltins {
        dest: Register,
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
