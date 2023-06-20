use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::bytecode::{ConstID, FuncID, OpcodePos, OptRegister, Register, UnoptRegister};

pub type UnoptOpcode = Opcode<UnoptRegister>;
pub type OptOpcode = Opcode<OptRegister>;

macro_rules! opcodes {
    (
        $(
            $(#[$delve:meta])?
            $name:ident $({
                $(
                    $($field:ident: $typ:ty)?
                    $([$reg_field:ident])?
                ),+ $(,)?
            })?
        ),* $(,)?
    ) => {

        #[derive(Debug, Clone, Copy, delve::EnumVariantNames, delve::EnumDisplay, delve::EnumToStr, delve::EnumFields, Serialize, Deserialize)]
        #[delve(rename_variants = "screamingsnakecase")]
        pub enum Opcode<R: Copy + std::fmt::Display> {
            $(
                $(#[$delve])?
                $name $({
                    $(
                        $($field: $typ)?
                        $($reg_field: R)?
                        ,
                    )+
                })?,
            )*
        }

        impl TryFrom<Opcode<UnoptRegister>> for Opcode<OptRegister> {
            type Error = ();

            fn try_from(value: Opcode<UnoptRegister>) -> Result<Self, Self::Error> {
                match value {
                    $(
                        Opcode::$name $({$(
                            $($field)?
                            $($reg_field)?
                            ,
                        )+})? => Ok(Opcode::$name $({$(
                            $($reg_field: $reg_field.try_into().map_err(|_| ())?,)?
                            $($field,)?
                        )+})?),
                    )*
                }
            }
        }

    };
}

opcodes! {
    #[delve(display = |id: &ConstID, to: &R| format!("load {id} -> {to}"))]
    LoadConst { id: ConstID, [to] },
    #[delve(display = |from: &R, to: &R| format!("{from} -> {to}"))]
    CopyDeep { [from], [to] },
    #[delve(display = |from: &R, to: &R| format!("{from} ~> {to}"))]
    CopyMem { [from], [to] },

    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} + {b} -> {to}"))]
    Plus { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} - {b} -> {to}"))]
    Minus { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} * {b} -> {to}"))]
    Mult { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} / {b} -> {to}"))]
    Div { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} % {b} -> {to}"))]
    Mod { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} ^ {b} -> {to}"))]
    Pow { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} == {b} -> {to}"))]
    Eq { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} != {b} -> {to}"))]
    Neq { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} > {b} -> {to}"))]
    Gt { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} >= {b} -> {to}"))]
    Gte { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} < {b} -> {to}"))]
    Lt { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} <= {b} -> {to}"))]
    Lte { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} | {b} -> {to}"))]
    BinOr { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} & {b} -> {to}"))]
    BinAnd { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a}..{b} -> {to}"))]
    Range { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} in {b} -> {to}"))]
    In { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} << {b} -> {to}"))]
    ShiftLeft { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} >> {b} -> {to}"))]
    ShiftRight { [a], [b], [to] },
    #[delve(display = |a: &R, b: &R, to: &R| format!("{a} as {b} -> {to}"))]
    As { [a], [b], [to] },

    #[delve(display = |a: &R, b: &R| format!("{a} += {b}"))]
    PlusEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} -= {b}"))]
    MinusEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} *= {b}"))]
    MultEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} /= {b}"))]
    DivEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} ^= {b}"))]
    PowEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} %= {b}"))]
    ModEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} &= {b}"))]
    BinAndEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} |= {b}"))]
    BinOrEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} <<= {b}"))]
    ShiftLeftEq { [a], [b] },
    #[delve(display = |a: &R, b: &R| format!("{a} >>= {b}"))]
    ShiftRightEq { [a], [b] },

    #[delve(display = |v: &R, to: &R| format!("!{v} -> {to}"))]
    Not { [v], [to] },
    #[delve(display = |v: &R, to: &R| format!("-{v} -> {to}"))]
    Negate { [v], [to] },

    #[delve(display = |to: &OpcodePos| format!("to {to}"))]
    Jump { to: OpcodePos },
    // #[delve(display = |to: &FuncID| format!("jump to {to:?}"))]
    // FuncJump { to: FuncID },
    #[delve(display = |check: &R, to: &OpcodePos| format!("if not {check}, to {to}"))]
    JumpIfFalse { [check], to: OpcodePos },
    #[delve(display = |check: &R, to: &OpcodePos| format!("if {check} == ?, to {to}"))]
    UnwrapOrJump { [check], to: OpcodePos },


    #[delve(display = |src: &R, dest: &R| format!("{src}.iter() -> {dest}"))]
    WrapIterator { [src], [dest] },
    #[delve(display = |src: &R, dest: &R| format!("{src}.next() -> {dest}"))]
    IterNext { [src], [dest] },

    #[delve(display = "ret")]
    Ret,

    #[delve(display = |dest: &R, len: &u16| format!("[...; {len}] -> {dest}"))]
    AllocArray { [dest], len: u16 },
    #[delve(display = |elem: &R, dest: &R| format!("push {elem} into {dest}"))]
    PushArrayElem { [elem], [dest] },

    #[delve(display = |dest: &R, cap: &u16| format!("{{...; {cap}}} -> {dest}"))]
    AllocDict { [dest], capacity: u16 },
    #[delve(display = |elem: &R, dest: &R, key: &R| format!("insert {key}:{elem} into {dest}"))]
    InsertDictElem { [elem], [dest], [key] },


    #[delve(display = |skip: &OpcodePos| format!("skip to {skip}"))]
    EnterArrowStatement { skip: OpcodePos },
    #[delve(display = || "yeet")]
    YeetContext,


    #[delve(display = |to: &R| format!("() -> {to}"))]
    LoadEmpty { [to] },

    #[delve(display = |src: &R, mr: &bool| format!("{} R{src}", if *mr { "export" } else { "return" }))]
    Return { [src], module_ret: bool },

    #[delve(display = |reg: &R| format!("dbg {reg}"))]
    Dbg { [reg] },

    #[delve(display = |reg: &R| format!("throw {reg}"))]
    Throw { [reg] },
}
