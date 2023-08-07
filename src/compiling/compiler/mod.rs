pub mod attributes;
pub mod exprs;
pub mod patterns;
pub mod stmts;
pub mod util;

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use ahash::AHashMap;
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::builder::BlockID;
use super::bytecode::UnoptRegister;
use super::error::CompileError;
use crate::cli::{BuildSettings, DocSettings};
use crate::compiling::builder::ProtoBytecode;
use crate::new_id_wrapper;
use crate::parsing::ast::{Ast, PatternNode, Statements, Vis};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource, TypeDefMap};
use crate::util::{ImmutStr, ImmutStr32, ImmutVec, Interner, SlabMap, Str32, String32};

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
    interner: Rc<RefCell<Interner>>,
    scopes: SlabMap<ScopeID, Scope>,
    pub global_return: Option<Spanned<ImmutVec<Spanned<Spur>>>>,

    build_settings: &'a BuildSettings,
    doc_settings: &'a DocSettings,
    is_doc_gen: bool,

    bytecode_map: &'a mut BytecodeMap,
    pub type_def_map: &'a mut TypeDefMap,

    pub local_type_defs: SlabMap<LocalTypeID, Vis<TypeDef<Spur>>>,
    pub available_custom_types: AHashMap<Spur, Vis<CustomTypeID>>,

    deferred_trigger_func_stack: Vec<Vec<DeferredTriggerFunc>>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        src: Rc<SpwnSource>,
        build_settings: &'a BuildSettings,
        doc_settings: &'a DocSettings,
        is_doc_gen: bool,
        bytecode_map: &'a mut BytecodeMap,
        type_def_map: &'a mut TypeDefMap,
        interner: Rc<RefCell<Interner>>,
    ) -> Self {
        Self {
            src,
            interner,
            scopes: SlabMap::new(),
            global_return: None,
            local_type_defs: SlabMap::new(),
            available_custom_types: AHashMap::new(),
            build_settings,
            doc_settings,
            is_doc_gen,
            bytecode_map,
            type_def_map,
            deferred_trigger_func_stack: vec![],
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
        self.interner.borrow_mut().get_or_intern(s)
    }

    pub fn resolve(&self, s: &Spur) -> ImmutStr {
        self.interner.borrow().resolve(s).into()
    }

    pub fn resolve_32(&self, s: &Spur) -> ImmutStr32 {
        String32::from_chars(self.interner.borrow().resolve(s).chars().collect_vec()).into()
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
            |b| {
                let base_scope = self.scopes.insert(Scope {
                    vars: Default::default(),
                    parent: None,
                    typ: Some(ScopeType::Global),
                });

                // if entry.is_some() {
                //     let import_type =
                //         ImportType::Module(BUILTIN_DIR.join("core/lib.spwn"), ModuleImport::Core);

                //     // self.extract_import(&import_type, CodeSpan::internal(), f, base_scope)?;
                // }
                // if let Some(attrs) = entry {
                //     if !attrs.iter().any(|a| *a == FileAttribute::NoStd) {
                //         let import_type =
                //             ImportType::Module(BUILTIN_DIR.join("std/lib.spwn"), ModuleImport::Std);
                //         // self.extract_import(&import_type, CodeSpan::internal(), f, base_scope)?;
                //     }
                // }

                self.compile_stmts(&ast.statements, base_scope, b)?;

                Ok(())
            },
            (Box::new([]), None),
            vec![],
            span,
        )?;
        let code = code.build(&self.src, self).unwrap();

        if !self.build_settings.debug_bytecode && !code.debug_funcs.is_empty() {
            code.debug_str(&self.src, Some(&code.debug_funcs))
        }

        self.bytecode_map.insert((*self.src).clone(), Rc::new(code));

        Ok(())
    }
}
