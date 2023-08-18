use std::rc::Rc;

use lasso::Spur;

use super::attributes::Doc;
use super::error::DocResult;
use super::DocCompiler;
use crate::parsing::ast::Attribute;
use crate::sources::SpwnSource;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Type {
    // if we encounter an impl of a type before the type def for that type, this will be false
    pub encountered_type_def: bool,
    // pub name: Spur,
    pub doc: Option<Vec<Doc>>,
}

impl Type {
    pub fn new(is_defined: bool) -> Type {
        Self {
            doc: None,
            encountered_type_def: is_defined,
        }
    }
}

impl<'a> DocCompiler<'a> {
    pub(crate) fn new_type(
        &self,
        is_defined: bool,
        attributes: &[Attribute],
        src: &Rc<SpwnSource>,
    ) -> DocResult<Type> {
        let mut t = Type::new(is_defined);

        if let Some(doc) = self
            .find_doc_attrs(attributes, src)
            .map_err(|e| e.to_report())?
        {
            t.doc = Some(doc);
        }

        Ok(t)
    }
}
