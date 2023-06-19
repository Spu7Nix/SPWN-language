use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use lasso::Spur;

use super::builder::{BlockID, CodeBuilder, JumpType};
use super::bytecode::{Bytecode, UnoptRegister};
use super::error::CompileError;
use crate::compiling::builder::ProtoBytecode;
use crate::compiling::opcodes::Opcode;
use crate::new_id_wrapper;
use crate::parsing::ast::{Ast, ExprNode, Expression, Statement, StmtNode};
use crate::parsing::utils::operators::{AssignOp, BinOp, UnaryOp};
use crate::sources::{CodeArea, CodeSpan, SpwnSource};
use crate::util::{ImmutCloneStr, ImmutStr, Interner, SlabMap};

pub type CompileResult<T> = Result<T, CompileError>;

new_id_wrapper! {
    ScopeID: u16;
}

#[derive(Debug, Clone, Copy)]
pub struct VarData {
    mutable: bool,
    def_span: CodeSpan,
    reg: UnoptRegister,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeType {
    Global,
    Loop(BlockID),
    MacroBody,
    TriggerFunc(CodeSpan),
    ArrowStmt(CodeSpan),
}

#[derive(Debug, Clone)]
pub struct Scope {
    vars: AHashMap<Spur, VarData>,
    parent: Option<ScopeID>,
    typ: Option<ScopeType>,
}

pub struct Compiler {
    src: Rc<SpwnSource>,
    interner: Rc<RefCell<Interner>>,
    scopes: SlabMap<ScopeID, Scope>,
}

impl Compiler {
    pub fn new(src: Rc<SpwnSource>, interner: Rc<RefCell<Interner>>) -> Self {
        Self {
            src,
            interner,
            scopes: SlabMap::new(),
        }
    }
}

impl Compiler {
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&self.src),
        }
    }

    pub fn resolve(&self, s: &Spur) -> Box<str> {
        self.interner.borrow().resolve(s).into()
    }

    pub fn derive_scope(&mut self, scope: ScopeID, typ: Option<ScopeType>) -> ScopeID {
        let scope = Scope {
            vars: AHashMap::new(),
            parent: Some(scope),
            typ,
        };
        self.scopes.insert(scope)
    }

    pub fn get_var(&self, var: Spur, scope: ScopeID) -> Option<VarData> {
        match self.scopes[scope].vars.get(&var) {
            Some(v) => Some(*v),
            None => match self.scopes[scope].parent {
                Some(p) => self.get_var(var, p),
                None => None,
            },
        }
    }

    fn is_inside_macro(&self, scope: ScopeID) -> bool {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::MacroBody => return true,
                ScopeType::Global => return false,
                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.is_inside_macro(k),
            None => false,
        }
    }

    pub fn assert_can_return(&self, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
        fn can_return_d(slf: &Compiler, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
            let scope = &slf.scopes[scope];
            match &scope.typ {
                Some(ScopeType::MacroBody) => return Ok(()),
                Some(ScopeType::TriggerFunc(def)) => {
                    return Err(CompileError::BreakInTriggerFuncScope {
                        area: slf.make_area(span),
                        def: slf.make_area(*def),
                    })
                },
                Some(ScopeType::ArrowStmt(def)) => {
                    return Err(CompileError::BreakInArrowStmtScope {
                        area: slf.make_area(span),
                        def: slf.make_area(*def),
                    })
                },
                Some(_) => (),
                None => (),
            }
            match scope.parent {
                Some(k) => can_return_d(slf, k, span),
                None => unreachable!(),
            }
        }

        if let Some(ScopeType::ArrowStmt(_)) = self.scopes[scope].typ {
            return Ok(()); // -> return
        }

        can_return_d(self, scope, span)
    }

    pub fn compile_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<UnoptRegister> {
        match &*expr.expr {
            Expression::Int(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::Float(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::String(v) => {
                todo!()
                // let reg = builder.next_reg();
                // builder.load_const(*v, reg, expr.span);
                // Ok(reg)
            },
            Expression::Bool(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::Op(left, op, right) => {
                let left = self.compile_expr(left, scope, builder)?;
                let right = self.compile_expr(right, scope, builder)?;

                let dest = builder.next_reg();

                macro_rules! bin_op {
                    ($opcode_name:ident) => {
                        builder.push_raw_opcode(
                            Opcode::$opcode_name {
                                a: left,
                                b: right,
                                to: dest,
                            },
                            expr.span,
                        )
                    };
                }

                match op {
                    BinOp::Plus => bin_op!(Plus),
                    BinOp::Minus => bin_op!(Minus),
                    BinOp::Mult => bin_op!(Mult),
                    BinOp::Div => bin_op!(Div),
                    BinOp::Mod => bin_op!(Mod),
                    BinOp::Pow => bin_op!(Pow),
                    BinOp::Eq => bin_op!(Eq),
                    BinOp::Neq => bin_op!(Neq),
                    BinOp::Gt => bin_op!(Gt),
                    BinOp::Gte => bin_op!(Gte),
                    BinOp::Lt => bin_op!(Lt),
                    BinOp::Lte => bin_op!(Lte),
                    BinOp::BinOr => bin_op!(BinOr),
                    BinOp::BinAnd => bin_op!(BinAnd),
                    BinOp::Range => bin_op!(Range),
                    BinOp::In => bin_op!(In),
                    BinOp::ShiftLeft => bin_op!(ShiftLeft),
                    BinOp::ShiftRight => bin_op!(ShiftRight),
                    BinOp::As => bin_op!(As),
                    BinOp::Or => todo!(),
                    BinOp::And => todo!(),
                }
                Ok(dest)
            },
            Expression::Unary(op, value) => {
                let value = self.compile_expr(value, scope, builder)?;
                let dest = builder.next_reg();

                macro_rules! unary_op {
                    ($opcode_name:ident) => {
                        builder
                            .push_raw_opcode(Opcode::$opcode_name { v: value, to: dest }, expr.span)
                    };
                }

                match op {
                    UnaryOp::ExclMark => unary_op!(Not),
                    UnaryOp::Minus => unary_op!(Negate),
                }

                Ok(dest)
            },
            Expression::Var(v) => match self.get_var(*v, scope) {
                Some(v) => Ok(v.reg),
                None => Err(CompileError::NonexistentVariable {
                    area: self.make_area(expr.span),
                    var: self.resolve(v),
                }),
            },
            Expression::Type(_) => todo!(),
            Expression::Array(v) => {
                let dest = builder.next_reg();

                builder.alloc_array(dest, v.len() as u16, expr.span);
                for e in v {
                    let r = self.compile_expr(e, scope, builder)?;
                    builder.push_array_elem(r, dest, expr.span);
                }

                Ok(dest)
            },
            Expression::Dict(v) => {
                let dest = builder.next_reg();

                builder.alloc_dict(dest, v.len() as u16, expr.span);
                for e in v {
                    let r = match &e.value {
                        Some(e) => self.compile_expr(e, scope, builder)?,
                        None => match self.get_var(e.name.value, scope) {
                            Some(v) => v.reg,
                            None => {
                                return Err(CompileError::NonexistentVariable {
                                    area: self.make_area(expr.span),
                                    var: self.resolve(&e.name),
                                })
                            },
                        },
                    };
                    let k = builder.next_reg();
                    builder.load_const::<ImmutStr>(self.resolve(&e.name.value), k, expr.span);
                    builder.insert_dict_elem(r, dest, k, expr.span)
                }

                Ok(dest)
            },
            Expression::Maybe(_) => todo!(),
            Expression::Is(..) => todo!(),
            Expression::Index { base, index } => todo!(),
            Expression::Member { base, name } => todo!(),
            Expression::TypeMember { base, name } => todo!(),
            Expression::Associated { base, name } => todo!(),
            Expression::Call {
                base,
                params,
                named_params,
            } => todo!(),
            Expression::Macro {
                args,
                ret_type,
                code,
            } => todo!(),
            Expression::TriggerFunc { attributes, code } => todo!(),
            Expression::TriggerFuncCall(_) => todo!(),
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => todo!(),
            Expression::Typeof(_) => todo!(),
            Expression::Builtins => todo!(),
            Expression::Empty => todo!(),
            Expression::Epsilon => todo!(),
            Expression::Import(_) => todo!(),
            Expression::Instance { base, items } => todo!(),
        }
    }

    pub fn compile_stmt(
        &mut self,
        stmt: &StmtNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<()> {
        match &*stmt.stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, builder)?;
            },
            Statement::Let(left, right) => match &*left.expr {
                Expression::Var(v) => {
                    let reg = builder.next_reg();
                    self.scopes[scope].vars.insert(
                        *v,
                        VarData {
                            mutable: true,
                            def_span: stmt.span,
                            reg,
                        },
                    );
                    let e = self.compile_expr(right, scope, builder)?;
                    builder.copy_deep(e, reg, stmt.span);
                },
                _ => todo!("destruction !!!!!!!!!!!!!!!!!!!!!!!!"),
            },
            Statement::AssignOp(left, op, right) => {
                let right_reg = self.compile_expr(right, scope, builder)?;

                let var = match &*left.expr {
                    Expression::Var(v) => *v,
                    _ => todo!("destruction !!!!!!!!!!!!!!!!!!!!!!!!"),
                };

                macro_rules! assign_op {
                    ($opcode_name:ident) => {
                        match self.get_var(var, scope) {
                            Some(data) if data.mutable => builder.push_raw_opcode(
                                Opcode::$opcode_name {
                                    a: data.reg,
                                    b: right_reg,
                                },
                                stmt.span,
                            ),
                            Some(data) => {
                                return Err(CompileError::ImmutableAssign {
                                    area: self.make_area(stmt.span),
                                    def_area: self.make_area(data.def_span),
                                    var: self.resolve(&var),
                                })
                            },
                            None => {
                                return Err(CompileError::NonexistentVariable {
                                    area: self.make_area(left.span),
                                    var: self.resolve(&var),
                                })
                            },
                        }
                    };
                }

                match op {
                    AssignOp::Assign => match self.get_var(var, scope) {
                        Some(data) if data.mutable => {
                            builder.copy_deep(right_reg, data.reg, stmt.span);
                        },
                        Some(data) => {
                            return Err(CompileError::ImmutableAssign {
                                area: self.make_area(stmt.span),
                                def_area: self.make_area(data.def_span),
                                var: self.resolve(&var),
                            })
                        },
                        None => {
                            let reg = builder.next_reg();
                            self.scopes[scope].vars.insert(
                                var,
                                VarData {
                                    mutable: false,
                                    def_span: stmt.span,
                                    reg,
                                },
                            );
                            builder.copy_deep(right_reg, reg, stmt.span);
                        },
                    },
                    AssignOp::PlusEq => assign_op!(PlusEq),
                    AssignOp::MinusEq => assign_op!(MinusEq),
                    AssignOp::MultEq => assign_op!(MultEq),
                    AssignOp::DivEq => assign_op!(DivEq),
                    AssignOp::PowEq => assign_op!(PowEq),
                    AssignOp::ModEq => assign_op!(ModEq),
                    AssignOp::BinAndEq => assign_op!(BinAndEq),
                    AssignOp::BinOrEq => assign_op!(BinOrEq),
                    AssignOp::ShiftLeftEq => assign_op!(ShiftLeftEq),
                    AssignOp::ShiftRightEq => assign_op!(ShiftRightEq),
                }
            },
            Statement::If {
                branches,
                else_branch,
            } => {
                builder.new_block(|b| {
                    let outer = b.block;

                    for (cond, code) in branches {
                        b.new_block(|b| {
                            let cond_reg = self.compile_expr(cond, scope, b)?;
                            b.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                            let derived = self.derive_scope(scope, None);

                            for s in code {
                                self.compile_stmt(s, derived, b)?;
                            }
                            b.jump(Some(outer), JumpType::End, stmt.span);
                            Ok(())
                        })?;
                    }

                    if let Some(code) = else_branch {
                        let derived = self.derive_scope(scope, None);

                        for s in code {
                            self.compile_stmt(s, derived, b)?;
                        }
                    }

                    Ok(())
                })?;
            },
            Statement::While { cond, code } => {
                builder.new_block(|b| {
                    let cond_reg = self.compile_expr(cond, scope, b)?;
                    b.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                    let derived = self.derive_scope(scope, Some(ScopeType::Loop(b.block)));

                    for s in code {
                        self.compile_stmt(s, derived, b)?;
                    }
                    b.jump(None, JumpType::Start, stmt.span);

                    Ok(())
                })?;
            },
            Statement::For {
                iter_var,
                iterator,
                code,
            } => {
                let iter_exec = self.compile_expr(iterator, scope, builder)?;
                let iter_reg = builder.next_reg();
                builder.push_raw_opcode(
                    Opcode::WrapIterator {
                        src: iter_exec,
                        dest: iter_reg,
                    },
                    stmt.span,
                );

                builder.new_block(|b| {
                    let next_reg = b.next_reg();
                    b.iter_next(iter_reg, next_reg, iter_var.span);

                    b.jump(None, JumpType::UnwrapOrEnd(next_reg), iterator.span);

                    let derived = self.derive_scope(scope, Some(ScopeType::Loop(b.block)));

                    match &*iter_var.expr {
                        Expression::Var(v) => {
                            let var_reg = b.next_reg();
                            self.scopes[derived].vars.insert(
                                *v,
                                VarData {
                                    mutable: false,
                                    def_span: iter_var.span,
                                    reg: var_reg,
                                },
                            );
                            b.copy_deep(next_reg, var_reg, iter_var.span);
                        },
                        _ => todo!("destruction !!!!!!!!!!!!!!!!!!!!!!!!"),
                    };

                    for s in code {
                        self.compile_stmt(s, derived, b)?;
                    }
                    b.jump(None, JumpType::Start, stmt.span);

                    Ok(())
                })?;
            },
            Statement::TryCatch {
                try_code,
                branches,
                catch_all,
            } => todo!(),
            Statement::Arrow(s) => {
                builder.new_block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::ArrowStmt(stmt.span))); // variables made in arrow statements shouldnt be allowed anyways
                    b.enter_arrow(stmt.span);
                    self.compile_stmt(&s, inner_scope, b)?;
                    b.yeet_context(stmt.span);
                    Ok(())
                })?;
            },
            Statement::Return(v) => match self.scopes[scope].typ {
                Some(ScopeType::Global) => {},
                _ => {
                    if self.is_inside_macro(scope) {
                        self.assert_can_return(scope, stmt.span)?;
                        match v {
                            None => {
                                let out_reg = builder.next_reg();
                                builder.load_empty(out_reg, stmt.span);
                                builder.ret(out_reg, false, stmt.span)
                            },
                            Some(expr) => {
                                let ret_reg = self.compile_expr(expr, scope, builder)?;
                                builder.ret(ret_reg, false, stmt.span)
                            },
                        }
                    } else {
                        return Err(CompileError::ReturnOutsideMacro {
                            area: self.make_area(stmt.span),
                        });
                    }
                },
            },
            Statement::Break => todo!(),
            Statement::Continue => todo!(),
            Statement::TypeDef { name, private } => todo!(),
            Statement::ExtractImport(_) => todo!(),
            Statement::Impl { base, items } => todo!(),
            Statement::Overload { op, macros } => todo!(),
            Statement::Dbg(_) => todo!(),
            Statement::Throw(_) => todo!(),
        }
        Ok(())
    }

    pub fn compile(&mut self, ast: &Ast) -> CompileResult<Bytecode> {
        let mut code = ProtoBytecode::new();
        code.new_func(|b| {
            let base_scope = self.scopes.insert(Scope {
                vars: Default::default(),
                parent: None,
                typ: Some(ScopeType::Global),
            });
            for stmt in &ast.statements {
                self.compile_stmt(stmt, base_scope, b)?;
            }
            Ok(())
        })?;
        let code = code.build(&self.src).unwrap();
        // println!("{:#?}", code);

        code.debug_str(&self.src);

        todo!()
    }
}
