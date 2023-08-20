// extra md syntax:
// ```spwn -> runnable example
// ``` -> non-runnable example
// !```spwn -> runnable example that errors
// !``` -> non-runnable example that errors
//
// [`<type name>`] -> auto link to <type name> docs
// [`path::to::type`] -> link to type
//
// > [!NOTE]
// > ....
//
// > [!WARNING]
// > ....
//
// > [!ERROR]
// > ....
//
// > [!IMPORTANT]
// > ....
//

mod attributes;
pub mod error;
mod file;
mod impls;
mod types;

use std::path::PathBuf;
use std::rc::Rc;

use ahash::AHashMap;
use lasso::Spur;

use self::attributes::Doc;
use self::error::{DocError, DocResult};
use self::file::File;
use self::impls::Impl;
use self::types::Type;
use crate::cli::DocSettings;
use crate::parsing::ast::{Ast, ExprNode, Expression, Statement, StmtNode, VisTrait};
use crate::parsing::parser::Parser;
use crate::sources::SpwnSource;
use crate::util::interner::Interner;
use crate::util::{ImmutStr, SlabMap};

pub struct DocCompiler<'a> {
    settings: &'a DocSettings,
    interner: Interner,
    root: PathBuf,

    files: AHashMap<Rc<SpwnSource>, File>,
    types: AHashMap<(Rc<SpwnSource>, Spur), Type>,
    impls: AHashMap<Spur, Impl>,
}

impl<'a> DocCompiler<'a> {
    pub fn new(settings: &'a DocSettings, root: PathBuf) -> Self {
        Self {
            settings,
            interner: Interner::new(),
            root,

            types: AHashMap::new(),
            impls: AHashMap::new(),
            files: AHashMap::new(),
        }
    }

    // fn intern(&self, s: &str) -> Spur {
    //     self.interner.borrow_mut().get_or_intern(s)
    // }

    // pub fn resolve(&self, s: &Spur) -> ImmutStr {
    //     self.Interner.resolve(s).into()
    // }

    // pub fn resolve_32(&self, s: &Spur) -> ImmutStr32 {
    //     String32::from_chars(self.Interner.resolve(s).chars().collect_vec()).into()
    // }

    fn compile_stmt(&mut self, stmt: &StmtNode, src: &Rc<SpwnSource>) -> DocResult<()> {
        match &*stmt.stmt {
            Statement::TypeDef { name, .. } if name.is_pub() => {
                let typ = self.new_type(true, &stmt.attributes, src)?;

                self.types.insert((Rc::clone(src), *name.value()), typ);

                // todo: check if we have members for static types maybe
            },

            Statement::Assign(..) => todo!(),
            Statement::Impl { name, items } => {
                let spur = name.value;

                let t = self.new_type(false, &stmt.attributes, src)?;
                // leave the type as-is if it was already defined, otherwise insert it as undefined
                self.types.entry((Rc::clone(src), spur)).or_insert(t);

                // self.impls.entry(spur).and_modify(f)
            },
            _ => (),
        }

        Ok(())
    }

    fn compile_ast(&mut self, ast: &Ast, src: Rc<SpwnSource>) -> DocResult<()> {
        for stmt in &ast.statements {
            self.compile_stmt(stmt, &src)?;
        }

        Ok(())
    }

    fn compile_file(&mut self, file: PathBuf) -> DocResult<()> {
        let src = Rc::new(SpwnSource::File(file));
        let code = src.read().ok_or(DocError::FailedToReadSpwnFile)?;

        let mut parser = Parser::new(&code, Rc::clone(&src), self.interner.clone());

        let ast = parser.parse().map_err(|e| e.to_report())?;

        if self.file_has_exports(&ast) {
            self.files.insert(
                Rc::clone(&src),
                File::new(
                    self.find_doc_attrs(&ast.file_attributes, &src)
                        .map_err(|e| e.to_report())?,
                ),
            );
        }

        self.compile_ast(&ast, src)
    }

    fn traverse_library(&mut self) -> DocResult<()> {
        Ok(())
    }

    pub fn compile(&mut self) -> DocResult<()> {
        if let Some(t) = &self.settings.target_dir {
            std::env::set_current_dir(t)?;
        }

        if self.settings.lib.is_some() {
            self.traverse_library()?;
        } else {
            self.compile_file(self.root.clone())?;
        }

        Ok(())
    }
}
