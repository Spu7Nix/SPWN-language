use itertools::Itertools;

use super::{CompileResult, Compiler, ScopeID};
use crate::compiling::builder::CodeBuilder;
use crate::compiling::bytecode::UnoptRegister;
use crate::compiling::opcodes::Opcode;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{Pattern, PatternNode};

impl Compiler<'_> {
    pub fn compile_pattern_check(
        &mut self,
        expr_reg: UnoptRegister,
        pattern: &PatternNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<UnoptRegister> {
        let out_reg = builder.next_reg();

        match &*pattern.pat {
            Pattern::Any => {
                builder.load_const(true, out_reg, pattern.span);
            },
            Pattern::Type(t) => {
                let a = builder.next_reg();
                self.load_type(t, a, pattern.span, builder)?;
                let b = builder.next_reg();
                builder.type_of(expr_reg, b, pattern.span);
                builder.eq(a, b, out_reg, pattern.span);
            },
            Pattern::Either(a, b) => {
                self.or_op(
                    &[
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, a, scope, builder)
                        },
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, b, scope, builder)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::Both(a, b) => {
                self.and_op(
                    &[
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, a, scope, builder)
                        },
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, b, scope, builder)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::Eq(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.eq(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::Neq(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.neq(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::Lt(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.lt(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::Lte(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.lte(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::Gt(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.gt(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::Gte(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.gte(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::In(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.in_op(expr_reg, reg, out_reg, pattern.span);
            },
            Pattern::ArrayPattern(p, len) => todo!(),
            Pattern::DictPattern(_) => todo!(),
            Pattern::ArrayDestructure(v) => {
                self.and_op(
                    &[
                        &|_, builder| {
                            let arr_typ = builder.next_reg();
                            builder.load_const(ValueType::Array, arr_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.eq(expr_typ, arr_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|_, builder| {
                            let pat_len = builder.next_reg();
                            builder.load_const(v.len() as i64, pat_len, pattern.span);

                            let expr_len = builder.next_reg();
                            builder.len(expr_reg, expr_len, pattern.span);

                            let gte_reg = builder.next_reg();
                            builder.gte(expr_len, pat_len, gte_reg, pattern.span);

                            Ok(gte_reg)
                        },
                        &|compiler, builder| {
                            let mut funcs = vec![];

                            #[allow(clippy::type_complexity)]
                            for (i, elem) in v.iter().enumerate() {
                                let f: Box<
                                    dyn Fn(
                                        &mut Compiler<'_>,
                                        &mut CodeBuilder<'_>,
                                    )
                                        -> CompileResult<UnoptRegister>,
                                > = Box::new(move |compiler, builder| {
                                    // todo!()
                                    let idx = builder.next_reg();
                                    builder.load_const(i as i64, idx, pattern.span);

                                    let elem_reg = builder.next_reg();
                                    builder.index(expr_reg, elem_reg, idx, pattern.span);

                                    compiler.compile_pattern_check(elem_reg, elem, scope, builder)
                                });
                                funcs.push(f);
                            }
                            let all_reg = builder.next_reg();
                            builder.load_const(true, all_reg, pattern.span);

                            compiler.and_op(
                                &funcs.iter().map(|e| &**e).collect_vec()[..],
                                all_reg,
                                pattern.span,
                                builder,
                            )?;

                            Ok(all_reg)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::DictDestructure(_) => todo!(),
            Pattern::MaybeDestructure(_) => todo!(),
            Pattern::InstanceDestructure(..) => todo!(),
            Pattern::Path { path, is_ref } => todo!(),
            Pattern::Mut { name, is_ref } => todo!(),
            Pattern::MacroPattern { args, ret_type } => todo!(),
        }

        Ok(out_reg)
    }
}
