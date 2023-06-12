use std::str::FromStr;

use ahash::AHashMap;
use paste::paste;

use super::ast::{Expression, Spanned};
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

#[allow(unused_macros)]
macro_rules! parse_string {
    ($parser:ident) => {
        $parser.resolve(&$parser.parse_plain_string($parser.slice(), $parser.span())?)
    };
}

macro_rules! attributes {
    (
        $( #[check_validity($check:ident)] )?
        pub enum $enum:ident {
            $(
                $(#[valid_on($($item:ident),+ $(,)*)])?
                $variant:ident
                $(
                    (
                        $typ1:ty
                        $(,$typ:ty)*
                    )
                )?
                $(
                    {
                        $field1:ident: $f_typ1:ty, $($field:ident: $f_typ:ty,)*
                    }
                )?,
            )*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $enum {
            $(
                $variant $( ( $typ1 $(,$typ)*) )? $( { $field1: $f_typ1, $($field: $f_typ,)* } )?,
            )*
        }

        impl ParseAttribute for $enum {
            #[allow(redundant_semicolons)]
            fn parse(parser: &mut Parser) -> ParseResult<$enum> {
                parser.expect_tok(Token::Ident)?;

                let attr_name = parser.slice().to_string();
                let attr_name_span = parser.span();

                #[allow(unused_variables)]
                let attr = loop {
                    $(
                        paste!(
                            if attr_name == stringify!([< $variant:snake >]) {
                                $(
                                    let field_names = vec![stringify!($field1) $(,stringify!($field))*];
                                    parser.expect_tok(Token::LParen)?;

                                    let mut field_map = AHashMap::new();

                                    for (i, _) in field_names.iter().enumerate() {

                                        if i != 0 {
                                            parser.expect_tok(Token::Comma).map_err(|_| {
                                                SyntaxError::InvalidAttributeArgCount {
                                                    attribute: stringify!([< $variant:snake >]).into(),
                                                    expected: field_names.len(),

                                                    area: parser.make_area(attr_name_span)
                                                }
                                            })?;
                                        }
                                        parser.expect_tok(Token::Ident)?;

                                        let field = parser.slice().to_string();
                                        let field_span = parser.span();

                                        if !field_names.contains(&&field[..]) {
                                            return Err(SyntaxError::InvalidAttributeField {
                                                field,
                                                area: parser.make_area(field_span),
                                                attribute: attr_name,
                                                fields: field_names.iter().map(|s| s.to_string()).collect()
                                            })
                                        }

                                        if let Some((_, prev_span, _)) = field_map.get(&field) {
                                            return Err(SyntaxError::DuplicateAttributeField {
                                                field,
                                                used_again: parser.make_area(field_span),
                                                first_used: parser.make_area(*prev_span),
                                            })
                                        }

                                        parser.expect_tok(Token::Assign)?;
                                        parser.expect_tok(Token::String)?;

                                        field_map.insert(field, (parse_string!(parser), field_span, parser.span()));
                                    }

                                    parser.skip_tok(Token::Comma);
                                    parser.expect_tok(Token::RParen).map_err(|_| {
                                        SyntaxError::InvalidAttributeArgCount {
                                            attribute: stringify!([< $variant:snake >]).into(),
                                            expected: field_names.len(),

                                            area: parser.make_area(attr_name_span)
                                        }
                                    })?;
                                )?

                                $( let tuple_arg_amount = [stringify!($typ1), $(stringify!($typ)),*].len(); )?

                                    let r = $enum::$variant
                                        $(
                                            (
                                                {
                                                    stringify!($typ1);

                                                    parser.expect_tok(Token::LParen)?;
                                                    parser.expect_tok(Token::String)?;

                                                    parse_string!(parser)
                                                }
                                                $(
                                                    ,{

                                                        stringify!($typ);

                                                        parser.expect_tok(Token::Comma).map_err(|_| {
                                                            SyntaxError::InvalidAttributeArgCount {
                                                                attribute: stringify!([< $variant:snake >]).into(),
                                                                expected: tuple_arg_amount,

                                                                area: parser.make_area(attr_name_span)
                                                            }
                                                        })?;

                                                        parser.expect_tok(Token::String)?;

                                                        parse_string!(parser)
                                                    }
                                                )*
                                            );

                                            parser.skip_tok(Token::Comma);
                                            parser.expect_tok(Token::RParen).map_err(|_| {
                                                SyntaxError::InvalidAttributeArgCount {
                                                    attribute: stringify!([< $variant:snake >]).into(),
                                                    expected: tuple_arg_amount,

                                                    area: parser.make_area(attr_name_span)
                                                }
                                            })?;
                                        )?
                                        $(
                                            {
                                                $field1: {stringify!($f_typ1); field_map[stringify!($field1)].0.clone()},
                                                $(
                                                    $field: {stringify!($f_typ); field_map[stringify!($field)].0.clone()},
                                                )*
                                            };
                                        )?
                                    ;
                                break r;
                            }
                        );
                    )*

                    return Err(SyntaxError::UnknownAttribute {
                        area: parser.make_area(attr_name_span),
                        attribute: attr_name,
                        valid: paste!(vec![$(hyperlink(
                            "https://spu7nix.net/spwn/#/attributes?id=attributes",
                            Some(stringify!([<$variant:snake>]))
                        )),*])
                    })
                };

                #[allow(unreachable_code)]
                Ok(attr)
            }
        }


        attributes!(
            @impl !$($check)?; $enum;
            $(
                $variant: $($($item),+)?;
            )*
        );

    };

    (@impl !; $($rest:tt)*) => {};

    (
        @impl !$check:ident; $enum:ident;
        $(
            $variant:ident: $($item:ident),+;
        )*
    ) => {
        impl IsValidOn<$check> for Vec<Spanned<$enum>> {
            fn is_valid_on(&self, node: &Spanned<$check>, src: SpwnSource) -> ParseResult<()> {
                if !self.is_empty() {
                    let mut map: AHashMap<&str, Vec<String>> = AHashMap::new();

                    paste!(
                        $(
                            $(
                                map.entry(
                                    stringify!($item)
                                )
                                .and_modify( |v| v.push(
                                    stringify!([<$variant:snake>])
                                        .to_string()
                                ))
                                .or_insert(vec![
                                    stringify!([<$variant:snake>]).to_string()
                                ]);
                            )+
                        )+


                        for attr in self {
                            match attr.value {
                                $(
                                    $enum::$variant{..} => match &node.value {
                                        $(
                                            $check::$item {..} => (),
                                        )+
                                        other => return Err(SyntaxError::MismatchedAttribute {
                                            area: src.area(attr.span),
                                            expr_area: src.area(node.span),
                                            attr: stringify!([< $variant:snake >]).into(),
                                            valid: map.remove(Into::<&'static str>::into(other)),
                                        }),
                                    },
                                )+
                            }
                        }
                    );
                }

                Ok(())
            }
        }
    };
}

attributes! {
    #[check_validity(Expression)]
    pub enum ExprAttribute {
        #[valid_on(TriggerFunc)]
        NoOptimize,

        #[valid_on(TriggerFunc, Macro)]
        DebugBytecode,
    }
}

attributes! {
    #[check_validity(Statement)]
    pub enum StmtAttribute {
        #[valid_on(Arrow)]
        Doc(String),

        // #[valid_on(Macro)]
        // Constructor,

        #[valid_on(Let, TypeDef)]
        Deprecated { since: String, note: String, },
    }
}

attributes! {
    pub enum FileAttribute {
        CacheOutput,
        NoStd,
    }
}

/////////////////////////////

pub trait ParseType {
    fn parse(parser: &mut Parser) -> ParseResult<Self>
    where
        Self: Sized;
}

// // TODO: mayb move individual parser functions into here for less code duplication
// impl ParseType for String {
//     fn parse(parser: &mut Parser) -> ParseResult<Self>
//     where
//         Self: Sized,
//     {
//         parser.expect_tok(Token::String);
//         // do string parsing logic here rather than calling the string parsing function?
//         Ok(parser.resolve(&parser.parse_plain_string(parser.slice(), parser.span())?))
//     }
// }
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
macro_rules! attributes2 {
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
        #[allow(unused_variables, unused_mut, dead_code)]
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
                                let mut found = 0;
                                fn parse_t<T: std::str::FromStr>(parser: &mut Parser<'_>, found: &mut usize, span: $crate::CodeSpan, t_name: &'static str) -> ParseResult<T> {
                                    const COUNT: usize = {
                                        0
                                        $(; [stringify!($typ1), $(stringify!($typ)),*].len() )?
                                        $(; [stringify!($field1), $(stringify!($field)),*].len() )?
                                    };

                                    if parser.next_is(Token::RSqBracket) {
                                        return Err(SyntaxError::InvalidAttributeArgCount2 {
                                            attribute: stringify!([< $variant:snake >]).into(),
                                            expected: COUNT,
                                            found: *found,
                                            area: parser.make_area(span)
                                        })
                                    } else {
                                        let v = into_t::<T>(parser, &parse_string!(parser), parser.span(), t_name);
                                        *found += 1;
                                        v
                                    }
                                }

                                $(
                                    stringify!($typ1);
                                    if parser.next_is(Token::LParen) {
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

                                        field_map.insert(name, (parser.span(), parse_string!(parser)));

                                        if i < FIELD_NAMES.len() - 1 {
                                            parser.expect_tok(Token::Comma)?;
                                        }
                                    }

                                    parser.skip_tok(Token::Comma);

                                    if field_map.len() != FIELD_NAMES.len() {
                                        return Err(SyntaxError::InvalidAttributeArgCount2 {
                                            attribute: stringify!([< $variant:snake >]).into(),
                                            expected: FIELD_NAMES.len(),
                                            found: field_map.len(),
                                            area: parser.make_area(attr_name_span)
                                        })
                                    }

                                    parser.expect_tok(Token::RParen)?;
                                )?

                                Ok($enum::$variant $(
                                    (
                                        parse_t::<$typ1>(parser, &mut found, attr_name_span, stringify!($typ1))?,
                                        $(
                                            {
                                                parser.expect_tok(Token::Comma)?;
                                                parser.expect_tok(Token::String)?;
                                                parse_t::<$typ>(parser, &mut found, attr_name_span, stringify!($typ))?
                                            },
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
                                )?)
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

        #[derive(Debug, Clone, PartialEq, Eq)]
        $vis enum $enum {
            $(
                $variant $( ( $typ1 $(,$typ)*) )? $( { $field1: $f_typ1, $($field: $f_typ,)* } )?,
            )*
        }
    };
}

attributes2! {
    pub enum AAA {
        #[valid_on(Expression::Int, Statement::Let)] A,
        #[valid_on(Expression::Int, Statement::Let)] Deprecated { since: String, note: String, },
    }
}
