use semver::Version;
use serde::{Deserialize, Serialize};

use super::{CompileResult, Compiler};
use crate::compiling::error::CompileError;
use crate::parsing::ast::{AttrArgs, Attribute, Expression};
use crate::parsing::attributes::attr_names;
use crate::sources::{Spannable, Spanned};

pub struct Alias {
    pub name: String,
}

impl Compiler<'_> {
    fn find_attr_by_str_name(&self, attrs: &[Attribute], name: &str) -> bool {
        for attr in attrs {
            if &*self.interner.resolve(&attr.item.name) != name {
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

    pub fn find_no_std_attr(&self, attrs: &[Attribute]) -> bool {
        self.find_attr_by_str_name(attrs, attr_names::NO_STD)
    }

    pub fn find_alias_attr(&self, attrs: &[Attribute]) -> CompileResult<Option<Spanned<Alias>>> {
        for attr in attrs {
            if &*self.interner.resolve(&attr.item.name) != attr_names::ALIAS {
                continue;
            }

            match &attr.item.args {
                AttrArgs::Eq(expr) => match &*expr.expr {
                    Expression::Var(s) => {
                        return Ok(Some(
                            Alias {
                                name: self.interner.resolve(s).into(),
                            }
                            .spanned(attr.span),
                        ))
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
