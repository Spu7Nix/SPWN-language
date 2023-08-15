pub mod attributes;
pub mod exprs;
pub mod patterns;
pub mod stmts;
pub mod util;

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::rc::Rc;

use ahash::AHashMap;
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::builder::BlockID;
use super::bytecode::{CallExpr, OptBytecode, OptFunction, UnoptRegister};
use super::deprecated::DeprecatedFeatures;
use super::error::CompileError;
use super::optimizer::optimize_code;
use crate::cli::{BuildSettings, DocSettings};
use crate::compiling::builder::ProtoBytecode;
use crate::new_id_wrapper;
use crate::parsing::ast::{Ast, Import, ImportType, PatternNode, Statements, Vis};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource, TypeDefMap, ZEROSPAN};
use crate::util::interner::Interner;
use crate::util::{ImmutStr, ImmutStr32, ImmutVec, SlabMap, Str32, String32, BUILTIN_DIR};

pub type CompileResult<T> = Result<T, CompileError>;

new_id_wrapper! {
    ScopeID: u16;
    LocalTypeID: u16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomTypeID {
    pub local: LocalTypeID,
    pub source_hash: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct VarData {
    mutable: bool,
    def_span: CodeSpan,
    reg: UnoptRegister,
}

#[derive(Clone, Debug)]
pub enum ScopeType {
    Global,
    Loop(BlockID),
    MacroBody(Option<Rc<PatternNode>>), // return pattern
    TriggerFunc(CodeSpan),
    ArrowStmt(CodeSpan),
}

#[derive(Debug, Clone)]
pub struct Scope {
    vars: AHashMap<Spur, VarData>,
    parent: Option<ScopeID>,
    typ: Option<ScopeType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDef<S> {
    pub src: Rc<SpwnSource>,
    pub def_span: CodeSpan,
    pub name: S,
}

pub struct DeferredTriggerFunc {
    pub stmts: Statements,
    pub group_reg: UnoptRegister,
    pub fn_reg: UnoptRegister,
    pub span: CodeSpan,
}

pub struct Compiler<'a> {
    src: Rc<SpwnSource>,
    interner: Interner,
    scopes: SlabMap<ScopeID, Scope>,
    pub global_return: Option<Spanned<ImmutVec<Spanned<Spur>>>>,

    settings: &'a BuildSettings,

    bytecode_map: &'a mut BytecodeMap,
    pub type_def_map: &'a mut TypeDefMap,

    pub local_type_defs: SlabMap<LocalTypeID, Vis<TypeDef<Spur>>>,
    pub available_custom_types: AHashMap<Spur, Vis<CustomTypeID>>,

    deferred_trigger_func_stack: Vec<Vec<DeferredTriggerFunc>>,

    pub deprecated_features: DeprecatedFeatures,
}

impl<'a> Compiler<'a> {
    pub fn new(
        src: Rc<SpwnSource>,
        settings: &'a BuildSettings,
        bytecode_map: &'a mut BytecodeMap,
        type_def_map: &'a mut TypeDefMap,
        interner: Interner,
    ) -> Self {
        Self {
            src,
            interner,
            scopes: SlabMap::new(),
            global_return: None,
            local_type_defs: SlabMap::new(),
            available_custom_types: AHashMap::new(),
            settings,
            bytecode_map,
            type_def_map,
            deferred_trigger_func_stack: vec![],
            deprecated_features: DeprecatedFeatures::default(),
        }
    }
}

impl Compiler<'_> {
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&self.src),
        }
    }

    fn intern(&self, s: &str) -> Spur {
        self.interner.get_or_intern(s)
    }

    pub fn resolve(&self, s: &Spur) -> ImmutStr {
        self.interner.resolve(s)
    }

    pub fn resolve_32(&self, s: &Spur) -> ImmutStr32 {
        self.interner.resolve_32(s)
    }

    pub fn src_hash(&self) -> u32 {
        let mut hasher = DefaultHasher::default();
        self.src.hash(&mut hasher);
        let h = hasher.finish();
        (h % (u32::MAX as u64)) as u32
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

    pub fn get_accessible_vars(
        &self,
        scope: ScopeID,
    ) -> Box<dyn Iterator<Item = (Spur, VarData)> + '_> {
        let iter = self.scopes[scope].vars.iter().map(|(s, d)| (*s, *d));
        if let Some(p) = self.scopes[scope].parent {
            Box::new(iter.chain(self.get_accessible_vars(p)))
        } else {
            Box::new(iter)
        }
    }

    pub fn compile(&mut self, ast: &Ast, span: CodeSpan) -> CompileResult<()> {
        let mut code = ProtoBytecode::new();
        code.new_func(
            |builder| {
                let base_scope = self.scopes.insert(Scope {
                    vars: Default::default(),
                    parent: None,
                    typ: Some(ScopeType::Global),
                });

                // if !matches!(&*self.src, SpwnSource::Core(..) | SpwnSource::Std(..)) {
                //     let import_reg = builder.next_reg();
                //     // println!("fgfgdfgdfgd");
                //     let (names, s, types) = self.compile_import(
                //         &Import {
                //             typ: ImportType::File,
                //             path: BUILTIN_DIR.join("core/lib.spwn"),
                //         },
                //         span,
                //         Rc::clone(&self.src),
                //         SpwnSource::Core,
                //     )?;
                //     builder.import(import_reg, s, ZEROSPAN);
                //     self.extract_import(&names, &types, base_scope, import_reg, builder, ZEROSPAN);

                //     if !self.find_no_std_attr(&ast.file_attributes) {
                //         let (names, s, types) = self.compile_import(
                //             &Import {
                //                 typ: ImportType::File,
                //                 path: BUILTIN_DIR.join("std/lib.spwn"),
                //             },
                //             span,
                //             Rc::clone(&self.src),
                //             SpwnSource::Std,
                //         )?;
                //         builder.import(import_reg, s, ZEROSPAN);
                //         self.extract_import(
                //             &names, &types, base_scope, import_reg, builder, ZEROSPAN,
                //         );
                //     }
                // }

                self.compile_stmts(&ast.statements, base_scope, builder)?;

                Ok(())
            },
            (Box::new([]), None),
            vec![],
            span,
        )?;
        let mut unopt_code = code.build(&self.src, self).unwrap();
        // unopt_code.debug_str(&self.src, None);

        // let mut s = String::new();

        // s += &format!("{}\n", self.src.name());
        // let mut v = vec![];
        // for func in &unopt_code.functions {
        //     v.push(func.regs_used)
        // }
        if !self.settings.no_optimize_bytecode {
            optimize_code(&mut unopt_code);
        }

        // for (idx, func) in unopt_code.functions.iter().enumerate() {
        //     s += &format!("regs: {} -> {}\n", v[idx], func.regs_used)
        // }
        // s += &format!("------------------------\n\n");

        // let mut data_file = OpenOptions::new()
        //     .append(true)
        //     .open("dog.txt")
        //     .expect("cannot open file");

        // data_file.write(s.as_bytes()).expect("write failed");

        let opt_code = OptBytecode {
            source_hash: unopt_code.source_hash,
            version: unopt_code.version,
            constants: unopt_code.constants,
            functions: unopt_code
                .functions
                .iter()
                .map(|f| {
                    let opt_func = OptFunction {
                        regs_used: f.regs_used.try_into().unwrap(),
                        opcodes: f
                            .opcodes
                            .iter()
                            .map(|&v| v.map(|o| o.try_into().unwrap()))
                            .collect_vec()
                            .into(),
                        span: f.span,
                        args: f.args.clone(),
                        spread_arg: f.spread_arg,
                        captured_regs: f
                            .captured_regs
                            .iter()
                            .map(|&(a, b)| (a.try_into().unwrap(), b.try_into().unwrap()))
                            .collect_vec(),
                        child_funcs: f.child_funcs.clone(),
                    };
                    opt_func
                })
                .collect_vec(),
            custom_types: unopt_code.custom_types,
            export_names: unopt_code.export_names,
            import_paths: unopt_code.import_paths,
            debug_funcs: unopt_code.debug_funcs,
            call_exprs: unopt_code
                .call_exprs
                .iter()
                .map(|v| CallExpr {
                    dest: v.dest.map(|v| v.try_into().unwrap()),
                    positional: v
                        .positional
                        .iter()
                        .map(|&(r, b)| (r.try_into().unwrap(), b))
                        .collect_vec()
                        .into(),
                    named: v
                        .named
                        .iter()
                        .map(|(s, r, b)| (s.clone(), (*r).try_into().unwrap(), *b))
                        .collect_vec()
                        .into(),
                })
                .collect_vec(),
            deprecated_features: self.deprecated_features.clone(),
        };

        if !self.settings.debug_bytecode && !opt_code.debug_funcs.is_empty() {
            opt_code.debug_str(&self.src, Some(&opt_code.debug_funcs))
        }

        self.bytecode_map
            .insert((*self.src).clone(), Rc::new(opt_code));

        Ok(())
    }
}
