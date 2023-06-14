use ahash::AHashMap;
use paste::paste;

use super::ast::{DictItem, Expression, Spanned};
use crate::lexing::tokens::Token;
use crate::parsing::ast::Statement;
use crate::parsing::error::SyntaxError;
use crate::parsing::parser::{ParseResult, Parser};
use crate::util::hyperlink;
use crate::SpwnSource;

pub trait ParseAttribute {
    fn parse(parser: &mut Parser<'_>) -> ParseResult<Self>
    where
        Self: Sized;
}

pub trait IsValidOn<T: Into<&'static str>> {
    fn is_valid_on(&self, node: &Spanned<T>, src: SpwnSource) -> ParseResult<()>;
}

fn into_t<T: std::str::FromStr>(
    parser: &mut Parser,
    s: &str,
    span: crate::CodeSpan,
    t_name: &'static str,
) -> ParseResult<T> {
    T::from_str(s).map_err(|_| SyntaxError::InvalidAttributeArgType {
        expected: t_name,
        area: parser.make_area(span),
    })
}

#[rustfmt::skip]
macro_rules! attributes {
    (
        $vis:vis enum
        $enum:ident {
            $(
                $(
                    #[
                        valid_on(
                            $(
                                $(Expression:: $v_expr:ident)?
                                $(Statement:: $v_stmt:ident)?
                            ),*
                        )
                    ]
                )?
                $variant:ident $(($typ1:ty $(, $typ:ty)*))? $({ $field1:ident : $f_typ1:ty, $($field:ident : $f_typ:ty,)* })?,
            )*
        }
    ) => {
        impl IsValidOn<Expression> for Vec<Spanned<$enum>> {
            fn is_valid_on(&self, node: &Spanned<Expression>, src: SpwnSource) -> ParseResult<()> {
                let mut map: AHashMap<&str, Vec<String>> = AHashMap::new();
                paste! {
                    for attr in self {
                        match &attr.value {
                            $(
                                $enum::$variant {..} => match &node.value {
                                    $(
                                        $(
                                            $(Expression::$v_expr {..} => (),)?
                                        )*
                                    )?
                                    other => return Err(SyntaxError::MismatchedAttribute {
                                        area: src.area(attr.span),
                                        expr_area: src.area(node.span),
                                        attr: stringify!([< $variant:snake >]).into(),
                                        valid:  {
                                            $(
                                                $(
                                                    $(
                                                        map.entry(
                                                            stringify!($v_expr)
                                                        )
                                                        .and_modify( |v| v.push(
                                                            stringify!([<$variant:snake>])
                                                                .to_string()
                                                        ))
                                                        .or_insert(vec![
                                                            stringify!([<$variant:snake>]).to_string()
                                                        ]);
                                                    )?
                                                )*
                                            )?
                                            map.remove(Into::<&'static str>::into(other))
                                        },
                                    }),
                                }
                            )*
                        }
                    }
                }

                Ok(())
            }
        }

        impl IsValidOn<Statement> for Vec<Spanned<$enum>> {
            fn is_valid_on(&self, node: &Spanned<Statement>, src: SpwnSource) -> ParseResult<()> {
                let mut map: AHashMap<&str, Vec<String>> = AHashMap::new();

                paste! {
                    for attr in self {
                        match &attr.value {
                            $(
                                $enum::$variant {..} => match &node.value {
                                    $(
                                        $(
                                            $(Statement::$v_stmt {..} => (),)?
                                        )*
                                    )?
                                    other => return Err(SyntaxError::MismatchedAttribute {
                                        area: src.area(attr.span),
                                        expr_area: src.area(node.span),
                                        attr: stringify!([< $variant:snake >]).into(),
                                        valid: {
                                            $(
                                                $(
                                                    $(
                                                        map.entry(
                                                            stringify!($v_stmt)
                                                        )
                                                        .and_modify( |v| v.push(
                                                            stringify!([<$variant:snake>])
                                                                .to_string()
                                                        ))
                                                        .or_insert(vec![
                                                            stringify!([<$variant:snake>]).to_string()
                                                        ]);
                                                    )?
                                                )*
                                            )?
                                            map.remove(Into::<&'static str>::into(other))
                                        },
                                    }),
                                }
                            )*
                        }
                    }
                }

                Ok(())
            }
        }

        #[allow(unused_variables, unused_mut, dead_code, unused_assignments)]
        impl ParseAttribute for $enum {
            fn parse(parser: &mut Parser<'_>) -> ParseResult<Self>
            where
                Self: Sized
            {
                // the #[ are already consumed by the parser
                parser.expect_tok(Token::Ident)?;

                let attr_name = parser.slice().to_string();
                let attr_name_span = parser.span();

                paste! {
                    match &attr_name[..] {
                        $(
                            stringify!([< $variant:snake >]) => {
                                $(
                                    stringify!($typ1);

                                    let mut used_paren = false;

                                    const TOTAL: usize = { [stringify!($typ1), $(stringify!($typ)),*].len() };
                                    let mut found = 0;

                                    if parser.next_is(Token::LParen) {
                                        used_paren = true;
                                        parser.next();
                                    } else if {
                                        #[allow(unreachable_code, unused_labels)]
                                        'v: {
                                            $(break 'v false; stringify!($typ);)*
                                            parser.next_is(Token::Assign)
                                        }
                                    } {
                                        parser.next();
                                    } else {
                                        return Err(SyntaxError::UnexpectedToken {
                                            expected: format!("({}", { " or =" $(; stringify!($typ); "")* }),
                                            found: parser.next(),
                                            area: parser.make_area(parser.span()),
                                        })
                                    }


                                    let mut fields: Vec<($crate::CodeSpan, String)> = Vec::new();

                                    for i in 0..TOTAL {
                                        parser.expect_tok(Token::String)?;
                                        fields.push((parser.span(), parser.resolve(&parser.parse_plain_string(parser.slice(), parser.span())?)));

                                        // cant underflow if the loop nevers runs
                                        if i < TOTAL - 1 {
                                            parser.expect_tok(Token::Comma)?;
                                        }
                                    }

                                    parser.skip_tok(Token::Comma);

                                    if used_paren {
                                        parser.expect_tok(Token::RParen)?;
                                    }
                                )?
                                $(
                                    const FIELD_NAMES: &[&str] = &[stringify!($field1) $(,stringify!($field))*];
                                    let mut field_map: AHashMap<String, ($crate::CodeSpan, String)> = AHashMap::new();

                                    parser.expect_tok(Token::LParen)?;

                                    for i in 0..FIELD_NAMES.len() {
                                        parser.expect_tok(Token::Ident)?;
                                        let name = parser.slice().to_string();

                                        if !FIELD_NAMES.contains(&&name[..]) {
                                            return Err(SyntaxError::InvalidAttributeField {
                                                field: name.to_string(),
                                                area: parser.make_area(parser.span()),
                                                attribute: attr_name,
                                                fields: FIELD_NAMES.iter().map(|s| s.to_string()).collect()
                                            })
                                        }

                                        if let Some((prev_span, _)) = field_map.get(&name) {
                                            return Err(SyntaxError::DuplicateAttributeField {
                                                field: name.to_string(),
                                                used_again: parser.make_area(parser.span()),
                                                first_used: parser.make_area(*prev_span),
                                            })
                                        }

                                        parser.expect_tok(Token::Assign)?;
                                        parser.expect_tok(Token::String)?;

                                        field_map.insert(name, (parser.span(), parser.resolve(&parser.parse_plain_string(parser.slice(), parser.span())?)));

                                        if i < FIELD_NAMES.len() - 1 {
                                            parser.expect_tok(Token::Comma)?;
                                        }
                                    }

                                    parser.skip_tok(Token::Comma);

                                    if field_map.len() != FIELD_NAMES.len() {
                                        return Err(SyntaxError::InvalidAttributeArgCount {
                                            attribute: stringify!([< $variant:snake >]).into(),
                                            expected: FIELD_NAMES.len(),
                                            found: field_map.len(),
                                            area: parser.make_area(attr_name_span)
                                        })
                                    }

                                    parser.expect_tok(Token::RParen)?;
                                )?

                                let mut i = 0;

                                let v = $enum::$variant $(
                                    (
                                        {
                                            let (span, s) = &fields[i];
                                            let t = into_t::<$typ1>(parser, s, *span, stringify!($typ1))?;
                                            i += 1;
                                            t
                                        }
                                        $(
                                            {
                                                let (span, s) = &fields[i];
                                                let t = into_t::<$typ>(parser, s, *span, stringify!($typ))?;
                                                i += 1;
                                                t
                                            }
                                        )*
                                    )
                                )?
                                $(
                                    {
                                        $field1: {
                                            let (span, s) = &field_map[stringify!($field1)];
                                            into_t::<$f_typ1>(parser, s, *span, stringify!($f_typ1))?
                                        },
                                        $(
                                            $field: {
                                                let (span, s) = &field_map[stringify!($field)];
                                                into_t::<$f_typ>(parser, s, *span, stringify!($f_typ))?
                                            },
                                        )*
                                    }
                                )?;

                                parser.skip_tok(Token::RParen);

                                Ok(v)
                            },
                        )*
                        _ => Err(SyntaxError::UnknownAttribute {
                            area: parser.make_area(attr_name_span),
                            attribute: attr_name,
                            valid: paste!(vec![$(hyperlink(
                                "https://spu7nix.net/spwn/#/attributes?id=attributes",
                                Some(stringify!([<$variant:snake>]))
                            )),*])
                        }),
                    }
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, delve::EnumToStr)]
        #[delve(rename_variants = "snakecase")]
        $vis enum $enum {
            $(
                $variant $( ( $typ1 $(,$typ)*) )? $( { $field1: $f_typ1, $($field: $f_typ,)* } )?,
            )*
        }
    };
}

attributes! {
    pub enum FileAttribute {
        CacheOutput,
        NoStd,

        Doc(String),
    }
}

pub type SemVer = semver::Version;

attributes! {
    pub enum Attributes {
        #[valid_on(Expression::TriggerFunc)]
        NoOptimize,

        #[valid_on(Expression::TriggerFunc, Expression::Macro)]
        DebugBytecode,

        #[valid_on(
            Statement::TypeDef,
            Statement::Let,
            Statement::AssignOp,
        )]
        Deprecated { since: SemVer, note: String, },

        #[valid_on(
            Statement::TypeDef,
            Statement::Let,
            Statement::AssignOp,
        )]
        Doc(String),
    }
}

impl IsValidOn<DictItem> for Vec<Spanned<Attributes>> {
    fn is_valid_on(&self, node: &Spanned<DictItem>, src: SpwnSource) -> ParseResult<()> {
        for attr in self {
            match &attr.value {
                Attributes::Deprecated { .. } => (),
                Attributes::Doc(..) => (),
                other => {
                    return Err(SyntaxError::MismatchedAttribute {
                        area: src.area(attr.span),
                        expr_area: src.area(node.span),
                        attr: Into::<&'static str>::into(other).into(),
                        valid: Some(vec!["doc".into(), "deprecated".into()]),
                    });
                },
            }
        }

        Ok(())
    }
}
