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

pub trait ParseFromRaw<T> {
    fn parse_from_raw(parser: &mut Parser<'_>, on: &T, attr: RawAttribute) -> ParseResult<Self>
    where
        Self: Sized;
}

pub trait ValidOn<T> {
    fn is_valid(&self, attr: &T) -> bool;
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
        // impl ValidOn<$enum> for Expression {
        //     fn is_valid(&self, attr: &$enum) -> bool {
        //         match attr {
        //             $(
        //                 $enum::$variant {..} => {
        //                     match self {
        //                         $(
        //                             $(
        //                                 $(Self::$v_expr { .. } => true,)?
        //                             )*
        //                         )?
        //                         _ => false
        //                     }
        //                 },
        //             )*
        //         }
        //     }
        // }

        // impl ValidOn<$enum> for Statement {
        //     fn is_valid(&self, attr: &$enum) -> bool {
                // match attr {
                //     $(
                //         $enum::$variant {..} => {
                //             match self {
                //                 $(
                //                     $(
                //                         $(Self::$v_stmt { .. } => true,)?
                //                     )*
                //                 )?
                //                 _ => false
                //             }
                //         },
                //     )*
                // }
        //     }
        // }

        impl ParseFromRaw<Expression> for $enum {
            fn parse_from_raw(parser: &mut Parser<'_>, on: &Expression, attr: RawAttribute) -> ParseResult<Self>
            where
                Self: Sized 
            {
                // match attr {
                //     $(
                //         $enum::$variant {..} => {
                //             match self {
                //                 $(
                //                     $(
                //                         $(Self::$v_stmt { .. } => true,)?
                //                     )*
                //                 )?
                //                 _ => false
                //             }
                //         },
                //     )*
                // }

                todo!()
            }
        }

        impl ParseFromRaw<Statement> for $enum {
            fn parse_from_raw(parser: &mut Parser<'_>, on: &Statement, attr: RawAttribute) -> ParseResult<Self>
            where
                Self: Sized 
            {
                todo!()
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        $vis enum $enum {
            $(
                $variant $( ( $typ1 $(,$typ)*) )? $( { $field1: $f_typ1, $($field: $f_typ,)* } )?,
            )*
        }

        impl ParseAttribute for $enum {
            fn parse(parser: &mut Parser) -> ParseResult<$enum> {
                todo!()
            }
        }
    };
}

pub struct RawAttribute {
    tokens: Vec<Token>,
}

attributes2! {
    pub enum AAA {
        #[valid_on(Expression::Int, Statement::Let)] Doc(String),
    }
}

/*

attribute:
name,
arguments -> 1 arg allow #[x(...)] and #[x = ...] syntax


#[doc = "aaa"]
#[doc("aaaa")]

#[no_optimize]

attributes! {
    pub enum StmtAttribute {
        #[valid_on(Expression::Arrow)] Doc(String),

        #[valid_on(Let, TypeDef)]
        Deprecated { since: String, note: String, },
    }
}

#[doc("aaa")] a: 10
a: #[doc("aaa")] 10
 */
