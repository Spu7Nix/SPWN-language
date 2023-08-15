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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Deprecated {
    pub reason: String,
    pub since: Version,
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

    pub fn find_deprecated_attr(
        &self,
        attrs: &[Attribute],
    ) -> CompileResult<Option<Spanned<Deprecated>>> {
        for attr in attrs {
            if &*self.interner.resolve(&attr.item.name) != attr_names::DEPRECATED {
                continue;
            }

            match &attr.item.args {
                AttrArgs::Delimited(args) => {
                    let reason = args
                        .iter()
                        .find(|a| &*self.interner.resolve(&a.name) == "reason")
                        .unwrap();

                    let reason = match &*reason.expr.expr {
                        Expression::String(s) => s.get_compile_time(&self.interner).ok_or(
                            CompileError::InvalidAttributeString {
                                area: self.make_area(reason.expr.span),
                            },
                        )?,
                        _ => {
                            return Err(CompileError::InvalidAttributeArgType {
                                expected: "string",
                                args_area: self.make_area(reason.expr.span),
                            })
                        },
                    };

                    let since = args
                        .iter()
                        .find(|a| &*self.interner.resolve(&a.name) == "since")
                        .unwrap();

                    let since = Version::parse(&match &*since.expr.expr {
                        Expression::String(s) => s.get_compile_time(&self.interner).ok_or(
                            CompileError::InvalidAttributeString {
                                area: self.make_area(since.expr.span),
                            },
                        )?,
                        _ => {
                            return Err(CompileError::InvalidAttributeArgType {
                                expected: "string",
                                args_area: self.make_area(since.expr.span),
                            })
                        },
                    })
                    .map_err(|_| CompileError::InvalidAttributeArgType {
                        expected: "valid semver version",
                        args_area: self.make_area(since.expr.span),
                    })?;

                    return Ok(Some(Deprecated { reason, since }.spanned(attr.span)));
                },
                _ => unreachable!(),
            }
        }

        Ok(None)
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
