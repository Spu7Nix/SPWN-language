use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use semver::Version;
use slab::Slab;

use super::bytecode::{Bytecode, Constant, Register, UnoptRegister};
use super::compiler::{CompileResult, Compiler};
use super::opcodes::{Opcode, UnoptOpcode};
use crate::compiling::bytecode::Function;
use crate::compiling::compiler::CustomTypeID;
use crate::compiling::opcodes::OptOpcode;
use crate::interpreting::value::ValueType;
use crate::new_id_wrapper;
use crate::sources::{CodeSpan, Spannable, Spanned, SpwnSource};
use crate::util::{ImmutStr, ImmutVec, SlabMap, UniqueRegister};

new_id_wrapper! {
    BlockID: u16;
}

#[derive(Debug, Clone, Copy)]
enum JumpTo {
    Start(BlockID),
    End(BlockID),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpType {
    Start,
    End,
    StartIfFalse(UnoptRegister),
    EndIfFalse(UnoptRegister),
    UnwrapOrStart(UnoptRegister),
    UnwrapOrEnd(UnoptRegister),
}

#[derive(Debug, Clone, Copy)]
enum ProtoOpcode {
    Raw(UnoptOpcode),
    Jump(JumpTo),
    JumpIfFalse(UnoptRegister, JumpTo),
    UnwrapOrJump(UnoptRegister, JumpTo),
    EnterArrowStatement(JumpTo),
    MatchCatch(JumpTo),
}

#[derive(Debug)]
enum BlockContent {
    Opcode(Spanned<ProtoOpcode>),
    Block(BlockID),
}

#[derive(Default, Debug)]
pub struct Block {
    content: Vec<BlockContent>,
}

#[derive(Debug)]
struct ProtoFunc {
    code: BlockID,
    regs_used: usize,
    span: CodeSpan,
}

#[derive(Debug)]
pub struct ProtoBytecode {
    consts: UniqueRegister<Constant>,
    functions: Vec<ProtoFunc>,
    blocks: SlabMap<BlockID, Block>,

    import_paths: UniqueRegister<SpwnSource>,
    // custom_types:
}

impl ProtoBytecode {
    pub fn new() -> Self {
        Self {
            consts: UniqueRegister::new(),
            functions: vec![],
            blocks: SlabMap::new(),
            import_paths: UniqueRegister::new(),
        }
    }

    pub fn new_func<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
        span: CodeSpan,
    ) -> CompileResult<()> {
        let f_block = self.blocks.insert(Default::default());
        self.functions.push(ProtoFunc {
            code: f_block,
            regs_used: 0,
            span,
        });
        f(&mut CodeBuilder {
            func: self.functions.len() - 1,
            bytecode_builder: self,
            block: f_block,
        })
    }

    pub fn build(mut self, src: &Rc<SpwnSource>, compiler: &Compiler) -> Result<Bytecode, ()> {
        type BlockPos = (u16, u16);

        let mut constants = vec![unsafe { std::mem::zeroed() }; self.consts.len()];
        for (v, k) in self.consts.drain() {
            constants[v] = k
        }

        let mut funcs = vec![];

        for (func_id, func) in self.functions.iter().enumerate() {
            let mut block_positions = AHashMap::new();
            let mut code_len = 0;

            let mut opcodes = vec![];

            fn get_block_pos(
                code: &ProtoBytecode,
                block: BlockID,
                length: &mut u16,
                positions: &mut AHashMap<BlockID, BlockPos>,
            ) {
                let start = *length;
                for c in &code.blocks[block].content {
                    match c {
                        BlockContent::Opcode(_) => {
                            *length += 1;
                        },
                        BlockContent::Block(b) => get_block_pos(code, *b, length, positions),
                    }
                }
                let end = *length;
                positions.insert(block, (start, end));
            }

            fn build_block(
                code: &ProtoBytecode,
                block: BlockID,
                opcodes: &mut Vec<Spanned<OptOpcode>>,
                positions: &AHashMap<BlockID, BlockPos>,
            ) -> Result<(), ()> {
                let get_jump_pos = |jump: JumpTo| -> u16 {
                    match jump {
                        JumpTo::Start(path) => positions[&path].0,
                        JumpTo::End(path) => positions[&path].1,
                    }
                };

                for content in &code.blocks[block].content {
                    match content {
                        BlockContent::Opcode(o) => {
                            let opcode = o.map(|v| match v {
                                ProtoOpcode::Raw(o) => o,
                                ProtoOpcode::Jump(to) => UnoptOpcode::Jump {
                                    to: get_jump_pos(to).into(),
                                },
                                ProtoOpcode::JumpIfFalse(r, to) => UnoptOpcode::JumpIfFalse {
                                    check: r,
                                    to: get_jump_pos(to).into(),
                                },
                                ProtoOpcode::UnwrapOrJump(r, to) => UnoptOpcode::UnwrapOrJump {
                                    check: r,
                                    to: get_jump_pos(to).into(),
                                },
                                ProtoOpcode::EnterArrowStatement(skip) => {
                                    UnoptOpcode::EnterArrowStatement {
                                        skip: get_jump_pos(skip).into(),
                                    }
                                },
                                ProtoOpcode::MatchCatch(jump) => UnoptOpcode::MatchCatch {
                                    jump: get_jump_pos(jump).into(),
                                },
                            });
                            opcodes.push(Spanned {
                                value: opcode.value.try_into().unwrap(),
                                span: opcode.span,
                            })
                        },
                        BlockContent::Block(b) => build_block(code, *b, opcodes, positions)?,
                    }
                }
                Ok(())
            }

            get_block_pos(&self, func.code, &mut code_len, &mut block_positions);
            build_block(&self, func.code, &mut opcodes, &block_positions)?;

            funcs.push(Function {
                regs_used: func.regs_used.try_into().unwrap(),
                opcodes: opcodes.into(),
                span: func.span,
            })
        }

        let mut import_paths = vec![unsafe { std::mem::zeroed() }; self.import_paths.len()];
        for (v, k) in self.import_paths.drain() {
            import_paths[v] = k
        }

        let src_hash = compiler.src_hash();

        Ok(Bytecode {
            source_hash: md5::compute(src.read().unwrap()).into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            constants: constants.into(),
            functions: funcs.into(),
            custom_types: compiler
                .custom_type_defs
                .iter()
                .map(|(id, v)| {
                    (
                        CustomTypeID {
                            local: id,
                            source_hash: src_hash,
                        },
                        v.map(|def| compiler.resolve(&def.name).spanned(def.def_span)),
                    )
                })
                .collect(),
            export_names: match &compiler.global_return {
                Some(v) => v.iter().map(|s| compiler.resolve(&s.value)).collect(),
                None => Box::new([]),
            },
            import_paths: import_paths.into(),
        })
    }
}

pub struct CodeBuilder<'a> {
    bytecode_builder: &'a mut ProtoBytecode,
    pub func: usize,
    pub block: BlockID,
}

impl CodeBuilder<'_> {
    fn current_block(&mut self) -> &mut Block {
        &mut self.bytecode_builder.blocks[self.block]
    }

    pub fn next_reg(&mut self) -> UnoptRegister {
        let r = Register(self.bytecode_builder.functions[self.func].regs_used);
        self.bytecode_builder.functions[self.func].regs_used += 1;
        r
    }

    pub fn new_block<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
    ) -> CompileResult<()> {
        let f_block = self.bytecode_builder.blocks.insert(Default::default());

        self.current_block()
            .content
            .push(BlockContent::Block(f_block));

        f(&mut CodeBuilder {
            block: f_block,
            func: self.func,
            bytecode_builder: self.bytecode_builder,
        })
    }

    fn push_opcode(&mut self, opcode: ProtoOpcode, span: CodeSpan) {
        self.current_block()
            .content
            .push(BlockContent::Opcode(opcode.spanned(span)))
    }

    pub fn push_raw_opcode(&mut self, opcode: UnoptOpcode, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(opcode), span)
    }

    pub fn load_const<T: Into<Constant>>(&mut self, v: T, reg: UnoptRegister, span: CodeSpan) {
        let id = self.bytecode_builder.consts.insert(v.into()).into();
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn jump(&mut self, block: Option<BlockID>, jump_type: JumpType, span: CodeSpan) {
        let block = block.unwrap_or(self.block);
        let opcode = match jump_type {
            JumpType::Start => ProtoOpcode::Jump(JumpTo::Start(block)),
            JumpType::End => ProtoOpcode::Jump(JumpTo::End(block)),
            JumpType::StartIfFalse(reg) => ProtoOpcode::JumpIfFalse(reg, JumpTo::Start(block)),
            JumpType::EndIfFalse(reg) => ProtoOpcode::JumpIfFalse(reg, JumpTo::End(block)),
            JumpType::UnwrapOrStart(reg) => ProtoOpcode::UnwrapOrJump(reg, JumpTo::Start(block)),
            JumpType::UnwrapOrEnd(reg) => ProtoOpcode::UnwrapOrJump(reg, JumpTo::End(block)),
        };

        self.push_opcode(opcode, span);
    }

    pub fn alloc_array(&mut self, dest: UnoptRegister, len: u16, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::AllocArray { dest, len }), span)
    }

    pub fn push_array_elem(&mut self, elem: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::PushArrayElem { elem, dest }), span)
    }

    pub fn alloc_dict(&mut self, dest: UnoptRegister, capacity: u16, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::AllocDict { dest, capacity }), span)
    }

    pub fn insert_dict_elem(
        &mut self,
        elem: UnoptRegister,
        dest: UnoptRegister,
        key: UnoptRegister,
        span: CodeSpan,
        private: bool,
    ) {
        if private {
            self.push_opcode(
                ProtoOpcode::Raw(Opcode::InsertPrivDictElem { elem, dest, key }),
                span,
            )
        } else {
            self.push_opcode(
                ProtoOpcode::Raw(Opcode::InsertDictElem { elem, dest, key }),
                span,
            )
        }
    }

    pub fn copy_deep(&mut self, from: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::CopyDeep { from, to }), span)
    }

    pub fn iter_next(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::IterNext { src, dest }), span)
    }

    pub fn enter_arrow(&mut self, span: CodeSpan) {
        self.push_opcode(
            ProtoOpcode::EnterArrowStatement(JumpTo::End(self.block)),
            span,
        )
    }

    pub fn yeet_context(&mut self, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::YeetContext), span)
    }

    pub fn load_empty(&mut self, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadEmpty { to }), span)
    }

    pub fn ret(&mut self, src: UnoptRegister, module_ret: bool, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Return { src, module_ret }), span)
    }

    pub fn dbg(&mut self, reg: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Dbg { reg }), span)
    }

    pub fn throw(&mut self, reg: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Throw { reg }), span)
    }

    // pub fn create_type(&mut self, name: String, private: bool, span: CodeSpan) -> CustomTypeKey {
    //     self.code_builder
    //         .custom_types
    //         .insert((name.spanned(span), private))
    // }

    pub fn import(&mut self, dest: UnoptRegister, src: SpwnSource, span: CodeSpan) {
        let id = self.bytecode_builder.import_paths.insert(src).into();
        self.push_opcode(ProtoOpcode::Raw(Opcode::Import { id, dest }), span);
    }

    pub fn member(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        member: Spanned<ImmutVec<char>>,
        span: CodeSpan,
    ) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::Member {
                from,
                dest,
                member: next_reg,
            }),
            span,
        )
    }

    pub fn index(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        index: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::Index {
                base: from,
                dest,
                index,
            }),
            span,
        )
    }

    pub fn type_of(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::TypeOf { src, dest }), span)
    }

    pub fn assert(&mut self, reg: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::Assert { reg }), span)
    }

    pub fn assert_matches(&mut self, reg: UnoptRegister, pat: UnoptRegister, span: CodeSpan) {
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::AssertMatches { reg, pat }),
            span,
        )
    }

    pub fn index_set_mem(&mut self, index: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::IndexSetMem { index }), span)
    }

    pub fn member_set_mem(&mut self, member: Spanned<ImmutVec<char>>, span: CodeSpan) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::MemberSetMem { member: next_reg }),
            span,
        )
    }

    pub fn change_mem(&mut self, from: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::ChangeMem { from }), span)
    }

    pub fn write_mem(&mut self, from: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::WriteMem { from }), span)
    }

    pub fn match_catch(&mut self, block: Option<BlockID>, end: bool, span: CodeSpan) {
        let block = block.unwrap_or(self.block);
        let opcode = if end {
            ProtoOpcode::MatchCatch(JumpTo::End(block))
        } else {
            ProtoOpcode::MatchCatch(JumpTo::Start(block))
        };

        self.push_opcode(opcode, span);
    }
}
