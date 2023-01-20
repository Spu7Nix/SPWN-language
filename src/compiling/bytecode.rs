use std::hash::Hash;

use ahash::AHashMap;
use colored::Colorize;
use delve::{FieldNames, VariantNames};
use regex::Regex;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};

use crate::{
    error::RainbowColorGenerator,
    gd::ids::IDClass,
    vm::opcodes::{Opcode, Register},
};

use super::compiler::CompileResult;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Id(IDClass, Option<u16>),
}

impl std::fmt::Debug for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(v) => write!(f, "{}", v.to_string()),
            Constant::Float(v) => write!(f, "{}", v.to_string()),
            Constant::Bool(v) => write!(f, "{}", v.to_string()),
            Constant::String(v) => write!(f, "\"{}\"", v),
            Constant::Id(class, n) => write!(
                f,
                "{}{}",
                if let Some(n) = n {
                    n.to_string()
                } else {
                    "".into()
                },
                class.letter()
            ),
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Constant::Int(v) => v.hash(state),
            Constant::Float(v) => v.to_bits().hash(state),
            Constant::String(v) => v.hash(state),
            Constant::Bool(v) => v.hash(state),
            Constant::Id(v, c) => {
                v.hash(state);
                c.hash(state);
            }
        }
    }
}
impl Eq for Constant {}

struct UniqueRegister<K: Key, T: Hash + Eq> {
    slotmap: SlotMap<K, T>,
    indexes: AHashMap<T, K>,
}

impl<K: Key, T: Hash + Eq + Clone> UniqueRegister<K, T> {
    pub fn new() -> Self {
        Self {
            slotmap: SlotMap::default(),
            indexes: AHashMap::new(),
        }
    }
    pub fn insert(&mut self, value: T) -> K {
        match self.indexes.get(&value) {
            Some(k) => *k,
            None => {
                let k = self.slotmap.insert(value.clone());
                self.indexes.insert(value, k);
                k
            }
        }
    }
}

#[derive(Debug, Clone)]
enum JumpTo {
    Start(Vec<usize>),
    End(Vec<usize>),
}
#[derive(Debug, Clone)]
enum ProtoOpcode {
    Raw(Opcode),

    Jump(JumpTo),
    JumpIfFalse(Register, JumpTo),
    LoadConst(Register, ConstKey),

    EnterArrowStatement(JumpTo),
}

#[derive(Debug)]
struct Block {
    path: Vec<usize>,
    content: Vec<BlockContent>,
}
#[derive(Debug)]
enum BlockContent {
    Code(Vec<ProtoOpcode>),
    Block(Block),
}
impl BlockContent {
    fn assume_code(&mut self) -> &mut Vec<ProtoOpcode> {
        match self {
            BlockContent::Code(v) => v,
            _ => {
                panic!("CODE: hej man?? what yu say men? what you say me? FAKC YOU MAN. FACK YOU.")
            }
        }
    }
    fn assume_block(&mut self) -> &mut Block {
        match self {
            BlockContent::Block(v) => v,
            _ => {
                panic!("BLOCK: hej man?? what yu say men? what you say me? FAKC YOU MAN. FACK YOU.")
            }
        }
    }
}

struct ProtoFunc {
    code: Block,
}

new_key_type! {
    pub struct ConstKey;
}
pub struct BytecodeBuilder {
    constants: UniqueRegister<ConstKey, Constant>,

    funcs: Vec<ProtoFunc>,
}

pub struct FuncBuilder<'a> {
    code_builder: &'a mut BytecodeBuilder,

    func: usize,
    pub block_path: Vec<usize>,

    used_regs: u8,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self {
            constants: UniqueRegister::new(),
            funcs: vec![],
        }
    }

    pub fn new_func<F>(&mut self, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder) -> CompileResult<()>,
    {
        let new_func = ProtoFunc {
            code: Block {
                path: vec![],
                content: vec![BlockContent::Code(vec![])],
            },
        };
        self.funcs.push(new_func);

        let mut func_builder = FuncBuilder {
            func: self.funcs.len() - 1,
            code_builder: self,
            block_path: vec![],
            used_regs: 0,
        };

        f(&mut func_builder)
    }

    pub fn build(self) -> Bytecode {
        let mut const_index_map = SecondaryMap::default();

        let consts = self
            .constants
            .slotmap
            .into_iter()
            .enumerate()
            .map(|(i, (k, c))| {
                const_index_map.insert(k, i);
                c
            })
            .collect::<Vec<_>>();

        let mut functions = vec![];

        for f in self.funcs {
            type PositionMap<'a> = AHashMap<&'a Vec<usize>, (usize, usize)>;

            let mut block_positions = AHashMap::new();

            let mut length = 0;

            fn get_block_pos<'a>(
                b: &'a Block,
                length: &mut usize,
                positions: &mut PositionMap<'a>,
            ) {
                let start = *length;
                for c in &b.content {
                    match c {
                        BlockContent::Code(code) => {
                            *length += code.len();
                        }
                        BlockContent::Block(b) => get_block_pos(b, length, positions),
                    }
                }
                let end = *length;
                positions.insert(&b.path, (start, end));
            }

            get_block_pos(&f.code, &mut length, &mut block_positions);
            // for (path, (start, end)) in &block_positions {
            //     println!("{:?} -> [{}, {})", path, start, end)
            // }

            let mut opcodes = vec![];

            fn build_block(
                b: &Block,
                opcodes: &mut Vec<Opcode>,
                positions: &PositionMap<'_>,
                const_index_map: &SecondaryMap<ConstKey, usize>,
            ) {
                let get_jump_pos = |jump: &JumpTo| -> usize {
                    match jump {
                        JumpTo::Start(path) => positions[path].0,
                        JumpTo::End(path) => positions[path].1,
                    }
                };

                for content in &b.content {
                    match content {
                        BlockContent::Code(v) => {
                            for opcode in v {
                                opcodes.push(match opcode {
                                    ProtoOpcode::Raw(o) => *o,
                                    ProtoOpcode::Jump(to) => Opcode::Jump {
                                        to: get_jump_pos(to) as u16,
                                    },
                                    ProtoOpcode::JumpIfFalse(r, to) => Opcode::JumpIfFalse {
                                        src: *r,
                                        to: get_jump_pos(to) as u16,
                                    },
                                    ProtoOpcode::LoadConst(r, k) => Opcode::LoadConst {
                                        dest: *r,
                                        id: const_index_map[*k] as u16,
                                    },
                                    ProtoOpcode::EnterArrowStatement(to) => {
                                        Opcode::EnterArrowStatement {
                                            skip_to: get_jump_pos(to) as u16,
                                        }
                                    }
                                })
                            }
                        }
                        BlockContent::Block(b) => {
                            build_block(b, opcodes, positions, const_index_map)
                        }
                    }
                }
            }

            build_block(&f.code, &mut opcodes, &block_positions, &const_index_map);

            functions.push(Function { opcodes })
        }

        Bytecode { consts, functions }
    }
}

impl<'a> FuncBuilder<'a> {
    fn current_block(&mut self) -> &mut Block {
        let mut block = &mut self.code_builder.funcs[self.func].code;
        for idx in &self.block_path {
            block = block.content[*idx].assume_block();
        }
        block
    }
    fn current_code(&mut self) -> &mut Vec<ProtoOpcode> {
        self.current_block()
            .content
            .last_mut()
            .unwrap()
            .assume_code()
    }

    pub fn next_reg(&mut self) -> Register {
        let old = self.used_regs;
        self.used_regs = self
            .used_regs
            .checked_add(1)
            .expect("sil;ly goober used too mnay regusters!!!! i🙌!");
        old
    }

    pub fn block<F>(&mut self, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder) -> CompileResult<()>,
    {
        let mut func_builder = {
            let mut new_path = self.block_path.clone();

            let block = self.current_block();
            new_path.push(block.content.len());

            block.content.push(BlockContent::Block(Block {
                path: new_path.clone(),
                content: vec![BlockContent::Code(vec![])],
            }));

            FuncBuilder {
                code_builder: self.code_builder,
                func: self.func,
                block_path: new_path,
                used_regs: self.used_regs,
            }
        };

        f(&mut func_builder)?;

        self.used_regs = func_builder.used_regs;
        self.current_block()
            .content
            .push(BlockContent::Code(vec![]));

        Ok(())
    }

    pub fn new_array<F>(&mut self, len: usize, dest: Register, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder, &mut Vec<Register>) -> CompileResult<()>,
    {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::AllocArray {
                size: len as u16,
                dest,
            }));

        let mut items = vec![];
        f(self, &mut items)?;

        for i in items {
            self.current_code()
                .push(ProtoOpcode::Raw(Opcode::PushArrayElem { elem: i, dest }))
        }

        Ok(())
    }

    pub fn new_dict<F>(&mut self, len: usize, dest: Register, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder, &mut Vec<(String, Register)>) -> CompileResult<()>,
    {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::AllocDict {
                size: len as u16,
                dest,
            }));

        let mut items = vec![];
        f(self, &mut items)?;

        for (k, r) in items {
            let key_reg = self.next_reg();
            self.load_string(k, key_reg);

            self.current_code()
                .push(ProtoOpcode::Raw(Opcode::PushDictElem {
                    elem: r,
                    key: key_reg,
                    dest,
                }))
        }

        Ok(())
    }

    pub fn load_int(&mut self, value: i64, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Int(value));
        self.current_code().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_float(&mut self, value: f64, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Float(value));
        self.current_code().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_string(&mut self, value: String, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::String(value));
        self.current_code().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_bool(&mut self, value: bool, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Bool(value));
        self.current_code().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_id(&mut self, value: Option<u16>, class: IDClass, reg: Register) {
        let k = self
            .code_builder
            .constants
            .insert(Constant::Id(class, value));
        self.current_code().push(ProtoOpcode::LoadConst(reg, k))
    }

    pub fn add(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Add { left, right, dest }))
    }
    pub fn sub(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Sub { left, right, dest }))
    }
    pub fn mult(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Mult { left, right, dest }))
    }
    pub fn div(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Div { left, right, dest }))
    }
    pub fn modulo(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Mod { left, right, dest }))
    }
    pub fn pow(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Pow { left, right, dest }))
    }
    pub fn shl(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::ShiftLeft { left, right, dest }))
    }
    pub fn shr(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::ShiftRight { left, right, dest }))
    }
    pub fn eq(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Eq { left, right, dest }))
    }
    pub fn neq(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Neq { left, right, dest }))
    }
    pub fn gt(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Gt { left, right, dest }))
    }
    pub fn gte(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Gte { left, right, dest }))
    }
    pub fn lt(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Lt { left, right, dest }))
    }
    pub fn lte(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Lte { left, right, dest }))
    }
    pub fn range(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Range { left, right, dest }))
    }
    pub fn in_op(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::In { left, right, dest }))
    }
    pub fn as_op(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::As { left, right, dest }))
    }
    pub fn is_op(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::As { left, right, dest }))
    }
    pub fn bin_or(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinOr { left, right, dest }))
    }
    pub fn bin_and(&mut self, left: Register, right: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinAnd { left, right, dest }))
    }

    pub fn add_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::AddEq { left, right }))
    }
    pub fn sub_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::SubEq { left, right }))
    }
    pub fn mult_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::MultEq { left, right }))
    }
    pub fn div_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::DivEq { left, right }))
    }
    pub fn modulo_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::ModEq { left, right }))
    }
    pub fn pow_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::PowEq { left, right }))
    }
    pub fn shl_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::ShiftLeftEq { left, right }))
    }
    pub fn shr_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::ShiftRightEq { left, right }))
    }
    pub fn bin_and_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinAndEq { left, right }))
    }
    pub fn bin_or_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinOrEq { left, right }))
    }
    pub fn bin_not_eq(&mut self, left: Register, right: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinNotEq { left, right }))
    }

    pub fn unary_not(&mut self, src: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Not { src, dest }))
    }
    pub fn unary_negate(&mut self, src: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Negate { src, dest }))
    }
    pub fn unary_bin_not(&mut self, src: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::BinNot { src, dest }))
    }

    pub fn copy(&mut self, from: Register, to: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Copy { from, to }))
    }

    pub fn print(&mut self, reg: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Print { reg }))
    }

    pub fn repeat_block(&mut self) {
        let path = self.block_path.clone();
        self.current_code()
            .push(ProtoOpcode::Jump(JumpTo::Start(path)))
    }

    pub fn exit_block(&mut self) {
        let path = self.block_path.clone();
        self.current_code()
            .push(ProtoOpcode::Jump(JumpTo::End(path)))
    }
    pub fn exit_other_block(&mut self, path: Vec<usize>) {
        self.current_code()
            .push(ProtoOpcode::Jump(JumpTo::End(path)))
    }
    pub fn enter_arrow(&mut self) {
        let path = self.block_path.clone();
        self.current_code()
            .push(ProtoOpcode::EnterArrowStatement(JumpTo::End(path)))
    }
    // pub fn exit_block_absolute(&mut self, to: usize) {
    //     self.current_code().push(ProtoOpcode::Jump(to))
    // }

    pub fn exit_if_false(&mut self, reg: Register) {
        let path = self.block_path.clone();
        self.current_code()
            .push(ProtoOpcode::JumpIfFalse(reg, JumpTo::End(path)))
    }
    pub fn sex(&self) {
        println!("func: {:?}, path: {:?}", self.func, self.block_path);
    }

    pub fn load_none(&mut self, reg: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::LoadNone { dest: reg }))
    }
    pub fn wrap_maybe(&mut self, src: Register, dest: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::WrapMaybe { src, dest }))
    }
    pub fn load_empty(&mut self, reg: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::LoadEmpty { dest: reg }))
    }

    pub fn index(&mut self, from: Register, dest: Register, index: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Index { from, dest, index }))
    }
    pub fn member(&mut self, from: Register, dest: Register, member: String) {
        let next_reg = self.next_reg();
        self.load_string(member, next_reg);
        self.current_code().push(ProtoOpcode::Raw(Opcode::Member {
            from,
            dest,
            member: next_reg,
        }))
    }
    pub fn associated(&mut self, from: Register, dest: Register, associated: String) {
        let next_reg = self.next_reg();
        self.load_string(associated, next_reg);
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Associated {
                from,
                dest,
                name: next_reg,
            }))
    }

    pub fn load_builtins(&mut self, to: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::LoadBuiltins { dest: to }))
    }

    pub fn ret(&mut self, src: Register) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::Ret { src }))
    }

    pub fn yeet_context(&mut self) {
        self.current_code()
            .push(ProtoOpcode::Raw(Opcode::YeetContext))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    opcodes: Vec<Opcode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bytecode {
    consts: Vec<Constant>,

    functions: Vec<Function>,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let longest_opcode: usize = Opcode::VARIANT_NAMES
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(2);

        writeln!(f, "{}: {:?}\n", "Constants".red(), self.consts)?;

        let mut colors = RainbowColorGenerator::new(150.0, 0.4, 0.9, 60.0);

        let mut lines = vec![];
        let mut formatted_opcodes = vec![];
        let mut formatted_field_names = vec![];
        let mut longest_formatted = 0;
        let mut longest_field_name = 0;

        let col_reg = Regex::new(r"(R\d+)").unwrap();

        let ansi_regex = Regex::new(r#"(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]"#).unwrap();
        let clear_ansi = |s: &str| ansi_regex.replace_all(s, "").to_string();

        for (i, func) in self.functions.iter().enumerate() {
            writeln!(
                f,
                "{}",
                format!("======== Function {} ========", i).yellow()
            )?;

            for (i, opcode) in func.opcodes.iter().enumerate() {
                lines.push(format!(
                    "{}  {:>pad$}",
                    i.to_string().blue().bold(),
                    <&Opcode as Into<&'static str>>::into(&opcode),
                    pad = (longest_opcode - (i.to_string().len() - 1))
                ));

                let formatted = match opcode {
                    Opcode::LoadConst { dest, id } => {
                        format!(
                            "{} -> R{dest}",
                            format!("{:?}", &self.consts[*id as usize])
                                .bright_purple()
                                .bold()
                        )
                    }
                    _ => {
                        format!("{}", opcode)
                    }
                };

                let formatted = col_reg
                    .replace_all(&formatted, "$1".red().bold().to_string())
                    .to_string();
                let f_len = clear_ansi(&formatted).len();

                if f_len > longest_formatted {
                    longest_formatted = f_len;
                }
                formatted_opcodes.push(formatted);

                let field_names = opcode.field_names().unwrap_or(&[]).join(", ");

                if field_names.len() > longest_field_name {
                    longest_field_name = field_names.len();
                }
                formatted_field_names.push(field_names);
            }

            for (i, line) in lines.iter_mut().enumerate() {
                let c = colors.next();

                let fmto = &formatted_opcodes[i];
                let fmto_len = clear_ansi(&fmto).len();

                let fmtv = &formatted_field_names[i];

                let bytes = bincode::serialize(&func.opcodes[i]).unwrap();

                line.push_str(&format!(
                    "  {} {:pad$}｜ {}{:pad2$}  ｜  {}",
                    fmto.bright_white(),
                    "",
                    fmtv.bright_yellow(),
                    "",
                    bytes
                        .iter()
                        .map(|n| format!("{:0>2X}", n))
                        .collect::<Vec<String>>()
                        .join(" ")
                        .truecolor(c.0, c.1, c.2),
                    pad = longest_formatted - fmto_len,
                    pad2 = longest_field_name - fmtv.len(),
                ));
            }

            writeln!(f, "{}", lines.join("\n"))?
        }

        Ok(())
    }
}
