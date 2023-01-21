use std::fmt::Display;

use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};

use delve::{EnumDisplay, EnumFields, EnumToStr, EnumVariantNames};

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

pub type Register = u8;
pub type UnoptRegister = usize;

pub type ConstID = u16;
pub type JumpPos = u16;
pub type AllocSize = u16;

macro_rules! opcodes {
    (
        <$g:ident = $d:ident> where ($($g_info:tt)*);

        $(
            $(#[$meta:meta])*
            $variant:ident $({
                $(
                    $($field:ident: $typ:ty)?
                    $(=> $reg_field:ident)?
                ),+
            })?,
        )+
    ) => {
        #[derive(
            Clone,
            Copy,
            PartialEq,
            Eq,
            Debug,
            EnumDisplay,
            EnumToStr,
            EnumFields,
            EnumVariantNames
        )]
        #[delve(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum Opcode<$g = $d> where $($g_info)* {
            $(
                $(#[$meta])*
                $variant $({
                    $(
                        $($field: $typ,)?
                        $($reg_field: $g,)?
                    )+
                })?,
            )+
        }

        pub type UnoptOpcode = Opcode<UnoptRegister>;

        impl TryFrom<UnoptOpcode> for Opcode {
            type Error = ();

            fn try_from(value: UnoptOpcode) -> Result<Self, Self::Error> {
                match value {
                    $(
                        UnoptOpcode::$variant $({
                            $(
                                $($reg_field,)?
                                $($field,)?
                            )+
                            ..
                        })? => Ok(
                            Opcode::$variant
                            $(
                                {
                                    $(
                                        $($reg_field: $reg_field.try_into().map_err(|_| ())?,)?
                                        $($field,)?
                                    )+
                                }
                            )?
                        ),
                    )+
                }
            }
        }
    };
}

opcodes! {
    <R = Register> where (R: Display + Copy);

    LoadConst {
        => dest,
        id: ConstID,
    },

    #[delve(display = |f: &R, t: &R| format!("R{f} -> R{t}"))]
    Copy { => from, => to },
    #[delve(display = |reg: &R| format!("print R{reg}"))]
    Print { => reg },
    // LoadBuiltin {},

    // Call {},
    #[delve(display = |s: &AllocSize, d: &R| format!("[...; {s}] -> R{d}"))]
    AllocArray {
        size: AllocSize,
        => dest,
    },
    #[delve(display = |s: &AllocSize, d: &R| format!("{{...; {s}}} -> R{d}"))]
    AllocDict {
        size: AllocSize,
        => dest,
    },

    #[delve(display = |e: &R, d: &R| format!("push R{d} into R{e}"))]
    PushArrayElem { => elem, => dest },
    #[delve(display = |e: &R, k: &R, d: &R| format!("insert R{k}:R{e} into R{d}"))]
    PushDictElem { => elem, => key, => dest },

    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} + R{b} -> R{x}"))]
    Add { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} - R{b} -> R{x}"))]
    Sub { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} * R{b} -> R{x}"))]
    Mult { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} / R{b} -> R{x}"))]
    Div { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} % R{b} -> R{x}"))]
    Mod { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} ^ R{b} -> R{x}"))]
    Pow { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} << R{b} -> R{x}"))]
    ShiftLeft { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} >> R{b} -> R{x}"))]
    ShiftRight { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} | R{b} -> R{x}"))]
    BinOr { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} & R{b} -> R{x}"))]
    BinAnd { => left, => right, => dest },

    #[delve(display = |a: &R, b: &R| format!("R{a} += R{b}"))]
    AddEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} -= R{b}"))]
    SubEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} *= R{b}"))]
    MultEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} /= R{b}"))]
    DivEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} %= R{b}"))]
    ModEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} ^= R{b}"))]
    PowEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} <<= R{b}"))]
    ShiftLeftEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} >>= R{b}"))]
    ShiftRightEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} &= R{b}"))]
    BinAndEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} |= R{b}"))]
    BinOrEq { => left, => right },
    #[delve(display = |a: &R, b: &R| format!("R{a} ~= R{b}"))]
    BinNotEq { => left, => right },

    #[delve(display = |s: &R, d: &R| format!("!R{s} -> R{d}"))]
    Not { => src, => dest },
    #[delve(display = |s: &R, d: &R| format!("-R{s} -> R{d}"))]
    Negate { => src, => dest },
    #[delve(display = |s: &R, d: &R| format!("~R{s} -> R{d}"))]
    BinNot { => src, => dest },

    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} == R{b} -> R{x}"))]
    Eq { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} != R{b} -> R{x}"))]
    Neq { => left, => right, => dest },

    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} > R{b} -> R{x}"))]
    Gt { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} < R{b} -> R{x}"))]
    Lt { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} >= R{b} -> R{x}"))]
    Gte { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} <= R{b} -> R{x}"))]
    Lte { => left, => right, => dest },

    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a}..R{b} -> R{x}"))]
    Range { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} in R{b} -> R{x}"))]
    In { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} as R{b} -> R{x}"))]
    As { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} is R{b} -> R{x}"))]
    Is { => left, => right, => dest },

    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} && R{b} -> R{x}"))]
    And { => left, => right, => dest },
    #[delve(display = |a: &R, b: &R, x: &R| format!("R{a} || R{b} -> R{x}"))]
    Or { => left, => right, => dest },

    #[delve(display = |to: &JumpPos| format!("to {to}"))]
    Jump {
        to: JumpPos,
    },
    #[delve(display = |s: &R, to: &JumpPos| format!("if not R{s}, to {to}"))]
    JumpIfFalse {
        => src,
        to: JumpPos,
    },

    #[delve(display = |s: &R| format!("return R{s}"))]
    Ret { => src },

    #[delve(display = |s: &R, d: &R| format!("R{s}? -> R{d}"))]
    WrapMaybe { => src, => dest },
    #[delve(display = |d: &R| format!("? -> R{d}"))]
    LoadNone { => dest },

    #[delve(display = |d: &R| format!("() -> R{d}"))]
    LoadEmpty { => dest },

    #[delve(display = |f: &R, d: &R, i: &R| format!("R{f}[R{i}] -> R{d}"))]
    Index { => from, => dest, => index },
    #[delve(display = |f: &R, d: &R, i: &R| format!("R{f}.R{i} -> R{d}"))]
    Member { => from, => dest, => member },
    #[delve(display = |f: &R, d: &R, i: &R| format!("R{f}::R{i} -> R{d}"))]
    Associated { => from, => dest, => name },

    #[delve(display = || "yeet".to_string())]
    YeetContext,
    #[delve(display = |to: &JumpPos| format!("skip to {to}"))]
    EnterArrowStatement {
        skip_to: JumpPos,
    },

    #[delve(display = |d: &R| format!("$ -> R{d}"))]
    LoadBuiltins { => dest },

    #[delve(display = |s: &R| format!("export R{s}"))]
    Export { => src },
}
