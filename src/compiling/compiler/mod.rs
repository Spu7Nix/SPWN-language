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
use crate::cli::BuildSettings;
use crate::compiling::builder::ProtoBytecode;
use crate::new_id_wrapper;
use crate::parsing::ast::{Ast, Vis};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource};
use crate::util::{ImmutStr, ImmutVec, Interner, SlabMap};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignType {
    Normal { is_let: bool },
    Match,
}

impl AssignType {
    fn is_declare(&self) -> bool {
        matches!(
            self,
            AssignType::Match | AssignType::Normal { is_let: true }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub def_span: CodeSpan,
    pub name: Spur,
}

pub struct Compiler<'a> {
    src: Rc<SpwnSource>,
    interner: Rc<RefCell<Interner>>,
    scopes: SlabMap<ScopeID, Scope>,
    pub global_return: Option<Spanned<ImmutVec<Spanned<Spur>>>>,

    settings: &'a BuildSettings,
    bytecode_map: &'a mut BytecodeMap,

    pub custom_type_defs: SlabMap<LocalTypeID, Vis<TypeDef>>,
    available_custom_types: AHashMap<Spur, CustomTypeID>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        src: Rc<SpwnSource>,
        settings: &'a BuildSettings,
        bytecode_map: &'a mut BytecodeMap,
        interner: Rc<RefCell<Interner>>,
    ) -> Self {
        Self {
            src,
            interner,
            scopes: SlabMap::new(),
            global_return: None,
            custom_type_defs: SlabMap::new(),
            available_custom_types: AHashMap::new(),
            settings,
            bytecode_map,
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

    fn resolve_arr(&self, s: &Spur) -> ImmutVec<char> {
        self.interner
            .borrow()
            .resolve(s)
            .chars()
            .collect_vec()
            .into()
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

    pub fn compile(&mut self, ast: &Ast, span: CodeSpan) -> CompileResult<()> {
        let mut code = ProtoBytecode::new();
        code.new_func(
            |b| {
                let base_scope = self.scopes.insert(Scope {
                    vars: Default::default(),
                    parent: None,
                    typ: Some(ScopeType::Global),
                });
                for stmt in &ast.statements {
                    self.compile_stmt(stmt, base_scope, b)?;
                }
                Ok(())
            },
            span,
        )?;
        let code = code.build(&self.src, self).unwrap();

        self.bytecode_map.insert((*self.src).clone(), code);

        Ok(())
    }
}
