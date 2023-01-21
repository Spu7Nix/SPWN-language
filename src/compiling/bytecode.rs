use std::{fmt::Display, hash::Hash};

use ahash::AHashMap;
use colored::Colorize;
use delve::VariantNames;
use regex::Regex;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};

use crate::{
    error::RainbowColorGenerator,
    gd::ids::IDClass,
    parsing::ast::{Spannable, Spanned},
    sources::{CodeSpan, SpwnSource},
    util::Digest,
    vm::opcodes::{Opcode, Register, UnoptOpcode, UnoptRegister},
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
            Constant::Int(v) => write!(f, "{}", v),
            Constant::Float(v) => write!(f, "{}", v),
            Constant::Bool(v) => write!(f, "{}", v),
            Constant::String(v) => write!(f, "{:?}", v),
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
    Raw(UnoptOpcode),

    Jump(JumpTo),
    JumpIfFalse(UnoptRegister, JumpTo),
    LoadConst(UnoptRegister, ConstKey),

    EnterArrowStatement(JumpTo),
}

#[derive(Debug)]
struct Block {
    path: Vec<usize>,
    content: Vec<BlockContent>,
}
#[derive(Debug)]
enum BlockContent {
    Code(Vec<Spanned<ProtoOpcode>>),
    Block(Block),
}

impl BlockContent {
    fn assume_code(&mut self) -> &mut Vec<Spanned<ProtoOpcode>> {
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

    used_regs: usize,
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

    pub fn build(self) -> Bytecode<UnoptRegister> {
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
        let mut opcode_span_map = AHashMap::new();

        for (f_n, f) in self.funcs.iter().enumerate() {
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
                func: usize,
                opcodes: &mut Vec<UnoptOpcode>,
                opcode_span_map: &mut AHashMap<(usize, usize), CodeSpan>,
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
                                opcodes.push(match &opcode.value {
                                    ProtoOpcode::Raw(o) => *o,
                                    ProtoOpcode::Jump(to) => UnoptOpcode::Jump {
                                        to: get_jump_pos(to) as u16,
                                    },
                                    ProtoOpcode::JumpIfFalse(r, to) => UnoptOpcode::JumpIfFalse {
                                        src: *r,
                                        to: get_jump_pos(to) as u16,
                                    },
                                    ProtoOpcode::LoadConst(r, k) => UnoptOpcode::LoadConst {
                                        dest: *r,
                                        id: const_index_map[*k] as u16,
                                    },
                                    ProtoOpcode::EnterArrowStatement(to) => {
                                        UnoptOpcode::EnterArrowStatement {
                                            skip_to: get_jump_pos(to) as u16,
                                        }
                                    }
                                });

                                if opcode.span != CodeSpan::invalid() {
                                    opcode_span_map.insert((func, opcodes.len() - 1), opcode.span);
                                }
                            }
                        }
                        BlockContent::Block(b) => build_block(
                            b,
                            func,
                            opcodes,
                            opcode_span_map,
                            positions,
                            const_index_map,
                        ),
                    }
                }
            }

            build_block(
                &f.code,
                f_n,
                &mut opcodes,
                &mut opcode_span_map,
                &block_positions,
                &const_index_map,
            );

            functions.push(Function { opcodes })
        }

        Bytecode {
            source_hash: None,
            consts,
            functions,
            opcode_span_map,
        }
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
    fn current_code(&mut self) -> &mut Vec<Spanned<ProtoOpcode>> {
        self.current_block()
            .content
            .last_mut()
            .unwrap()
            .assume_code()
    }

    fn push_opcode(&mut self, opcode: ProtoOpcode) {
        self.current_code()
            .push(opcode.spanned(CodeSpan::invalid()))
    }
    fn push_opcode_spanned(&mut self, opcode: ProtoOpcode, span: CodeSpan) {
        self.current_code().push(opcode.spanned(span))
    }

    pub fn next_reg(&mut self) -> UnoptRegister {
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

    pub fn new_array<F>(&mut self, len: u16, dest: UnoptRegister, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder, &mut Vec<UnoptRegister>) -> CompileResult<()>,
    {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::AllocArray {
            size: len,
            dest,
        }));

        let mut items = vec![];
        f(self, &mut items)?;

        for i in items {
            self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::PushArrayElem {
                elem: i,
                dest,
            }))
        }

        Ok(())
    }

    pub fn new_dict<F>(&mut self, len: u16, dest: UnoptRegister, f: F) -> CompileResult<()>
    where
        F: FnOnce(&mut FuncBuilder, &mut Vec<(String, UnoptRegister)>) -> CompileResult<()>,
    {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::AllocDict { size: len, dest }));

        let mut items = vec![];
        f(self, &mut items)?;

        for (k, r) in items {
            let key_reg = self.next_reg();
            self.load_string(k, key_reg, CodeSpan::invalid());

            self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::PushDictElem {
                elem: r,
                key: key_reg,
                dest,
            }))
        }

        Ok(())
    }

    pub fn load_int(&mut self, value: i64, reg: UnoptRegister, span: CodeSpan) {
        let k = self.code_builder.constants.insert(Constant::Int(value));
        self.push_opcode_spanned(ProtoOpcode::LoadConst(reg, k), span)
    }
    pub fn load_float(&mut self, value: f64, reg: UnoptRegister, span: CodeSpan) {
        let k = self.code_builder.constants.insert(Constant::Float(value));
        self.push_opcode_spanned(ProtoOpcode::LoadConst(reg, k), span)
    }
    pub fn load_string(&mut self, value: String, reg: UnoptRegister, span: CodeSpan) {
        let k = self.code_builder.constants.insert(Constant::String(value));
        self.push_opcode_spanned(ProtoOpcode::LoadConst(reg, k), span)
    }
    pub fn load_bool(&mut self, value: bool, reg: UnoptRegister, span: CodeSpan) {
        let k = self.code_builder.constants.insert(Constant::Bool(value));
        self.push_opcode_spanned(ProtoOpcode::LoadConst(reg, k), span)
    }
    pub fn load_id(
        &mut self,
        value: Option<u16>,
        class: IDClass,
        reg: UnoptRegister,
        span: CodeSpan,
    ) {
        let k = self
            .code_builder
            .constants
            .insert(Constant::Id(class, value));
        self.push_opcode_spanned(ProtoOpcode::LoadConst(reg, k), span)
    }

    pub fn add(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Add { left, right, dest }),
            span,
        )
    }
    pub fn sub(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Sub { left, right, dest }),
            span,
        )
    }
    pub fn mult(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Mult { left, right, dest }),
            span,
        )
    }
    pub fn div(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Div { left, right, dest }),
            span,
        )
    }
    pub fn modulo(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Mod { left, right, dest }),
            span,
        )
    }
    pub fn pow(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Pow { left, right, dest }),
            span,
        )
    }
    pub fn shl(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::ShiftLeft { left, right, dest }),
            span,
        )
    }
    pub fn shr(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::ShiftRight { left, right, dest }),
            span,
        )
    }
    pub fn eq(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Eq { left, right, dest }),
            span,
        )
    }
    pub fn neq(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Neq { left, right, dest }),
            span,
        )
    }
    pub fn gt(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Gt { left, right, dest }),
            span,
        )
    }
    pub fn gte(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Gte { left, right, dest }),
            span,
        )
    }
    pub fn lt(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Lt { left, right, dest }),
            span,
        )
    }
    pub fn lte(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Lte { left, right, dest }),
            span,
        )
    }
    pub fn range(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::Range { left, right, dest }),
            span,
        )
    }
    pub fn in_op(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::In { left, right, dest }),
            span,
        )
    }
    pub fn as_op(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::As { left, right, dest }),
            span,
        )
    }
    pub fn is_op(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::As { left, right, dest }),
            span,
        )
    }
    pub fn bin_or(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::BinOr { left, right, dest }),
            span,
        )
    }
    pub fn bin_and(
        &mut self,
        left: UnoptRegister,
        right: UnoptRegister,
        dest: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::BinAnd { left, right, dest }),
            span,
        )
    }

    pub fn add_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::AddEq { left, right }), span)
    }
    pub fn sub_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::SubEq { left, right }), span)
    }
    pub fn mult_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::MultEq { left, right }), span)
    }
    pub fn div_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::DivEq { left, right }), span)
    }
    pub fn modulo_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::ModEq { left, right }), span)
    }
    pub fn pow_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::PowEq { left, right }), span)
    }
    pub fn shl_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::ShiftLeftEq { left, right }),
            span,
        )
    }
    pub fn shr_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::ShiftRightEq { left, right }),
            span,
        )
    }
    pub fn bin_and_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::BinAndEq { left, right }),
            span,
        )
    }
    pub fn bin_or_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::BinOrEq { left, right }), span)
    }
    pub fn bin_not_eq(&mut self, left: UnoptRegister, right: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::BinNotEq { left, right }),
            span,
        )
    }

    pub fn unary_not(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::Not { src, dest }), span)
    }
    pub fn unary_negate(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::Negate { src, dest }), span)
    }
    pub fn unary_bin_not(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::BinNot { src, dest }), span)
    }

    pub fn copy(&mut self, from: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::Copy { from, to }), span)
    }

    pub fn print(&mut self, reg: UnoptRegister) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Print { reg }))
    }

    pub fn repeat_block(&mut self) {
        let path = self.block_path.clone();
        self.push_opcode(ProtoOpcode::Jump(JumpTo::Start(path)))
    }

    pub fn exit_block(&mut self) {
        let path = self.block_path.clone();
        self.push_opcode(ProtoOpcode::Jump(JumpTo::End(path)))
    }
    pub fn exit_other_block(&mut self, path: Vec<UnoptRegister>) {
        self.push_opcode(ProtoOpcode::Jump(JumpTo::End(path)))
    }
    pub fn enter_arrow(&mut self) {
        let path = self.block_path.clone();
        self.push_opcode(ProtoOpcode::EnterArrowStatement(JumpTo::End(path)))
    }
    // pub fn exit_block_absolute(&mut self, to: UnoptRegister) {
    //     self.current_code().push(ProtoOpcode::Jump(to))
    // }

    pub fn exit_if_false(&mut self, reg: UnoptRegister) {
        let path = self.block_path.clone();
        self.push_opcode(ProtoOpcode::JumpIfFalse(reg, JumpTo::End(path)))
    }

    pub fn load_none(&mut self, reg: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::LoadNone { dest: reg }), span)
    }
    pub fn wrap_maybe(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::WrapMaybe { src, dest }), span)
    }
    pub fn load_empty(&mut self, reg: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(ProtoOpcode::Raw(UnoptOpcode::LoadEmpty { dest: reg }), span)
    }

    pub fn index(&mut self, from: UnoptRegister, dest: UnoptRegister, index: UnoptRegister) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Index { from, dest, index }))
    }
    pub fn member(&mut self, from: UnoptRegister, dest: UnoptRegister, member: String) {
        let next_reg = self.next_reg();
        self.load_string(member, next_reg, CodeSpan::invalid());
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Member {
            from,
            dest,
            member: next_reg,
        }))
    }
    pub fn associated(&mut self, from: UnoptRegister, dest: UnoptRegister, associated: String) {
        let next_reg = self.next_reg();
        self.load_string(associated, next_reg, CodeSpan::invalid());
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Associated {
            from,
            dest,
            name: next_reg,
        }))
    }

    pub fn load_builtins(&mut self, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode_spanned(
            ProtoOpcode::Raw(UnoptOpcode::LoadBuiltins { dest: to }),
            span,
        )
    }

    pub fn ret(&mut self, src: UnoptRegister) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Ret { src }))
    }

    pub fn export(&mut self, src: UnoptRegister) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Export { src }))
    }

    pub fn yeet_context(&mut self) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::YeetContext))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "Opcode<R>: Serialize, for<'de2> Opcode<R>: Deserialize<'de2>")]
pub struct Function<R>
where
    R: Display + Copy,
{
    pub opcodes: Vec<Opcode<R>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "Opcode<R>: Serialize, for<'de2> Opcode<R>: Deserialize<'de2>")]
pub struct Bytecode<R>
where
    R: Display + Copy,
{
    pub source_hash: Option<Digest>,
    pub consts: Vec<Constant>,

    pub functions: Vec<Function<R>>,
    pub opcode_span_map: AHashMap<(usize, usize), CodeSpan>,
}

impl<R> Bytecode<R>
where
    R: Display + Copy,
    Opcode<R>: Serialize,
{
    pub fn debug_str(&self, src: &SpwnSource) {
        let code = src.read().unwrap();

        let longest_opcode: usize = Opcode::<Register>::VARIANT_NAMES
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(2);

        println!("{}: {:?}\n", "Constants".bright_cyan().bold(), self.consts);

        let mut colors = RainbowColorGenerator::new(150.0, 0.4, 0.9, 60.0);

        let col_reg = Regex::new(r"(R\d+)").unwrap();
        let arrow_reg = Regex::new(r"([+\-*/<>]+)").unwrap();

        let ansi_regex = Regex::new(r#"(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]"#).unwrap();
        let clear_ansi = |s: &str| ansi_regex.replace_all(s, "").to_string();

        for (func_i, func) in self.functions.iter().enumerate() {
            let mut lines = vec![];
            let mut formatted_opcodes = vec![];
            let mut longest_formatted = 0;
            let mut formatted_spans = vec![];
            let mut longest_span = 0;

            let max_num_width = func.opcodes.len().to_string().len();

            for (opcode_i, opcode) in func.opcodes.iter().enumerate() {
                lines.push(format!(
                    "{:<pad$}  {:>pad2$}",
                    opcode_i.to_string().bright_blue().bold(),
                    <&Opcode<R> as Into<&'static str>>::into(opcode),
                    pad = max_num_width,
                    pad2 = longest_opcode
                ));

                let formatted = match opcode {
                    Opcode::LoadConst { dest, id } => {
                        format!(
                            "{} -> R{dest}",
                            format!("{:?}", &self.consts[*id as usize])
                                .bright_cyan()
                                .bold()
                        )
                    }
                    _ => {
                        format!("{}", opcode)
                    }
                };

                let formatted = col_reg
                    .replace_all(&formatted, "$1".bright_red().bold().to_string())
                    .to_string();
                let f_len = clear_ansi(&formatted).len();

                let opcode_str = format!(
                    "{:?}",
                    match self.opcode_span_map.get(&(func_i, opcode_i)) {
                        Some(span) => &code[span.start..span.end],
                        None => "",
                    }
                );
                let opcode_str = opcode_str[1..opcode_str.len() - 1].to_string();
                let o_len = opcode_str.len();

                if f_len > longest_formatted {
                    longest_formatted = f_len;
                }
                formatted_opcodes.push(formatted);

                if o_len > longest_span {
                    longest_span = o_len;
                }
                formatted_spans.push(opcode_str);
            }

            for (i, line) in lines.iter_mut().enumerate() {
                let c = colors.next();

                let fmto = &formatted_opcodes[i];
                let fmto_len = clear_ansi(fmto).len();

                let ostr = &formatted_spans[i];
                let ostr = if ostr != "" {
                    ostr.to_string()
                } else {
                    format!("{}{}", " ".repeat((longest_span - 1) / 2), "-".red().bold())
                };
                let o_len = clear_ansi(&ostr).len();

                let bytes = bincode::serialize(&func.opcodes[i]).unwrap();

                line.push_str(&format!(
                    "  {} {:pad$} {line}  {}{:pad2$}  {line}  {} ",
                    fmto.bright_white(),
                    "",
                    ostr,
                    "",
                    bytes
                        .iter()
                        .map(|n| format!("{:0>2X}", n))
                        .collect::<Vec<String>>()
                        .join(" ")
                        .truecolor(c.0, c.1, c.2),
                    pad = longest_formatted - fmto_len,
                    pad2 = longest_span - o_len,
                    line = "│".bright_yellow(),
                ));
            }

            println!(
                "{}",
                format!(
                    "╭────┤ Function {} ├{}┬{}┬{}╮",
                    func_i,
                    "─".repeat(longest_formatted + max_num_width + 10),
                    "─".repeat(longest_span + 4),
                    // bytecode will never be more than 15 characters (2 spaces padding on either side, 2 hex chars + 1 space * 4)
                    "─".repeat(15),
                )
                .bright_yellow()
            );

            for line in &lines {
                println!("{0} {1} {0}", "│".bright_yellow(), line)
            }

            println!(
                "{}",
                format!(
                    "╰──────────────────{}┴{}┴{}╯",
                    "─".repeat(longest_formatted + max_num_width + 10),
                    "─".repeat(longest_span + 4),
                    // bytecode will never be more than 15 characters (2 spaces padding on either side, 2 hex chars + 1 space * 4)
                    "─".repeat(15),
                )
                .bright_yellow()
            );
        }
    }
}
