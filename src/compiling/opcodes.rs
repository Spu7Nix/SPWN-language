use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::bytecode::{OptRegister, Register, UnoptRegister};
use crate::gd::ids::IDClass;
use crate::gd::object_keys::ObjectKey;
use crate::new_id_wrapper;
use crate::parsing::operators::operators::Operator;

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

new_id_wrapper! {
    ConstID: u16;
    OpcodePos: u16;
    ImportID: u16;
    FuncID: u16;
    AttributeID: u16;
    CallExprID: u16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, delve::EnumDisplay, Serialize, Deserialize)]
pub enum RuntimeStringFlag {
    #[delve(display = "byte flag")]
    ByteString,
    #[delve(display = "unindent flag")]
    Unindent,
    #[delve(display = "b64 flag")]
    Base64,
}

opcodes! {
    #[delve(display = |id, to| format!("load {id} -> {to}"))]
    LoadConst { id: ConstID, [to] },

    #[delve(display = |from, to| format!("{from} deep -> {to}"))]
    CopyDeep { [from], [to] },
    #[delve(display = |from, to| format!("{from} shallow -> {to}"))]
    CopyShallow { [from], [to] },
    #[delve(display = |from, to| format!("{from} ref -> {to}"))]
    CopyRef { [from], [to] },

    #[delve(display = |from, to| format!("write {from} ~> {to}"))]
    Write { [from], [to] },
    #[delve(display = |from, to| format!("write {from} deep ~> {to}"))]
    WriteDeep { [from], [to] },

    #[delve(display = |from, to| format!("assign {from} ref -> {to}"))]
    AssignRef { [from], [to] },
    #[delve(display = |from, to| format!("assign {from} deep ~> {to}"))]
    AssignDeep { [from], [to] },

    #[delve(display = |a, b, to| format!("{a} + {b} -> {to}"))]
    Plus { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} - {b} -> {to}"))]
    Minus { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} * {b} -> {to}"))]
    Mult { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} / {b} -> {to}"))]
    Div { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} % {b} -> {to}"))]
    Mod { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} ^ {b} -> {to}"))]
    Pow { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} == {b} -> {to}"))]
    Eq { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} != {b} -> {to}"))]
    Neq { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} > {b} -> {to}"))]
    Gt { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} >= {b} -> {to}"))]
    Gte { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} < {b} -> {to}"))]
    Lt { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} <= {b} -> {to}"))]
    Lte { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} | {b} -> {to}"))]
    BinOr { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} & {b} -> {to}"))]
    BinAnd { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a}..{b} -> {to}"))]
    Range { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} in {b} -> {to}"))]
    In { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} << {b} -> {to}"))]
    ShiftLeft { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} >> {b} -> {to}"))]
    ShiftRight { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("{a} as {b} -> {to}"))]
    As { [a], [b], [to] },

    #[delve(display = |a, b| format!("{a} += {b}"))]
    PlusEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} -= {b}"))]
    MinusEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} *= {b}"))]
    MultEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} /= {b}"))]
    DivEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} ^= {b}"))]
    PowEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} %= {b}"))]
    ModEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} &= {b}"))]
    BinAndEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} |= {b}"))]
    BinOrEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} <<= {b}"))]
    ShiftLeftEq { [a], [b] },
    #[delve(display = |a, b| format!("{a} >>= {b}"))]
    ShiftRightEq { [a], [b] },

    #[delve(display = |v, to| format!("!{v} -> {to}"))]
    Not { [v], [to] },
    #[delve(display = |v, to| format!("-{v} -> {to}"))]
    Negate { [v], [to] },


    #[delve(display = |a, b, to| format!("pure {a} == {b} -> {to}"))]
    PureEq { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("pure {a} != {b} -> {to}"))]
    PureNeq { [a], [b], [to] },
    #[delve(display = |a, b, to| format!("pure {a} >= {b} -> {to}"))]
    PureGte { [a], [b], [to] },

    #[delve(display = |to| format!("to {to}"))]
    Jump { to: OpcodePos },
    // #[delve(display = |to: &FuncID| format!("jump to {to:?}"))]
    // FuncJump { to: FuncID },
    #[delve(display = |check, to| format!("if not {check}, to {to}"))]
    JumpIfFalse { [check], to: OpcodePos },
    #[delve(display = |check, to| format!("if {check}, to {to}"))]
    JumpIfTrue { [check], to: OpcodePos },
    #[delve(display = |check, to| format!("if {check} == ?, to {to}"))]
    UnwrapOrJump { [check], to: OpcodePos },


    #[delve(display = |src, dest| format!("{src}.iter() -> {dest}"))]
    IntoIterator { [src], [dest] },
    #[delve(display = |src, dest| format!("{src}.next() -> {dest}"))]
    IterNext { [src], [dest] },

    #[delve(display = |dest, len| format!("[...; {len}] -> {dest}"))]
    AllocArray { [dest], len: u16 },
    #[delve(display = |elem, dest| format!("push {elem} into {dest}"))]
    PushArrayElem { [elem], [dest] },

    #[delve(display = |dest, cap| format!("{{...; {cap}}} -> {dest}"))]
    AllocDict { [dest], capacity: u16 },
    #[delve(display = |elem, dest, key| format!("insert {key}:{elem} into {dest}"))]
    InsertDictElem { [elem], [dest], [key] },
    #[delve(display = |elem, dest, key| format!("insert priv {key}:{elem} into {dest}"))]
    InsertPrivDictElem { [elem], [dest], [key] },

    #[delve(display = |base, items, dest| format!("@{base}::{{{items}}} -> {dest}"))]
    MakeInstance { [base], [items], [dest] },

    #[delve(display = |dest, cap| format!("obj {{...; {cap}}} -> {dest}"))]
    AllocObject { [dest], capacity: u16 },
    #[delve(display = |dest, cap| format!("trigger {{...; {cap}}} -> {dest}"))]
    AllocTrigger { [dest], capacity: u16 },
    #[delve(display = |e: &R, k: &ObjectKey, d: &R| format!("insert {}:{e} into R{d}", <&ObjectKey as Into<& str>>::into(k)))]
    PushObjectElemKey { [elem], obj_key: ObjectKey, [dest] },
    #[delve(display = |e: &R, k: &u8, d: &R| format!("insert {k}:{e} into {d}"))]
    PushObjectElemUnchecked { [elem], obj_key: u8, [dest] },


    #[delve(display = |skip| format!("skip to {skip}"))]
    EnterArrowStatement { skip: OpcodePos },
    #[delve(display = "yeet")]
    YeetContext,


    #[delve(display = |to| format!("() -> {to}"))]
    LoadEmpty { [to] },
    #[delve(display = |to| format!("? -> {to}"))]
    LoadNone { [to] },
    #[delve(display = |to| format!("$ -> {to}"))]
    LoadBuiltins { [to] },
    #[delve(display = |to| format!("ε -> {to}"))]
    LoadEpsilon { [to] },


    #[delve(display = |c: &IDClass, d: &R| format!("?{c} -> {d}"))]
    LoadArbitraryID { class: IDClass, [dest] },


    #[delve(display = |flag: &RuntimeStringFlag, reg| format!("apply {flag} to {reg}"))]
    ApplyStringFlag { flag: RuntimeStringFlag, [reg] },


    #[delve(display = |from, to| format!("{from}? -> {to}"))]
    WrapMaybe { [from], [to] },

    #[delve(display = |src, mr: &bool| format!("{} {src}", if *mr { "export" } else { "return" }))]
    Return { [src], module_ret: bool },

    #[delve(display = |reg, s| format!("dbg{} {reg}", if *show_ptr { "*" } else { "" }))]
    Dbg { [reg], show_ptr: bool },

    #[delve(display = |reg| format!("throw {reg}"))]
    Throw { [reg] },

    #[delve(display = |id, dest| format!("import {id} -> {dest}"))]
    Import { id: ImportID, [dest] },

    #[delve(display = |from, dest| format!("@string({from}) -> {dest}"))]
    ToString { [from], [dest] },

    #[delve(display = |b, d, i| format!("{b}[{i}] -> {d}"))]
    Index { [base], [dest], [index] },

    #[delve(display = |f, d, i| format!("{f}.{i} -> {d}"))]
    MemberImmut { [from], [dest], [member] },
    #[delve(display = |f, d, i| format!("(mut) {f}.{i} -> {d}"))]
    MemberMut { [from], [dest], [member] },

    #[delve(display = |f, d, i| format!("{f}::{i} -> {d}"))]
    Associated { [from], [dest], [member] },
    #[delve(display = |f, d, i| format!("{f}.@{i} -> {d}"))]
    TypeMember { [from], [dest], [member] },

    #[delve(display = |s, d| format!("{s}.type -> {d}"))]
    TypeOf { [src], [dest] },

    #[delve(display = |s, d| format!("{s}.len() -> {d}"))]
    Len { [src], [dest] },

    #[delve(display = |s, d| format!("{s} arg amount -> {d}"))]
    ArgAmount { [src], [dest] },


    #[delve(display = |r, v| format!("if not {r}, throw mismatch"))]
    MismatchThrowIfFalse { [check_reg], [value_reg] },

    #[delve(display = |reg, to| format!("push try, catch -> {reg}, to {to}"))]
    PushTryCatch { [reg], to: OpcodePos },

    #[delve(display = "pop try catch")]
    PopTryCatch,

    #[delve(display = |i, r| format!("{i}: (...) {{...}} -> {r}"))]
    CreateMacro { func: FuncID, [dest]},
    #[delve(display = |to, f, arg| format!("{f} -> {to} default arg {arg}"))]
    PushMacroDefault { [to], [from], arg: u8},
    #[delve(display = |r| format!("mark arg 1 of {r} as `self`"))]
    MarkMacroMethod { [reg] },

    #[delve(display = |base, id| format!("{base}({id})"))]
    Call { [base], call: CallExprID },

    #[delve(display = |b, d| format!("impl @{b} {{{d}}}"))]
    Impl { [base], [dict] },

    #[delve(display = |args, dest| format!("run builtin with {args} args -> {dest}"))]
    RunBuiltin { args: u8, [dest] },

    #[delve(display = |s, d| format!("!{{{s}}} -> {d}"))]
    MakeTriggerFunc { [src], [dest] },
    #[delve(display = |f| format!("{f}!"))]
    CallTriggerFunc { [func] },

    #[delve(display = |r| format!("set context group from {r}"))]
    SetContextGroup { [reg] },

    #[delve(display = |from, op| format!("add {op:?} overload from {from}"))]
    AddOperatorOverload { [from], op: Operator },
    #[delve(display = "<internal>")]
    IncMismatchIdCount,
}
