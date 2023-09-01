use std::str::FromStr;

use ahash::AHashMap;
use itertools::Itertools;
use lasso::Spur;

use super::{CompileResult, Compiler, ScopeID, ScopeType, VarData};
use crate::compiling::builder::{CodeBuilder, JumpType};
use crate::compiling::bytecode::UnoptRegister;
use crate::compiling::error::CompileError;
use crate::compiling::opcodes::Opcode;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{AssignPath, ExprNode, Pattern, PatternNode, VisTrait};
use crate::sources::{CodeSpan, Spannable, ZEROSPAN};

pub struct PathInfo {
    pub reg: UnoptRegister,
    pub exists: Option<bool>,
}

impl Compiler<'_> {
    pub fn get_path_reg(
        &mut self,
        var: Spur,
        try_new: bool,
        path: &[AssignPath],
        scope: ScopeID,
        builder: &mut CodeBuilder,
        span: CodeSpan,
    ) -> CompileResult<PathInfo> {
        macro_rules! normal_access {
            () => {
                match self.get_var(var, scope) {
                    Some(v) => PathInfo {
                        reg: v.reg,
                        exists: Some(v.mutable),
                    },
                    // Some(v) if v.mutable => (v.reg, false),
                    // Some(v) => {
                    //     return Err(CompileError::ImmutableAssign {
                    //         area: self.make_area(span),
                    //         def_area: self.make_area(v.def_span),
                    //         var: self.resolve(&var),
                    //     })
                    // },
                    None => {
                        let r = builder.next_reg();
                        if path.is_empty() {
                            self.scopes[scope].vars.insert(
                                var,
                                VarData {
                                    mutable: false,
                                    def_span: span,
                                    reg: r,
                                },
                            );
                            PathInfo {
                                reg: r,
                                exists: None,
                            }
                        } else {
                            return Err(CompileError::NonexistentVariable {
                                area: self.make_area(span),
                                var: self.resolve(&var),
                            });
                        }
                    },
                }
            };
        }

        let path_info = if !try_new {
            normal_access!()
        } else if path.is_empty() {
            let r = builder.next_reg();
            self.scopes[scope].vars.insert(
                var,
                VarData {
                    mutable: false,
                    def_span: span,
                    reg: r,
                },
            );
            PathInfo {
                reg: r,
                exists: None,
            }
        } else {
            normal_access!()
        };

        if !path.is_empty() {
            let path_reg = builder.next_reg();
            builder.copy_ref(path_info.reg, path_reg, span);
            for i in path {
                match i {
                    AssignPath::Index(v) => {
                        let v = self.compile_expr(v, scope, builder)?;
                        builder.index(path_reg, path_reg, v, span);
                    },
                    AssignPath::Member(v) => {
                        builder.member(
                            path_reg,
                            path_reg,
                            self.resolve_32(v).spanned(span),
                            false,
                            span,
                        );
                    },
                    AssignPath::Associated(v) => {
                        builder.associated(
                            path_reg,
                            path_reg,
                            self.resolve_32(v).spanned(span),
                            span,
                        );
                    },
                }
            }

            Ok(PathInfo {
                reg: path_reg,
                exists: path_info.exists,
            })
        } else {
            Ok(path_info)
        }
        // println!("{}", path_reg);
    }

    pub fn compile_pattern_check(
        &mut self,
        expr_reg: UnoptRegister,
        pattern: &PatternNode,
        try_new_var: bool,
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
                builder.pure_eq(a, b, out_reg, pattern.span);
            },
            Pattern::IfGuard { pat, cond } => {
                self.and_op(
                    &[
                        &|compiler, builder| {
                            compiler.compile_pattern_check(
                                expr_reg,
                                pat,
                                try_new_var,
                                scope,
                                builder,
                            )
                        },
                        &|compiler, builder| compiler.compile_expr(cond, scope, builder),
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::Either(a, b) => {
                self.or_op(
                    &[
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, a, try_new_var, scope, builder)
                        },
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, b, try_new_var, scope, builder)
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
                            compiler.compile_pattern_check(expr_reg, a, try_new_var, scope, builder)
                        },
                        &|compiler, builder| {
                            compiler.compile_pattern_check(expr_reg, b, try_new_var, scope, builder)
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
            Pattern::ArrayPattern(elem_pat, len_pat) => {
                let expr_len = builder.next_reg();

                self.and_op(
                    &[
                        &|_, builder| {
                            let arr_typ = builder.next_reg();
                            builder.load_const(ValueType::Array, arr_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, arr_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|compiler, builder| {
                            builder.len(expr_reg, expr_len, pattern.span);

                            let len_match = compiler.compile_pattern_check(
                                expr_len,
                                len_pat,
                                try_new_var,
                                scope,
                                builder,
                            )?;

                            Ok(len_match)
                        },
                        &|compiler, builder| {
                            let idx_reg = builder.next_reg();
                            builder.load_const(0, idx_reg, pattern.span);

                            let one_reg = builder.next_reg();
                            builder.load_const(1, one_reg, pattern.span);

                            let out_reg = builder.next_reg();
                            builder.load_const(true, out_reg, pattern.span);

                            builder.new_block(|builder| {
                                let outer = builder.block;

                                builder.new_block(|builder| {
                                    let gt_reg = builder.next_reg();
                                    builder.gt(expr_len, idx_reg, gt_reg, pattern.span);
                                    builder.jump(None, JumpType::EndIfFalse(gt_reg), pattern.span);

                                    let elem = builder.next_reg();
                                    builder.index(expr_reg, elem, idx_reg, pattern.span);

                                    let elem_match = compiler.compile_pattern_check(
                                        elem,
                                        elem_pat,
                                        try_new_var,
                                        scope,
                                        builder,
                                    )?;
                                    builder.copy_shallow(elem_match, out_reg, pattern.span);
                                    builder.jump(
                                        Some(outer),
                                        JumpType::EndIfFalse(out_reg),
                                        pattern.span,
                                    );

                                    builder.push_raw_opcode(
                                        Opcode::PlusEq {
                                            a: idx_reg,
                                            b: one_reg,
                                            left_mut: true,
                                        },
                                        pattern.span,
                                    );

                                    builder.jump(None, JumpType::Start, pattern.span);

                                    Ok(())
                                })?;

                                Ok(())
                            })?;

                            Ok(out_reg)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::ArrayDestructure(array_pat) => {
                self.and_op(
                    &[
                        &|_, builder| {
                            let arr_typ = builder.next_reg();
                            builder.load_const(ValueType::Array, arr_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, arr_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|_, builder| {
                            let pat_len = builder.next_reg();
                            builder.load_const(array_pat.len() as i64, pat_len, pattern.span);

                            let expr_len = builder.next_reg();
                            builder.len(expr_reg, expr_len, pattern.span);

                            let gte_reg = builder.next_reg();
                            builder.pure_eq(expr_len, pat_len, gte_reg, pattern.span);

                            Ok(gte_reg)
                        },
                        &|compiler, builder| {
                            let mut funcs = vec![];

                            for (i, elem) in array_pat.iter().enumerate() {
                                let f: Box<
                                    dyn Fn(
                                        &mut Compiler<'_>,
                                        &mut CodeBuilder<'_>,
                                    )
                                        -> CompileResult<UnoptRegister>,
                                > = Box::new(move |compiler, builder| {
                                    let idx = builder.next_reg();
                                    builder.load_const(i as i64, idx, pattern.span);

                                    let elem_reg = builder.next_reg();
                                    builder.index(expr_reg, elem_reg, idx, pattern.span);

                                    compiler.compile_pattern_check(
                                        elem_reg,
                                        elem,
                                        try_new_var,
                                        scope,
                                        builder,
                                    )
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
            Pattern::DictDestructure(dict_pat) => {
                self.and_op(
                    &[
                        &|_, builder| {
                            let dict_typ = builder.next_reg();
                            builder.load_const(ValueType::Dict, dict_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, dict_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|_, builder| {
                            let pat_len = builder.next_reg();
                            builder.load_const(dict_pat.len() as i64, pat_len, pattern.span);

                            let expr_len = builder.next_reg();
                            builder.len(expr_reg, expr_len, pattern.span);

                            let gte_reg = builder.next_reg();
                            builder.pure_gte(expr_len, pat_len, gte_reg, pattern.span);

                            Ok(gte_reg)
                        },
                        &|compiler, builder| {
                            let mut funcs = vec![];

                            for (key, elem) in dict_pat.iter() {
                                let f: Box<
                                    dyn Fn(
                                        &mut Compiler<'_>,
                                        &mut CodeBuilder<'_>,
                                    )
                                        -> CompileResult<UnoptRegister>,
                                > = Box::new(move |compiler, builder| {
                                    let elem_reg = builder.next_reg();

                                    builder.member(
                                        expr_reg,
                                        elem_reg,
                                        key.map(|s| compiler.resolve_32(&s)),
                                        false,
                                        pattern.span,
                                    );

                                    compiler.compile_pattern_check(
                                        elem_reg,
                                        elem,
                                        try_new_var,
                                        scope,
                                        builder,
                                    )
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
            Pattern::DictPattern(value_pat) => {
                /*
                   @bool{:} = {a: true}

                   essentially performs

                   for [_, v] in {a: true} {
                       v is @bool
                   }
                */
                self.and_op(
                    &[
                        &|_, builder| {
                            let dict_typ = builder.next_reg();
                            builder.load_const(ValueType::Dict, dict_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, dict_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        // makes and iterates the iterator
                        &|compiler, builder| {
                            let out_reg = builder.next_reg();
                            builder.load_const(true, out_reg, pattern.span);

                            let iter_reg = builder.next_reg();
                            builder.push_raw_opcode(
                                Opcode::IntoIterator {
                                    src: expr_reg,
                                    dest: iter_reg,
                                },
                                pattern.span,
                            );

                            builder.new_block(|builder| {
                                let outer = builder.block;

                                builder.new_block(|builder| {
                                    let next_reg = builder.next_reg();
                                    builder.iter_next(iter_reg, next_reg, pattern.span);

                                    builder.jump(
                                        None,
                                        JumpType::UnwrapOrEnd(next_reg),
                                        pattern.span,
                                    );

                                    let loop_scope = compiler
                                        .derive_scope(scope, Some(ScopeType::Loop(builder.block)));

                                    // var with impossible name to avoid conflicts
                                    let fake_var = compiler.intern("#v");

                                    // destructures key/value pair with pattern `[_, #v]`
                                    let fake_iter_pat = PatternNode {
                                        pat: Box::new(Pattern::ArrayDestructure(vec![
                                            PatternNode {
                                                pat: Box::new(Pattern::Any),
                                                span: ZEROSPAN,
                                            },
                                            PatternNode {
                                                pat: Box::new(Pattern::Path {
                                                    var: fake_var,
                                                    path: vec![],
                                                }),
                                                span: value_pat.span,
                                            },
                                        ])),
                                        span: ZEROSPAN,
                                    };

                                    // should always match
                                    let iter_pat_matches = compiler.compile_pattern_check(
                                        next_reg,
                                        &fake_iter_pat,
                                        true,
                                        loop_scope,
                                        builder,
                                    )?;
                                    builder.mismatch_throw_if_false(
                                        iter_pat_matches,
                                        next_reg,
                                        pattern.span,
                                    );

                                    let fake_var_data = compiler
                                        .get_var(fake_var, loop_scope)
                                        .expect("BUG: #v missing"); // var created above so should exist

                                    let dict_value_matches = compiler.compile_pattern_check(
                                        fake_var_data.reg,
                                        value_pat,
                                        false,
                                        scope,
                                        builder,
                                    )?;
                                    builder.copy_shallow(dict_value_matches, out_reg, pattern.span);
                                    builder.jump(
                                        Some(outer),
                                        JumpType::EndIfFalse(out_reg),
                                        pattern.span,
                                    );

                                    builder.jump(None, JumpType::Start, pattern.span);

                                    Ok(())
                                })?;

                                Ok(())
                            })?;

                            Ok(out_reg)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::InstanceDestructure(typ_name, pat) => {
                self.and_op(
                    &[
                        &|compiler, builder| {
                            let instance_typ_reg = builder.next_reg();

                            let str_name = compiler.resolve(typ_name);

                            if ValueType::from_str(&str_name).is_ok() {
                                return Err(CompileError::BuiltinTypeDestructure {
                                    area: compiler.make_area(pattern.span),
                                });
                            }

                            compiler.load_type(
                                typ_name,
                                instance_typ_reg,
                                pattern.span,
                                builder,
                            )?;

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, instance_typ_reg, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|_, builder| {
                            let pat_len = builder.next_reg();
                            builder.load_const(pat.len() as i64, pat_len, pattern.span);

                            let expr_len = builder.next_reg();
                            builder.len(expr_reg, expr_len, pattern.span);

                            let gte_reg = builder.next_reg();
                            builder.pure_gte(expr_len, pat_len, gte_reg, pattern.span);

                            Ok(gte_reg)
                        },
                        // same as dict destructure
                        &|compiler, builder| {
                            let mut funcs = vec![];

                            for (key, elem) in pat.iter() {
                                let f: Box<
                                    dyn Fn(
                                        &mut Compiler<'_>,
                                        &mut CodeBuilder<'_>,
                                    )
                                        -> CompileResult<UnoptRegister>,
                                > = Box::new(move |compiler, builder| {
                                    let elem_reg = builder.next_reg();

                                    builder.member(
                                        expr_reg,
                                        elem_reg,
                                        key.map(|s| compiler.resolve_32(&s)),
                                        false,
                                        pattern.span,
                                    );

                                    compiler.compile_pattern_check(
                                        elem_reg,
                                        elem,
                                        try_new_var,
                                        scope,
                                        builder,
                                    )
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
            // <pat>? or ?
            Pattern::MaybeDestructure(maybe) => {
                //
                self.and_op(
                    &[
                        &|_, builder| {
                            let maybe_typ = builder.next_reg();
                            builder.load_const(ValueType::Maybe, maybe_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, maybe_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|compiler, builder| {
                            let out_reg = builder.next_reg();
                            builder.load_const(true, out_reg, pattern.span);

                            match maybe {
                                Some(maybe_pat) => {
                                    compiler.and_op(
                                        &[
                                            // checks first if the value is not `?`
                                            &|_, builder| {
                                                let option_none = builder.next_reg();
                                                builder.load_const(None, option_none, pattern.span); // `None` being `?`

                                                let is_not_none = builder.next_reg();
                                                builder.pure_neq(
                                                    option_none,
                                                    expr_reg,
                                                    is_not_none,
                                                    pattern.span,
                                                );

                                                Ok(is_not_none)
                                            },
                                            // then checks if the pattern matches
                                            &|compiler, builder| {
                                                let maybe_matches = compiler
                                                    .compile_pattern_check(
                                                        expr_reg, maybe_pat, false, scope, builder,
                                                    )?;

                                                Ok(maybe_matches)
                                            },
                                        ],
                                        out_reg,
                                        maybe_pat.span,
                                        builder,
                                    )?;
                                },
                                None => {
                                    let option_none = builder.next_reg();
                                    builder.load_const(None, option_none, pattern.span); // `None` being `?`

                                    let is_not_none = builder.next_reg();
                                    builder.pure_eq(
                                        option_none,
                                        expr_reg,
                                        is_not_none,
                                        pattern.span,
                                    );

                                    builder.copy_shallow(is_not_none, out_reg, pattern.span);
                                },
                            };

                            Ok(out_reg)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::Path { var, path } => {
                // if *is_ref && !path.is_empty() {
                //     return Err(CompileError::IllegalAssign {
                //         area: self.make_area(pattern.span),
                //     });
                // }
                // println!("{}", try_new_var);

                let path_info =
                    self.get_path_reg(*var, try_new_var, path, scope, builder, pattern.span)?;

                // if *is_ref {
                //     match path_info.exists {
                //         Some(mutable) => {
                //             builder.assign_ref(expr_reg, path_info.reg, mutable, pattern.span)
                //         },
                //         None => builder.copy_ref(expr_reg, path_info.reg, pattern.span),
                //     }
                // } else {
                match path_info.exists {
                    Some(mutable) => {
                        builder.assign_deep(expr_reg, path_info.reg, mutable, pattern.span)
                    },
                    None => builder.write_deep(expr_reg, path_info.reg, pattern.span),
                }
                // }
                builder.load_const(true, out_reg, pattern.span);
            },
            Pattern::Mut { name } => {
                let var_reg = builder.next_reg();
                self.scopes[scope].vars.insert(
                    *name,
                    VarData {
                        mutable: true,
                        def_span: pattern.span,
                        reg: var_reg,
                    },
                );
                // if *is_ref {
                //     builder.copy_ref(expr_reg, var_reg, pattern.span);
                // } else {
                builder.write_deep(expr_reg, var_reg, pattern.span);
                // }
                builder.load_const(true, out_reg, pattern.span);
            },
            Pattern::Ref { name } => {
                let var_reg = builder.next_reg();
                self.scopes[scope].vars.insert(
                    *name,
                    VarData {
                        mutable: true,
                        def_span: pattern.span,
                        reg: var_reg,
                    },
                );
                // if *is_ref {
                builder.copy_ref(expr_reg, var_reg, pattern.span);
                // } else {
                // builder.write_deep(expr_reg, var_reg, pattern.span);
                // }
                builder.load_const(true, out_reg, pattern.span);
            },
            Pattern::MacroPattern { args, .. } => {
                self.and_op(
                    &[
                        &|_, builder| {
                            let macro_typ = builder.next_reg();
                            builder.load_const(ValueType::Macro, macro_typ, pattern.span);

                            let expr_typ = builder.next_reg();
                            builder.type_of(expr_reg, expr_typ, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_typ, macro_typ, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                        &|_, builder| {
                            let pat_arg_amount = builder.next_reg();
                            builder.load_const(args.len() as i64, pat_arg_amount, pattern.span);

                            let expr_arg_amount = builder.next_reg();
                            builder.arg_amount(expr_reg, expr_arg_amount, pattern.span);

                            let eq_reg = builder.next_reg();
                            builder.pure_eq(expr_arg_amount, pat_arg_amount, eq_reg, pattern.span);

                            Ok(eq_reg)
                        },
                    ],
                    out_reg,
                    pattern.span,
                    builder,
                )?;
            },
            Pattern::Empty => {
                let macro_typ = builder.next_reg();
                builder.load_const(ValueType::Empty, macro_typ, pattern.span);

                let expr_typ = builder.next_reg();
                builder.type_of(expr_reg, expr_typ, pattern.span);

                builder.pure_eq(expr_typ, macro_typ, out_reg, pattern.span);
            },
        }

        Ok(out_reg)
    }
}
