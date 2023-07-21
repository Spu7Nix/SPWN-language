use super::{CompileResult, Compiler};
use crate::compiling::error::CompileError;
use crate::parsing::ast::{AttrArgs, Attribute, Expression};
use crate::parsing::attributes::attr_names;
use crate::sources::Spanned;

pub struct Overload {
    pub name: String,
}

impl Compiler<'_> {
    fn find_attr_by_str_name(&self, attrs: &[Attribute], name: &str) -> bool {
        for attr in attrs {
            if self.interner.borrow().resolve(&*attr.item.name) != name {
                continue;
            }

            return true;
        }
        false
    }

    pub fn find_debug_bytecode_attr(&self, attrs: &[Attribute]) -> bool {
        self.find_attr_by_str_name(attrs, attr_names::DEBUG_BYTECODE)
    }

    pub fn find_builtin_attr(&self, attrs: &[Attribute]) -> bool {
        self.find_attr_by_str_name(attrs, attr_names::BUILTIN)
    }

    pub fn find_overload_attr(&self, attrs: &[Attribute]) -> CompileResult<Option<Overload>> {
        for attr in attrs {
            if self.interner.borrow().resolve(&*attr.item.name) != attr_names::OVERLOAD {
                continue;
            }

            match &attr.item.args {
                AttrArgs::Eq(expr) => match &*expr.expr {
                    Expression::Var(s) => {
                        return Ok(Some(Overload {
                            name: self.interner.borrow().resolve(s).into(),
                        }))
                    },
                    _ => {
                        return Err(CompileError::InvalidAttributeArgType {
                            expected: "ident",
                            args_area: self.make_area(expr.span),
                        })
                    },
                },
                _ => unreachable!(),
            }
        }

        Ok(None)
    }
}
