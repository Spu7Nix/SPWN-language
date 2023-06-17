use std::cell::RefCell;
use std::rc::Rc;

use super::bytecode::{ConstID, Constant, UnoptRegister};
use super::opcodes::{Opcode, UnoptOpcode};
use crate::parsing::ast::{Spannable, Spanned};
use crate::sources::CodeSpan;
use crate::util::UniqueRegister;

#[derive(Debug)]
enum JumpTo {
    Start(Box<[usize]>),
    End(Box<[usize]>),
}
#[derive(Debug)]
enum ProtoOpcode {
    Raw(UnoptOpcode),
    Jump(JumpTo),
    JumpIfFalse(UnoptRegister, JumpTo),
}

#[derive(Debug)]
enum BlockContent {
    Opcode(Spanned<ProtoOpcode>),
    Block(Block),
}

impl BlockContent {
    pub fn assume_block(&self) -> &Block {
        match self {
            BlockContent::Block(b) => b,
            _ => panic!("badddly"),
        }
    }

    pub fn assume_block_mut(&mut self) -> &mut Block {
        match self {
            BlockContent::Block(b) => b,
            _ => panic!("badddly"),
        }
    }

    pub fn assume_code(&self) -> &Spanned<ProtoOpcode> {
        match self {
            BlockContent::Opcode(b) => b,
            _ => panic!("badddly"),
        }
    }

    pub fn assume_code_mut(&mut self) -> &mut Spanned<ProtoOpcode> {
        match self {
            BlockContent::Opcode(b) => b,
            _ => panic!("badddly"),
        }
    }
}

#[derive(Default, Debug)]
pub struct Block {
    content: Vec<BlockContent>,
}

#[derive(Debug)]
struct ProtoFunc {
    code: Block,
}

#[derive(Debug)]
pub struct BytecodeBuilder {
    consts: UniqueRegister<Constant>,
    functions: Vec<ProtoFunc>,
    regs_used: usize,
}
pub struct FuncBuilder<'a> {
    bytecode_builder: &'a mut BytecodeBuilder,
    func: usize,
    pub block_path: &'a [usize],
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self {
            consts: UniqueRegister::new(),
            functions: vec![],
            regs_used: 0,
        }
    }

    pub fn new_func<F: FnOnce(&mut FuncBuilder)>(&mut self, f: F) {
        self.functions.push(ProtoFunc {
            code: Default::default(),
        });
        let func = self.functions.len() - 1;
        f(&mut FuncBuilder {
            block_path: &[],
            func: 0,
            bytecode_builder: self,
        });
    }
}

impl FuncBuilder<'_> {
    fn current_block(&mut self) -> &mut Block {
        let mut block = &mut self.bytecode_builder.functions[self.func].code;
        for idx in self.block_path {
            block = block.content[*idx].assume_block_mut();
        }
        block
    }

    fn push_opcode(&mut self, opcode: ProtoOpcode, span: CodeSpan) {
        self.current_block()
            .content
            .push(BlockContent::Opcode(opcode.spanned(span)))
    }

    pub fn load_int(&mut self, v: i64, reg: UnoptRegister, span: CodeSpan) {
        let id = ConstID(self.bytecode_builder.consts.insert(Constant::Int(v)) as u16);
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn load_float(&mut self, v: f64, reg: UnoptRegister, span: CodeSpan) {
        let id = ConstID(self.bytecode_builder.consts.insert(Constant::Float(v)) as u16);
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn load_string(&mut self, v: Box<str>, reg: UnoptRegister, span: CodeSpan) {
        let id = ConstID(self.bytecode_builder.consts.insert(Constant::String(v)) as u16);
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn load_bool(&mut self, v: bool, reg: UnoptRegister, span: CodeSpan) {
        let id = ConstID(self.bytecode_builder.consts.insert(Constant::Bool(v)) as u16);
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn new_block<F: FnOnce(&mut FuncBuilder)>(&mut self, f: F) {
        let block = self.current_block();
        block.content.push(BlockContent::Block(Default::default()));
        let idx = block.content.len() - 1;

        let mut v = self.block_path.to_vec();
        v.push(idx);

        f(&mut FuncBuilder {
            block_path: &v,
            bytecode_builder: self.bytecode_builder,
            func: self.func,
        });
    }

    // pub fn jump_to_start(&mut self, block: Option<Rc<RefCell<Block>>>, span: CodeSpan) {
    //     self.push_opcode(
    //         ProtoOpcode::Jump(JumpTo::Start(block.unwrap_or(Rc::clone(&self.block)))),
    //         span,
    //     );
    // }

    // pub fn jump_to_end(&mut self, block: Option<Rc<RefCell<Block>>>, span: CodeSpan) {
    //     self.push_opcode(
    //         ProtoOpcode::Jump(JumpTo::End(block.unwrap_or(Rc::clone(&self.block)))),
    //         span,
    //     );
    // }
}
