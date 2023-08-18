use std::rc::Rc;

use super::DocCompiler;
use crate::compiling::compiler::CompileResult;
use crate::compiling::error::CompileError;
use crate::parsing::ast::{AttrArgs, Attribute, Expression};
use crate::parsing::attributes::attr_names;
use crate::sources::{CodeArea, CodeSpan, SpwnSource};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Doc {
    content: String,
}

impl<'a> DocCompiler<'a> {
    fn make_area(&self, span: CodeSpan, area: &Rc<SpwnSource>) -> CodeArea {
        CodeArea {
            src: Rc::clone(area),
            span,
        }
    }

    pub(crate) fn find_doc_attrs(
        &self,
        attrs: &[Attribute],
        current_src: &Rc<SpwnSource>,
    ) -> CompileResult<Option<Vec<Doc>>> {
        let mut docs = vec![];

        for attr in attrs {
            if &*self.interner.resolve(&attr.item.name) != attr_names::DOC {
                continue;
            }

            match &attr.item.args {
                AttrArgs::Eq(expr) => match &*expr.expr {
                    Expression::String(s) => {
                        let s = s.get_compile_time(&self.interner).ok_or(
                            CompileError::InvalidAttributeString {
                                area: self.make_area(expr.span, current_src),
                            },
                        )?;

                        docs.push(Doc { content: s });
                    },
                    _ => {
                        return Err(CompileError::InvalidAttributeArgType {
                            expected: "string",
                            args_area: self.make_area(expr.span, current_src),
                        })
                    },
                },
                _ => unreachable!(),
            }
        }

        Ok(Some(docs))
    }
}
