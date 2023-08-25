use ahash::AHashMap;
use itertools::Itertools;

use super::{ParseResult, Parser};
use crate::lexing::tokens::Token;
use crate::list_helper;
use crate::parsing::ast::{AttrArgs, AttrItem, AttrStyle, Attribute, DelimArg};
use crate::parsing::attributes::{
    get_attr_by_name_in_namespace, namespace_exists, AttributeDuplicates, AttributeTarget, ListArg,
};
use crate::parsing::error::SyntaxError;
use crate::sources::{CodeSpan, Spannable, Spanned};

impl Parser<'_> {
    pub fn check_attributes(
        &mut self,
        attrs: &[Attribute],
        target: Option<Spanned<AttributeTarget>>,
    ) -> ParseResult<()> {
        enum Duplicability {
            Warn,
            Err,
        }

        let mut possible_duplicates: AHashMap<&str, Spanned<Duplicability>> = AHashMap::new();

        for raw_attr in attrs {
            let name = self.resolve(&raw_attr.item.name);

            if let Some(dup) = possible_duplicates.get(&*name) {
                match dup.value {
                    Duplicability::Warn => todo!(),
                    Duplicability::Err => {
                        return Err(SyntaxError::DuplicateAttribute {
                            attribute: name.into(),
                            current_area: self.make_area(raw_attr.span),
                            old_area: self.make_area(dup.span),
                        })
                    },
                }
            }

            let namespace_span = raw_attr.item.namespace.map(|n| n.span);
            let namespace = raw_attr.item.namespace.map(|n| self.resolve(&n));

            if let (Some(ns), Some(span)) = (&namespace, namespace_span) {
                if !namespace_exists(ns) {
                    return Err(SyntaxError::UnknownAttributeNamespace {
                        namespace: ns.to_string(),
                        area: self.make_area(span),
                    });
                }
            }

            let attr_template = get_attr_by_name_in_namespace(&namespace, &name).ok_or(
                SyntaxError::UnknownAttribute {
                    attribute: name.clone().into(),
                    area: self.make_area(raw_attr.span),
                },
            )?;

            if raw_attr.style != AttrStyle::Inner {
                let target = target.expect("ERR: target must be provided with outer attr");

                if !attr_template.targets.contains(&target) {
                    return Err(SyntaxError::MismatchedAttributeTarget {
                        attribute: name.into(),
                        target_area: self.make_area(target.span),
                    });
                }
            }

            if !attr_template.style.contains(&raw_attr.style) {
                return Err(SyntaxError::MismatchedAttributeStyle {
                    style: raw_attr.style,
                    area: self.make_area(raw_attr.span),
                });
            }

            match attr_template.duplicates {
                AttributeDuplicates::WarnFollowing => {
                    possible_duplicates.insert(
                        attr_template.name,
                        Duplicability::Warn.spanned(raw_attr.span),
                    );
                },
                AttributeDuplicates::ErrorFollowing => {
                    possible_duplicates.insert(
                        attr_template.name,
                        Duplicability::Err.spanned(raw_attr.span),
                    );
                },
                AttributeDuplicates::DuplicatesOk => (),
            }

            match &raw_attr.item.args {
                AttrArgs::Empty => {
                    if !attr_template.template.word {
                        return Err(SyntaxError::NoArgumentsProvidedToAttribute {
                            attribute: name.into(),
                            attribute_area: self.make_area(raw_attr.item.name.span),
                        });
                    }
                },
                a @ AttrArgs::Delimited(args) => {
                    if let Some(list) = attr_template.template.list {
                        let all_args = list
                            .iter()
                            .map(|a| match a {
                                ListArg::Required(v) | ListArg::Optional(v) => v,
                            })
                            .collect_vec();

                        let mut required_args: AHashMap<&str, ()> = list
                            .iter()
                            .filter_map(|a| match a {
                                ListArg::Required(v) => Some((*v, ())),
                                _ => None,
                            })
                            .collect();
                        let required_args_len = required_args.len();

                        for arg in args {
                            let arg_name = self.resolve(&arg.name);

                            if !all_args.contains(&&&*arg_name) {
                                return Err(SyntaxError::UnknownAttributeArgument {
                                    attribute: name.into(),
                                    attribute_area: self.make_area(raw_attr.span),
                                    arg_area: self.make_area(arg.name.span),
                                });
                            } else {
                                required_args.remove(&*arg_name);
                            }
                        }

                        if !required_args.is_empty() {
                            return Err(SyntaxError::MissingRequiredArgumentsForAttribute {
                                attribute: name.into(),
                                expected: required_args_len,
                                found: required_args.len(),
                                attribute_area: self.make_area(raw_attr.span),
                                args_area: self.make_area(a.delimited_span()),
                                missing: required_args
                                    .iter()
                                    .map(|(k, _)| k.to_string())
                                    .collect_vec(),
                            });
                        }
                    }
                },
                AttrArgs::Eq(e) => {
                    if !attr_template.template.name_value {
                        return Err(SyntaxError::UnexpectedValueForAttribute {
                            attribute: name.into(),
                            attribute_area: self.make_area(raw_attr.span),
                            value_area: self.make_area(e.span),
                        });
                    }
                },
            }
        }

        Ok(())
    }

    pub fn parse_outer_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = vec![];
        loop {
            if self.next_are(&[Token::Hashtag, Token::LSqBracket])? {
                let s = self.span();

                self.next()?;
                self.next()?;

                attrs.push(self.parse_attr_meta(s, AttrStyle::Outer)?);
            } else {
                break;
            }
        }

        Ok(attrs)
    }

    pub fn parse_inner_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = vec![];
        loop {
            if self.next_are(&[Token::Hashtag, Token::ExclMark])? {
                let s = self.span();

                self.next()?;
                self.next()?;
                self.expect_tok(Token::LSqBracket)?;

                attrs.push(self.parse_attr_meta(s, AttrStyle::Inner)?);
            } else {
                break;
            }
        }

        Ok(attrs)
    }

    pub fn parse_attr_meta(
        &mut self,
        start_span: CodeSpan,
        attr_style: AttrStyle,
    ) -> ParseResult<Attribute> {
        let mut found_fields = AHashMap::new();

        let name;
        let mut namespace = None;

        match self.next()? {
            Token::Ident => (),
            tok => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: Token::Ident.to_str().into(),
                    found: tok,
                    area: self.make_area(self.span()),
                })
            },
        }

        if self.peek()? == Token::DoubleColon {
            namespace = Some(self.slice_interned().spanned(self.span()));
            self.next()?;
            self.expect_tok(Token::Ident)?;
            name = self.slice_interned().spanned(self.span());
        } else {
            name = self.slice_interned().spanned(self.span());
        }

        Ok(match self.peek()? {
            Token::RSqBracket => {
                self.next()?;
                Attribute {
                    style: attr_style,
                    item: AttrItem {
                        name,
                        namespace,
                        args: AttrArgs::Empty,
                    },
                    span: start_span.extend(self.span()),
                }
            },
            Token::Assign => {
                self.next()?;

                let expr = self.parse_unit(false)?;

                self.expect_tok(Token::RSqBracket)?;

                Attribute {
                    style: attr_style,
                    item: AttrItem {
                        name,
                        namespace,
                        args: AttrArgs::Eq(expr),
                    },
                    span: start_span.extend(self.span()),
                }
            },
            Token::LParen => {
                self.next()?;

                let mut args = vec![];

                if self.next_are(&[Token::Ident, Token::Assign])? {
                    list_helper!(self, RParen {
                        self.expect_tok(Token::Ident)?;

                        let name = self.slice_interned().spanned(self.span());

                        if let Some(first) = found_fields.get(&name.value) {
                            return Err(SyntaxError::DuplicateAttributeField {
                                used_again: self.make_area(self.span()),
                                field: self.resolve(&name).into(),
                                first_used: self.make_area(*first)
                            });
                        }
                        found_fields.insert(name.value, name.span);

                        self.expect_tok(Token::Assign)?;

                        let expr = self.parse_unit(false)?;

                        args.push(DelimArg { name, expr });
                    });

                    self.expect_tok(Token::RSqBracket)?;

                    Attribute {
                        style: attr_style,
                        item: AttrItem {
                            name,
                            namespace,
                            args: AttrArgs::Delimited(args),
                        },
                        span: start_span.extend(self.span()),
                    }
                } else {
                    let expr = self.parse_unit(false)?;

                    self.expect_tok(Token::RParen)?;
                    self.expect_tok(Token::RSqBracket)?;

                    Attribute {
                        style: attr_style,
                        item: AttrItem {
                            name,
                            namespace,
                            args: AttrArgs::Eq(expr),
                        },
                        span: start_span.extend(self.span()),
                    }
                }
            },
            tok => {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "`(`, `=` or `]`".into(),
                    found: tok,
                    area: self.make_area(self.span()),
                });
            },
        })
    }
}
