use ahash::AHashMap;

use super::attributes::Doc;
use super::impls::Impl;
use super::types::Type;
use super::DocCompiler;
use crate::parsing::ast::{Ast, Statement, VisTrait};

#[derive(Default)]
pub struct File {
    pub file_doc: Option<Vec<Doc>>,
}

impl File {
    pub fn new(file_doc: Option<Vec<Doc>>) -> Self {
        Self { file_doc }
    }
}

impl<'a> DocCompiler<'a> {
    // does the file export any variables or have any implicit type exports?
    pub(crate) fn file_has_exports(&self, ast: &Ast) -> bool {
        // if the last statement is a return (with a value) there is exports (more statements can't come after a module return)
        if let Some(Statement::Return(Some(..))) = ast.statements.last().map(|f| &*f.stmt) {
            return true;
        }

        for stmt in &ast.statements {
            match &*stmt.stmt {
                Statement::TypeDef { name, .. } if name.is_pub() => return true,
                _ => (),
            }
        }

        false
    }
}
