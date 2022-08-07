use crate::compilation::compiler::URegister;
use crate::sources::{CodeSpan, SpwnSource};
use std::ops::Index;

use super::compiler::Constant;
macro_rules! wrappers {
    ($($n:ident($t:ty))*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $n(pub $t);

            impl<T> Index<$n> for URegister<T> {
                type Output = T;

                fn index(&self, index: $n) -> &Self::Output {
                    &self.reg[index.0 as usize]
                }
            }
            impl From<$t> for $n {
                fn from(n: $t) -> Self {
                    $n(n)
                }
            }
        )*
    };
}
wrappers! {
    InstrNum(u16)

    VarID(u16)
    ConstID(u16)
    KeysID(u16)
    MemberID(u16)
    MacroBuildID(u16)
}

pub struct BytecodeFunc {
    pub instructions: Vec<(Instruction, CodeSpan)>,
    pub arg_ids: Vec<VarID>,
    pub capture_ids: Vec<VarID>,
    pub inner_ids: Vec<VarID>,
}

pub struct Code {
    pub source: SpwnSource,

    pub const_register: URegister<Constant>,
    pub keys_register: URegister<Vec<String>>,
    pub member_register: URegister<String>,
    #[allow(clippy::type_complexity)]
    pub macro_build_register: URegister<(usize, Vec<(String, bool, bool)>)>,

    pub var_count: usize,

    pub funcs: Vec<BytecodeFunc>,
}

impl Code {
    pub fn new(source: SpwnSource) -> Self {
        Self {
            source,
            const_register: URegister::new(),
            keys_register: URegister::new(),
            macro_build_register: URegister::new(),
            member_register: URegister::new(),
            var_count: 0,
            funcs: vec![],
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        for (i, f) in self.funcs.iter().enumerate() {
            writeln!(
                &mut debug_str,
                "================== Func {} ================== arg_ids: {:?}, capture_ids: {:?}, inner_ids: {:?}",
                i,
                f.arg_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                f.capture_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                f.inner_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
            )
            .unwrap();
            for (i, (instr, _)) in f.instructions.iter().enumerate() {
                writeln!(
                    &mut debug_str,
                    "{}\t{:?}    {}",
                    i,
                    instr,
                    ansi_term::Color::Green.bold().paint(match instr {
                        Instruction::LoadConst(c) => format!("{:?}", self.const_register[*c]),
                        Instruction::BuildDict(k) => format!("{:?}", self.keys_register[*k]),
                        Instruction::Member(k) => format!("{}", self.member_register[*k]),
                        Instruction::BuildMacro(b) =>
                            format!("{:?}", self.macro_build_register[*b]),
                        _ => "".into(),
                    })
                )
                .unwrap();
            }
        }

        let re = regex::Regex::new(r"ConstID\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow
                    .bold()
                    .paint("const $1")
                    .to_string(),
            )
            .into();
        let re = regex::Regex::new(r"VarID\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow.bold().paint("var $1").to_string(),
            )
            .into();
        let re = regex::Regex::new(r"InstrNum\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow.bold().paint("$1").to_string(),
            )
            .into();
        let re = regex::Regex::new(r"KeysID\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow
                    .bold()
                    .paint("dict keys $1")
                    .to_string(),
            )
            .into();
        let re = regex::Regex::new(r"MacroBuildID\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow
                    .bold()
                    .paint("macro build $1")
                    .to_string(),
            )
            .into();

        let re = regex::Regex::new(r"MemberID\(([^)]*)\)").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow
                    .bold()
                    .paint("member $1")
                    .to_string(),
            )
            .into();

        println!("{}", debug_str);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InstrPos {
    pub func: usize,
    pub idx: usize,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    LoadConst(ConstID),

    Plus,
    Minus,
    Mult,
    Div,
    Modulo,
    Pow,

    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,

    Negate,
    Not,

    LoadVar(VarID),
    SetVar(VarID),
    CreateVar(VarID),

    BuildArray(InstrNum),
    BuildDict(KeysID),

    Jump(InstrNum),
    JumpIfFalse(InstrNum),

    PopTop,
    PushEmpty,

    WrapMaybe,
    PushNone,

    TriggerFuncCall,
    PushTriggerFn,

    Print,

    ToIter,
    IterNext(InstrNum),

    Impl(KeysID),

    PushAnyPattern,
    BuildMacro(MacroBuildID),
    Call(InstrNum),
    Return,

    Index,
    Member(MemberID),
    TypeOf,

    YeetContext,
    EnterArrowStatement(InstrNum),
    EnterTriggerFunction(InstrNum),

    /// makes gd object data structure from last n elements on the stack
    BuildObject(InstrNum),
    BuildTrigger(InstrNum),
    AddObject,

    BuildInstance(KeysID),
}

impl Instruction {
    pub fn modify_num(&mut self, n: u16) {
        match self {
            Self::BuildArray(num)
            | Self::Jump(num)
            | Self::JumpIfFalse(num)
            | Self::EnterArrowStatement(num)
            | Self::EnterTriggerFunction(num)
            | Self::IterNext(num)
            | Self::BuildObject(num)
            | Self::BuildTrigger(num) => num.0 = n,
            _ => panic!("can't modify number of variant that doesnt hold nubere rf  v 🤓🤓🤓"),
        }
    }
}
