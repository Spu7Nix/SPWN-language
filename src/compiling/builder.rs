use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use itertools::Itertools;
use semver::Version;
use slab::Slab;

use super::bytecode::{Bytecode, CallExpr, Constant, Register, UnoptRegister};
use super::compiler::{CompileResult, Compiler};
use super::opcodes::{FuncID, Opcode, UnoptOpcode};
use crate::compiling::bytecode::Function;
use crate::compiling::compiler::CustomTypeID;
use crate::compiling::opcodes::OptOpcode;
use crate::interpreting::value::ValueType;
use crate::new_id_wrapper;
use crate::parsing::attributes::Attributes;
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
    StartIfTrue(UnoptRegister),
    EndIfTrue(UnoptRegister),
    UnwrapOrStart(UnoptRegister),
    UnwrapOrEnd(UnoptRegister),
    PushTryCatchStart(UnoptRegister),
    PushTryCatchEnd(UnoptRegister),
}

#[derive(Debug, Clone, Copy)]
enum ProtoOpcode {
    Raw(UnoptOpcode),
    Jump(JumpTo),
    JumpIfFalse(UnoptRegister, JumpTo),
    JumpIfTrue(UnoptRegister, JumpTo),
    UnwrapOrJump(UnoptRegister, JumpTo),
    EnterArrowStatement(JumpTo),
    PushTryCatch(UnoptRegister, JumpTo),
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

#[allow(clippy::type_complexity)]
struct ProtoFunc {
    code: BlockID,
    regs_used: usize,
    span: CodeSpan,
    args: ImmutVec<Spanned<Option<ImmutStr>>>,
    spread_arg: Option<u8>,
    captured_regs: Vec<(UnoptRegister, UnoptRegister)>,
}

// #[derive(Debug)]
pub struct ProtoBytecode {
    consts: UniqueRegister<Constant>,
    functions: Vec<ProtoFunc>,
    blocks: SlabMap<BlockID, Block>,

    import_paths: UniqueRegister<SpwnSource>,

    call_exprs: UniqueRegister<CallExpr<UnoptRegister, UnoptRegister, ImmutStr>>,
    debug_funcs: Vec<FuncID>,
}

impl ProtoBytecode {
    pub fn new() -> Self {
        Self {
            consts: UniqueRegister::new(),
            functions: vec![],
            blocks: SlabMap::new(),
            import_paths: UniqueRegister::new(),
            debug_funcs: vec![],
            call_exprs: UniqueRegister::new(),
        }
    }

    pub fn new_func<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
        args: (ImmutVec<Spanned<Option<ImmutStr>>>, Option<u8>),
        captured_regs: Vec<(UnoptRegister, UnoptRegister)>,
        span: CodeSpan,
    ) -> CompileResult<FuncID> {
        let f_block = self.blocks.insert(Default::default());
        self.functions.push(ProtoFunc {
            code: f_block,
            regs_used: args.0.len() + captured_regs.len(),
            span,
            args: args.0,
            spread_arg: args.1,
            captured_regs,
        });
        let func = self.functions.len() - 1;
        f(&mut CodeBuilder {
            func,
            proto_bytecode: self,
            block: f_block,
        })?;
        Ok(func.into())
    }

    pub fn build(mut self, src: &Rc<SpwnSource>, compiler: &Compiler) -> Result<Bytecode, ()> {
        type BlockPos = (u16, u16);

        let constants = self.consts.make_vec();
        let call_exprs = self.call_exprs.make_vec();

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
                                ProtoOpcode::JumpIfTrue(r, to) => UnoptOpcode::JumpIfTrue {
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
                                ProtoOpcode::PushTryCatch(reg, to) => UnoptOpcode::PushTryCatch {
                                    reg,
                                    to: get_jump_pos(to).into(),
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
                args: func.args.clone(),
                spread_arg: func.spread_arg,
                captured_regs: func
                    .captured_regs
                    .iter()
                    .copied()
                    .map(|(a, b)| (a.try_into().unwrap(), b.try_into().unwrap()))
                    .collect_vec()
                    .into(),
            })
        }

        let import_paths = self.import_paths.make_vec();

        let src_hash = compiler.src_hash();

        Ok(Bytecode {
            source_hash: md5::compute(src.read().unwrap()).into(),
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            constants: constants.into(),
            functions: funcs.into(),
            custom_types: compiler
                .local_type_defs
                .iter()
                .map(|(id, v)| {
                    (
                        CustomTypeID {
                            local: id,
                            source_hash: src_hash,
                        },
                        v.as_ref()
                            .map(|def| compiler.resolve(&def.name).spanned(def.def_span)),
                    )
                })
                .collect(),
            export_names: match &compiler.global_return {
                Some(v) => v.iter().map(|s| compiler.resolve(&s.value)).collect(),
                None => Box::new([]),
            },
            import_paths: import_paths.into(),
            debug_funcs: self.debug_funcs.into(),
            call_exprs: call_exprs
                .into_iter()
                .map(|ce| CallExpr {
                    positional: ce
                        .positional
                        .iter()
                        .cloned()
                        .map(|r| r.try_into().unwrap())
                        .collect_vec()
                        .into(),
                    named: ce
                        .named
                        .iter()
                        .cloned()
                        .map(|(s, r)| (s, r.try_into().unwrap()))
                        .collect_vec()
                        .into(),
                    dest: ce.dest.map(|r| r.try_into().unwrap()),
                })
                .collect_vec()
                .into(),
        })
    }
}

pub struct CodeBuilder<'a> {
    proto_bytecode: &'a mut ProtoBytecode,
    pub func: usize,
    pub block: BlockID,
}

impl<'a> CodeBuilder<'a> {
    pub fn copy(&'a mut self) -> Self {
        Self {
            proto_bytecode: self.proto_bytecode,
            func: self.func,
            block: self.block,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertMarker {
    pub func: usize,
    pub block: BlockID,
}

impl InsertMarker {
    pub fn with<'a>(self, builder: &'a mut CodeBuilder) -> CodeBuilder<'a> {
        CodeBuilder {
            proto_bytecode: builder.proto_bytecode,
            func: self.func,
            block: self.block,
        }
    }
}

impl<'a> CodeBuilder<'a> {
    fn current_block(&mut self) -> &mut Block {
        &mut self.proto_bytecode.blocks[self.block]
    }

    pub fn next_reg(&mut self) -> UnoptRegister {
        let r = Register(self.proto_bytecode.functions[self.func].regs_used);
        self.proto_bytecode.functions[self.func].regs_used += 1;
        r
    }

    pub fn new_block<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
    ) -> CompileResult<()> {
        let f_block = self.proto_bytecode.blocks.insert(Default::default());

        self.current_block()
            .content
            .push(BlockContent::Block(f_block));

        f(&mut CodeBuilder {
            block: f_block,
            func: self.func,
            proto_bytecode: self.proto_bytecode,
        })
    }

    pub fn new_func<F: FnOnce(&mut CodeBuilder) -> CompileResult<()>>(
        &mut self,
        f: F,
        args: (ImmutVec<Spanned<Option<ImmutStr>>>, Option<u8>),
        captured_regs: Vec<(UnoptRegister, UnoptRegister)>,
        span: CodeSpan,
    ) -> CompileResult<FuncID> {
        self.proto_bytecode.new_func(f, args, captured_regs, span)
    }

    fn push_opcode(&mut self, opcode: ProtoOpcode, span: CodeSpan) {
        self.current_block()
            .content
            .push(BlockContent::Opcode(opcode.spanned(span)))
    }

    pub fn mark_func_debug(&mut self, f: FuncID) {
        self.proto_bytecode.debug_funcs.push(f)
    }

    pub fn mark_insert(&mut self) -> InsertMarker {
        let f_block = self.proto_bytecode.blocks.insert(Default::default());

        self.current_block()
            .content
            .push(BlockContent::Block(f_block));

        InsertMarker {
            block: f_block,
            func: self.func,
        }
    }

    pub fn push_raw_opcode(&mut self, opcode: UnoptOpcode, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(opcode), span)
    }

    pub fn load_const<T: Into<Constant>>(&mut self, v: T, reg: UnoptRegister, span: CodeSpan) {
        let id = self.proto_bytecode.consts.insert(v.into()).into();
        self.push_opcode(ProtoOpcode::Raw(Opcode::LoadConst { id, to: reg }), span)
    }

    pub fn jump(&mut self, block: Option<BlockID>, jump_type: JumpType, span: CodeSpan) {
        let block = block.unwrap_or(self.block);
        let opcode = match jump_type {
            JumpType::Start => ProtoOpcode::Jump(JumpTo::Start(block)),
            JumpType::End => ProtoOpcode::Jump(JumpTo::End(block)),
            JumpType::StartIfFalse(reg) => ProtoOpcode::JumpIfFalse(reg, JumpTo::Start(block)),
            JumpType::EndIfFalse(reg) => ProtoOpcode::JumpIfFalse(reg, JumpTo::End(block)),
            JumpType::StartIfTrue(reg) => ProtoOpcode::JumpIfTrue(reg, JumpTo::Start(block)),
            JumpType::EndIfTrue(reg) => ProtoOpcode::JumpIfTrue(reg, JumpTo::End(block)),
            JumpType::UnwrapOrStart(reg) => ProtoOpcode::UnwrapOrJump(reg, JumpTo::Start(block)),
            JumpType::UnwrapOrEnd(reg) => ProtoOpcode::UnwrapOrJump(reg, JumpTo::End(block)),
            JumpType::PushTryCatchStart(reg) => {
                ProtoOpcode::PushTryCatch(reg, JumpTo::Start(block))
            },
            JumpType::PushTryCatchEnd(reg) => ProtoOpcode::PushTryCatch(reg, JumpTo::End(block)),
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

    pub fn copy_shallow(&mut self, from: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::CopyShallow { from, to }), span)
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
        let id = self.proto_bytecode.import_paths.insert(src).into();
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

    pub fn associated(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        member: Spanned<ImmutVec<char>>,
        span: CodeSpan,
    ) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::Associated {
                from,
                dest,
                member: next_reg,
            }),
            span,
        )
    }

    pub fn type_member(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        member: Spanned<ImmutVec<char>>,
        span: CodeSpan,
    ) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::TypeMember {
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

    pub fn mismatch_throw_if_false(
        &mut self,
        check_reg: UnoptRegister,
        value_reg: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::MismatchThrowIfFalse {
                check_reg,
                value_reg,
            }),
            span,
        )
    }

    // pub fn index_set_mem(&mut self, index: UnoptRegister, span: CodeSpan) {
    //     self.push_opcode(ProtoOpcode::Raw(UnoptOpcode::IndexSetMem { index }), span)
    // }

    // pub fn member_set_mem(&mut self, member: Spanned<ImmutVec<char>>, span: CodeSpan) {
    //     let next_reg = self.next_reg();
    //     self.load_const(member.value, next_reg, member.span);
    //     self.push_opcode(
    //         ProtoOpcode::Raw(UnoptOpcode::MemberSetMem { member: next_reg }),
    //         span,
    //     )
    // }

    // pub fn change_mem(&mut self, from: UnoptRegister, span: CodeSpan) {
    //     self.push_opcode(ProtoOpcode::Raw(Opcode::ChangeMem { from }), span)
    // }

    // pub fn write_mem(&mut self, from: UnoptRegister, span: CodeSpan) {
    //     self.push_opcode(ProtoOpcode::Raw(Opcode::WriteMem { from }), span)
    // }

    // pub fn and_op(
    //     &mut self,
    //     left: UnoptRegister,
    //     right: UnoptRegister,
    //     dest: UnoptRegister,
    //     span: CodeSpan,
    // ) -> CompileResult<()> {
    //     self.new_block(|b| {
    //         b.copy_deep(left, dest, span);
    //         b.jump(None, JumpType::EndIfFalse(dest), span);
    //         b.copy_deep(right, dest, span);
    //         Ok(())
    //     })?;
    //     Ok(())
    // }

    // pub fn or_op(
    //     &mut self,
    //     left: UnoptRegister,
    //     right: UnoptRegister,
    //     dest: UnoptRegister,
    //     span: CodeSpan,
    // ) -> CompileResult<()> {
    //     self.new_block(|b| {
    //         b.copy_deep(left, dest, span);
    //         b.jump(None, JumpType::EndIfFalse(dest), span);
    //         b.copy_deep(right, dest, span);
    //         Ok(())
    //     })?;
    //     Ok(())
    // }

    pub fn eq(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Eq { a, b, to }), span)
    }

    pub fn neq(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Neq { a, b, to }), span)
    }

    pub fn gt(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Gt { a, b, to }), span)
    }

    pub fn gte(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Gte { a, b, to }), span)
    }

    pub fn lt(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Lt { a, b, to }), span)
    }

    pub fn lte(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Lte { a, b, to }), span)
    }

    pub fn in_op(&mut self, a: UnoptRegister, b: UnoptRegister, to: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::In { a, b, to }), span)
    }

    pub fn len(&mut self, src: UnoptRegister, dest: UnoptRegister, span: CodeSpan) {
        self.push_opcode(ProtoOpcode::Raw(Opcode::Len { src, dest }), span)
    }

    pub fn associated_mem(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        member: Spanned<ImmutVec<char>>,
        span: CodeSpan,
    ) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::AssociatedMem {
                from,
                dest,
                member: next_reg,
            }),
            span,
        )
    }

    pub fn member_mem(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        member: Spanned<ImmutVec<char>>,
        span: CodeSpan,
    ) {
        let next_reg = self.next_reg();
        self.load_const(member.value, next_reg, member.span);
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::MemberMem {
                from,
                dest,
                member: next_reg,
            }),
            span,
        )
    }

    pub fn index_mem(
        &mut self,
        from: UnoptRegister,
        dest: UnoptRegister,
        index: UnoptRegister,
        span: CodeSpan,
    ) {
        self.push_opcode(
            ProtoOpcode::Raw(UnoptOpcode::IndexMem {
                base: from,
                dest,
                index,
            }),
            span,
        )
    }

    pub fn call(
        &mut self,
        base: UnoptRegister,
        v: CallExpr<UnoptRegister, UnoptRegister, ImmutStr>,
        span: CodeSpan,
    ) {
        let id = self.proto_bytecode.call_exprs.insert(v).into();
        self.push_opcode(ProtoOpcode::Raw(Opcode::Call { base, call: id }), span)
    }
}
