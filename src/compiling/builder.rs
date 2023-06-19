use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use semver::Version;
use slab::Slab;

use super::bytecode::{Bytecode, ConstID, Constant, Register, UnoptRegister};
use super::compiler::CompileResult;
use super::opcodes::{Opcode, UnoptOpcode};
use crate::compiling::bytecode::OpcodePos;
use crate::compiling::opcodes::OptOpcode;
use crate::new_id_wrapper;
use crate::sources::{CodeSpan, Spannable, Spanned, SpwnSource};
use crate::util::{ImmutVec, SlabMap, UniqueRegister};

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
}

#[derive(Debug)]
pub struct ProtoBytecode {
    consts: UniqueRegister<Constant>,
    functions: Vec<ProtoFunc>,
    blocks: SlabMap<BlockID, Block>,
    regs_used: usize,
}

impl ProtoBytecode {
    pub fn new() -> Self {
        Self {
            consts: UniqueRegister::new(),
            functions: vec![],
            blocks: SlabMap::new(),
            regs_used: 0,
        }
    }

    pub fn new_func<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
    ) -> CompileResult<()> {
        let f_block = self.blocks.insert(Default::default());
        self.functions.push(ProtoFunc { code: f_block });
        f(&mut CodeBuilder {
            bytecode_builder: self,
            block: f_block,
        })
    }

    pub fn build(mut self, src: &Rc<SpwnSource>) -> Result<Bytecode, ()> {
        type BlockPos = (u16, u16);

        let mut constants = vec![unsafe { std::mem::zeroed() }; self.consts.map.len()];
        for (k, v) in self.consts.map.drain() {
            constants[v] = k
        }

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

        for f in &self.functions {
            get_block_pos(&self, f.code, &mut code_len, &mut block_positions);
            build_block(&self, f.code, &mut opcodes, &block_positions)?;
        }

        Ok(Bytecode {
            source_hash: md5::compute(src.read().unwrap()).into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            constants: constants.into(),
            opcodes: opcodes.into(),
            regs_used: self.regs_used.try_into().unwrap(),
        })
        // Bytecode {
        //     source_hash: hash.into(),
        //     version: env!("CARGO_PKG_VERSION").into(),
        //     constants: constants.into_boxed_slice(),
        //     opcodes: todo!(),
        // }
    }
}

pub struct CodeBuilder<'a> {
    bytecode_builder: &'a mut ProtoBytecode,
    pub block: BlockID,
}

impl CodeBuilder<'_> {
    fn current_block(&mut self) -> &mut Block {
        &mut self.bytecode_builder.blocks[self.block]
    }

    pub fn next_reg(&mut self) -> UnoptRegister {
        let r = Register(self.bytecode_builder.regs_used);
        self.bytecode_builder.regs_used += 1;
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
    ) {
        self.push_opcode(
            ProtoOpcode::Raw(Opcode::InsertDictElem { elem, dest, key }),
            span,
        )
    }

    pub fn copy_deep(&mut self, from: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::CopyDeep { from, to }), span)
    }

    pub fn copy_mem(&mut self, from: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::CopyMem { from, to }), span)
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
}