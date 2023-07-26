use std::rc::Rc;
use std::str::FromStr;

use lasso::Spur;

use super::{CompileResult, Compiler, CustomTypeID, ScopeID, ScopeType};
use crate::compiling::builder::{BlockID, CodeBuilder, JumpType};
use crate::compiling::bytecode::UnoptRegister;
use crate::compiling::error::CompileError;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{DictItem, Import, ImportType, PatternNode, Vis, VisTrait};
use crate::parsing::parser::Parser;
use crate::sources::{CodeSpan, SpwnSource};
use crate::util::{ImmutStr, ImmutVec};

impl Compiler<'_> {
    pub fn is_inside_macro(&self, scope: ScopeID) -> Option<Option<Rc<PatternNode>>> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::MacroBody(p) => return Some(p.clone()),
                ScopeType::Global => return None,
                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.is_inside_macro(k),
            None => None,
        }
    }

    pub fn is_inside_loop(&self, scope: ScopeID) -> Option<BlockID> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(ScopeType::ArrowStmt(_) | ScopeType::TriggerFunc(_)) | None => {
                match scope.parent {
                    Some(k) => self.is_inside_loop(k),
                    None => None,
                }
            },
            Some(t) => match t {
                ScopeType::Loop(v) => Some(*v),
                _ => None,
            },
        }
    }

    pub fn assert_can_return(&self, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
        fn can_return_d(slf: &Compiler, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
            let scope = &slf.scopes[scope];
            match &scope.typ {
                Some(ScopeType::MacroBody(_)) => return Ok(()),
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

    pub fn assert_can_break_loop(&self, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::Loop(_) => return Ok(()),
                ScopeType::TriggerFunc(def) => {
                    return Err(CompileError::BreakInTriggerFuncScope {
                        area: self.make_area(span),
                        def: self.make_area(*def),
                    })
                },
                ScopeType::ArrowStmt(def) => {
                    return Err(CompileError::BreakInArrowStmtScope {
                        area: self.make_area(span),
                        def: self.make_area(*def),
                    })
                },

                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.assert_can_break_loop(k, span),
            None => unreachable!(), // should only be called after is_inside_loop
        }
    }

    pub fn load_type(
        &mut self,
        v: &lasso::Spur,
        reg: UnoptRegister,
        span: CodeSpan,
        builder: &mut CodeBuilder<'_>,
    ) -> Result<(), CompileError> {
        let name = self.resolve(v);

        match ValueType::from_str(&name) {
            Ok(v) => {
                builder.load_const(v, reg, span);
            },
            Err(_) => match self.available_custom_types.get(v) {
                Some(k) => builder.load_const(ValueType::Custom(*k), reg, span),
                None => {
                    return Err(CompileError::NonexistentType {
                        area: self.make_area(span),
                        type_name: name.into(),
                    })
                },
            },
        }
        Ok(())
    }

    pub fn compile_import(
        &mut self,
        import: &Import,
        span: CodeSpan,
        importer_src: Rc<SpwnSource>,
    ) -> CompileResult<(
        ImmutVec<ImmutStr>,
        SpwnSource,
        ImmutVec<(CustomTypeID, Spur)>,
    )> {
        let base_dir = importer_src.path().parent().unwrap();
        let mut path = base_dir.to_path_buf();

        match import.settings.typ {
            ImportType::File => path.push(&import.path),
            ImportType::Library => {
                path.push("libraries");
                path.push(&import.path);
                path.push("lib.spwn");
            },
        };

        let is_file = matches!(import.settings.typ, ImportType::File);

        let new_src = Rc::new(SpwnSource::File(path.clone()));

        let import_name = path.file_name().unwrap().to_str().unwrap();
        let import_base = path.parent().unwrap();
        let spwnc_path = import_base.join(format!(".spwnc/{import_name}.spwnc"));

        let code = match new_src.read() {
            Some(c) => c,
            None => {
                return Err(CompileError::NonexistentImport {
                    is_file,
                    name: import.path.to_str().unwrap().into(),
                    area: self.make_area(span),
                })
            },
        };

        let mut parser: Parser<'_> =
            Parser::new(&code, Rc::clone(&new_src), Rc::clone(&self.interner));

        let ast = parser
            .parse()
            .map_err(|e| CompileError::ImportSyntaxError {
                is_file,
                err: e,
                area: self.make_area(span),
            })?;

        let mut compiler = Compiler::new(
            Rc::clone(&new_src),
            self.build_settings,
            self.doc_settings,
            self.is_doc_gen,
            self.bytecode_map,
            self.type_def_map,
            Rc::clone(&self.interner),
        );

        compiler.compile(&ast, (0..code.len()).into())?;

        let export_names = compiler.bytecode_map[&new_src].export_names.clone();
        let custom_types = compiler.bytecode_map[&new_src]
            .custom_types
            .iter()
            .filter(|(_, v)| v.is_pub())
            .map(|(id, s)| (*id, compiler.intern(&s.value().value)))
            .collect();

        let bytes = bincode::serialize(&*self.bytecode_map[&new_src]).unwrap();

        // dont write bytecode if caching is disabled
        if !self.build_settings.no_bytecode_cache || !self.is_doc_gen {
            let _ = std::fs::create_dir(import_base.join(".spwnc"));
            std::fs::write(spwnc_path, bytes).unwrap();
        }

        Ok((export_names, (*new_src).clone(), custom_types))

        // todo: caching
        // 'from_cache: {
        //     if spwnc_path.is_file() {
        //         let source_bytes = std::fs::read(&spwnc_path).unwrap();

        //         let bytecode: Bytecode<Register> = match bincode::deserialize(&source_bytes) {
        //             Ok(b) => b,
        //             Err(_) => {
        //                 break 'from_cache;
        //             },
        //         };

        //         if bytecode.source_hash == hash.into()
        //             && bytecode.spwn_ver == env!("CARGO_PKG_VERSION")
        //         {
        //             for import in &bytecode.import_paths {
        //                 self.compile_import(&import.value, import.span, import_src.clone())?;
        //             }
        //             for (k, (name, private)) in &bytecode.custom_types {
        //                 self.custom_type_defs.insert(
        //                     TypeDef {
        //                         def_src: import_src.clone(),
        //                         name: self.intern(&name.value),
        //                         private: *private,
        //                     },
        //                     k.spanned(name.span),
        //                 );
        //             }

        //             self.map.map.insert(import_src, bytecode);
        //             return Ok(());
        //         }
        //     }
        // }
    }

    #[allow(clippy::type_complexity)]
    pub fn and_op(
        &mut self,
        elems: &[&dyn Fn(
            &mut Compiler<'_>,
            &mut CodeBuilder<'_>,
        ) -> CompileResult<UnoptRegister>],
        dest: UnoptRegister,
        span: CodeSpan,
        builder: &mut CodeBuilder<'_>,
    ) -> CompileResult<()> {
        builder.new_block(|builder| {
            for elem in elems {
                let r = elem(self, builder)?;
                builder.copy_deep(r, dest, span);
                builder.jump(None, JumpType::EndIfFalse(dest), span);
            }
            Ok(())
        })?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn or_op(
        &mut self,
        elems: &[&dyn Fn(
            &mut Compiler<'_>,
            &mut CodeBuilder<'_>,
        ) -> CompileResult<UnoptRegister>],
        dest: UnoptRegister,
        span: CodeSpan,
        builder: &mut CodeBuilder<'_>,
    ) -> CompileResult<()> {
        builder.new_block(|builder| {
            for elem in elems {
                let r = elem(self, builder)?;
                builder.copy_deep(r, dest, span);
                builder.jump(None, JumpType::EndIfTrue(dest), span);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn compile_return(
        &mut self,
        reg: UnoptRegister,
        pat: Option<&PatternNode>,
        module_ret: bool,
        scope: ScopeID,
        span: CodeSpan,
        builder: &mut CodeBuilder<'_>,
    ) -> CompileResult<()> {
        if let Some(pat) = pat {
            let matches_reg = self.compile_pattern_check(reg, pat, true, scope, builder)?;
            builder.mismatch_throw_if_false(matches_reg, reg, span);
        }

        builder.ret(reg, module_ret, span);

        Ok(())
    }

    pub fn compile_dictlike(
        &mut self,
        items: &Vec<Vis<DictItem>>,
        scope: ScopeID,
        span: CodeSpan,
        builder: &mut CodeBuilder<'_>,
    ) -> CompileResult<UnoptRegister> {
        let out = builder.next_reg();
        builder.alloc_dict(out, items.len() as u16, span);
        for item in items {
            let r = match &item.value().value {
                Some(e) => self.compile_expr(e, scope, builder)?,
                None => match self.get_var(item.value().name.value, scope) {
                    Some(v) => v.reg,
                    None => {
                        return Err(CompileError::NonexistentVariable {
                            area: self.make_area(span),
                            var: self.resolve(&item.value().name),
                        })
                    },
                },
            };
            let k = builder.next_reg();

            let chars = self.resolve_32(&item.value().name.value);
            builder.load_const(chars, k, item.value().name.span);

            builder.insert_dict_elem(r, out, k, span, item.is_priv())
        }
        Ok(out)
    }
}
