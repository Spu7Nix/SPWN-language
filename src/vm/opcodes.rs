use std::fmt::Display;

use delve::{EnumDisplay, EnumFields, EnumToStr, EnumVariantNames};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Serialize};

use crate::gd::ids::IDClass;

struct OpcodeVisitor;

impl Serialize for Opcode<Register> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // SAFETY:
        // opcodes will always be u32 or less
        serializer.serialize_u32(unsafe { std::mem::transmute::<_, u32>(*self) })
    }
}

impl<'de> Deserialize<'de> for Opcode<Register> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_u32(OpcodeVisitor)
    }
}

impl<'de> Visitor<'de> for OpcodeVisitor {
    type Value = Opcode<Register>;

    fn expecting(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        panic!("expected u32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        // SAFETY:
        // who is manually writing bytecode
        Ok(unsafe { std::mem::transmute::<_, Opcode<Register>>(value) })
    }
}

pub type Register = u8;
pub type UnoptRegister = usize;

pub type ConstID = u16;
pub type JumpPos = u16;
pub type AllocSize = u16;
pub type FunctionID = u16;
pub type ImportID = u16;

macro_rules! opcodes {
    (
        <$g:ident> where ($($g_info:tt)*);

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
        pub enum Opcode<$g> where $($g_info)* {
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

        impl TryFrom<UnoptOpcode> for Opcode<Register> {
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
    <R> where (R: Display + Copy );

    LoadConst {
        => dest,
        id: ConstID,
    },

    #[delve(display = |f: &R, t: &R| format!("R{f} -> R{t}"))]
    Copy { => from, => to },
    #[delve(display = |reg: &R| format!("print R{reg}"))]
    Print { => reg },


    #[delve(display = |b: &R, a: &R, d: &R| format!("R{b}(args R{a}) -> R{d}"))]
    Call { => base, => args, => dest },

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
    #[delve(display = |s: &AllocSize, d: &R| format!("obj {{...; {s}}} -> R{d}"))]
    AllocObject {
        size: AllocSize,
        => dest,
    },
    #[delve(display = |s: &AllocSize, d: &R| format!("trigger {{...; {s}}} -> R{d}"))]
    AllocTrigger {
        size: AllocSize,
        => dest,
    },

    #[delve(display = |e: &R, d: &R| format!("push R{e} into R{d}"))]
    PushArrayElem { => elem, => dest },
    #[delve(display = |e: &R, k: &R, d: &R| format!("insert R{k}:R{e} into R{d}"))]
    PushDictElem { => elem, => key, => dest },
    #[delve(display = |e: &R, i: &u8, d: &R| format!("insert i:R{e} into R{d}"))]
    PushObjectElem { => elem, obj_id: u8, => dest },

    #[delve(display = |i: &FunctionID, d: &R| format!("{i}: (...) {{...}} -> R{d}"))]
    CreateMacro {
        id: FunctionID,
        => dest,
    },
    #[delve(display = |n: &R, d: &R| format!("insert arg R{n} into R{d}"))]
    PushMacroArg { => name, => dest },
    #[delve(display = |s: &R, d: &R| format!("set default to R{s} for R{d}"))]
    SetMacroArgDefault { => src, => dest },
    #[delve(display = |s: &R, d: &R| format!("set pattern to R{s} for R{d}"))]
    SetMacroArgPattern { => src, => dest },

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

    #[delve(display = |c: &IDClass, d: &R| format!("?{} -> R{d}", c.letter()))]
    LoadArbitraryId { class: IDClass, => dest },

    #[delve(display = |src: &R| format!("change to R{src}"))]
    PushContextGroup { => src },
    #[delve(display = || "pop".to_string())]
    PopGroupStack,

    #[delve(display = |s: &R, d: &R| format!("!{{R{s}}} -> R{d}"))]
    MakeTriggerFunc { => src, => dest },

    #[delve(display = |b: &R, d: &R, i: &R| format!("R{b}[R{i}] ~> R{d}"))]
    Index { => base, => dest, => index },
    #[delve(display = |f: &R, d: &R, i: &R| format!("R{f}.R{i} ~> R{d}"))]
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
    #[delve(display = |s: &ImportID, d: &R| format!("import id {s} -> R{d}"))]
    Import { src: ImportID => dest },
}
