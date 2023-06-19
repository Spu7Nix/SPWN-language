use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use derive_more::{Deref, From};
use slab::Slab;

use super::bytecode::{Bytecode, ConstID, Constant, Register, UnoptRegister};
use super::opcodes::{Opcode, UnoptOpcode};
use crate::compiling::bytecode::OpcodePos;
use crate::sources::{CodeSpan, Spannable, Spanned};
use crate::util::{ImmutVec, UniqueRegister};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Deref)]
pub struct BlockID(usize);

#[derive(Debug, Clone, Copy)]
enum JumpTo {
    Start(BlockID),
    End(BlockID),
}
#[derive(Debug, Clone, Copy)]
enum ProtoOpcode {
    Raw(UnoptOpcode),
    Jump(JumpTo),
    JumpIfFalse(UnoptRegister, JumpTo),
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
    blocks: Slab<Block>,
    regs_used: usize,
}

impl ProtoBytecode {
    pub fn new() -> Self {
        Self {
            consts: UniqueRegister::new(),
            functions: vec![],
            blocks: Slab::new(),
            regs_used: 0,
        }
    }

    pub fn new_func<F: FnOnce(&mut CodeBuilder)>(&mut self, f: F) {
        let f_block = BlockID(self.blocks.insert(Default::default()));
        self.functions.push(ProtoFunc { code: f_block });
        f(&mut CodeBuilder {
            bytecode_builder: self,
            block: f_block,
        });
    }

    pub fn build(mut self) -> Bytecode<UnoptRegister> {
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
            for c in &code.blocks[block.0].content {
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
            opcodes: &mut Vec<Spanned<UnoptOpcode>>,
            positions: &AHashMap<BlockID, BlockPos>,
        ) {
            let get_jump_pos = |jump: JumpTo| -> u16 {
                match jump {
                    JumpTo::Start(path) => positions[&path].0,
                    JumpTo::End(path) => positions[&path].1,
                }
            };

            for content in &code.blocks[*block].content {
                match content {
                    BlockContent::Opcode(o) => opcodes.push(o.map(|v| match v {
                        ProtoOpcode::Raw(o) => o,
                        ProtoOpcode::Jump(to) => UnoptOpcode::Jump {
                            to: OpcodePos(get_jump_pos(to)),
                        },
                        ProtoOpcode::JumpIfFalse(r, to) => UnoptOpcode::JumpIfFalse {
                            check: r,
                            to: OpcodePos(get_jump_pos(to)),
                        },
                    })),
                    BlockContent::Block(b) => build_block(code, *b, opcodes, positions),
                }
            }
        }

        for f in &self.functions {
            get_block_pos(&self, f.code, &mut code_len, &mut block_positions);
            build_block(&self, f.code, &mut opcodes, &block_positions)
        }

        for o in opcodes {
            println!("{:?}", o);
        }
        todo!()
        // let hash = md5::compute(src.read().unwrap());

        // Bytecode {
        //     source_hash: hash.into(),
        //     version: env!("CARGO_PKG_VERSION"),
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
        &mut self.bytecode_builder.blocks[self.block.0]
    }

    pub fn next_reg(&mut self) -> UnoptRegister {
        let r = Register(self.bytecode_builder.regs_used);
        self.bytecode_builder.regs_used += 1;
        r
    }

    fn push_opcode(&mut self, opcode: ProtoOpcode, span: CodeSpan) {
        self.current_block()
            .content
            .push(BlockContent::Opcode(opcode.spanned(span)))
    }

    pub fn load_const<T: Into<Constant>>(&mut self, v: T, reg: UnoptRegister, span: CodeSpan) {
        let id = ConstID(self.bytecode_builder.consts.insert(v.into()) as u16);
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn jump_to_start(&mut self, block: Option<BlockID>, span: CodeSpan) {
        self.push_opcode(
            ProtoOpcode::Jump(JumpTo::Start(block.unwrap_or(self.block))),
            span,
        );
    }

    pub fn jump_to_end(&mut self, block: Option<BlockID>, span: CodeSpan) {
        self.push_opcode(
            ProtoOpcode::Jump(JumpTo::End(block.unwrap_or(self.block))),
            span,
        );
    }

    pub fn alloc_array(&mut self, reg: UnoptRegister, len: u16, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::AllocArray { reg, len }), span)
    }

    pub fn push_array_elem(&mut self, elem: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::PushArrayElem { elem, dest }), span)
    }

    pub fn alloc_dict(&mut self, reg: UnoptRegister, capacity: u16, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::AllocDict { reg, capacity }), span)
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

    pub fn new_block<F: FnOnce(&mut CodeBuilder)>(&mut self, f: F) {
        let f_block = BlockID(self.bytecode_builder.blocks.insert(Default::default()));

        self.current_block()
            .content
            .push(BlockContent::Block(f_block));

        f(&mut CodeBuilder {
            block: f_block,
            bytecode_builder: self.bytecode_builder,
        })
    }
}
