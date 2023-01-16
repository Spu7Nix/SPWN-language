use crate::error::hyperlink;
use crate::lexing::tokens::Token;
use crate::parsing::{
    error::SyntaxError,
    parser::{ParseResult, Parser},
};
use ahash::AHashMap;
use paste::paste;

use super::ast::{Expression, Spanned, Statement};

pub trait ParseAttribute {
    fn parse(parser: &mut Parser<'_>) -> ParseResult<Self>
    where
        Self: Sized;
}

pub trait IsValidOn<T> {
    fn assign_if_valid(&self, node: Spanned<T>) -> ParseResult<Self>
    where
        Self: Sized;
}

macro_rules! attributes {
    (
        $(
            $( #[check_validity($check:ident)] )?
            //$(#[$meta:meta])*
            pub enum $enum:ident {
                $(
                    $(#[valid_on($($items:path),* $(,)*)])?
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
                )+
            }
        )+
    ) => {
        $(
            macro_rules! parse_string {
                ($parser:ident) => {
                    $parser.parse_string($parser.slice(), $parser.span())?
                };
            }

            #[derive(Debug, Clone)]
            pub enum $enum {
                $(
                    $variant $( ( $typ1 $(,$typ)*) )? $( { $field1: $f_typ1, $($field: $f_typ,)* } )?,
                )+
            }

            impl ParseAttribute for $enum {
                #[allow(redundant_semicolons)]
                fn parse(parser: &mut Parser) -> ParseResult<$enum> {
                    parser.expect_tok(Token::Ident)?;

                    let attr_name = parser.slice().to_string();
                    let attr_name_span = parser.span();

                    let attr = loop {
                        $(
                            if paste!(attr_name == stringify!([< $variant:snake >])) {
                                // let has_fields = {false $(; stringify!($($typ)*); true)? $(; stringify!($($field)*); true)?};

                                $(
                                    let field_names = vec![stringify!($field1) $(,stringify!($field))*];
                                    parser.expect_tok(Token::LParen)?;

                                    let mut field_map = AHashMap::new();

                                    for (i, _) in field_names.iter().enumerate() {

                                        if i != 0 {
                                            parser.expect_tok(Token::Comma)?;
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
                                    parser.expect_tok(Token::RParen)?;
                                )?

                                paste!(
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

                                                        parser.expect_tok(Token::Comma)?;
                                                        parser.expect_tok(Token::String)?;

                                                        parse_string!(parser)
                                                    }
                                                )*
                                            );

                                            parser.skip_tok(Token::Comma);
                                            parser.expect_tok(Token::RParen)?;
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
                                );

                                break r;
                            }
                        )+

                        return Err(SyntaxError::UnknownAttribute {
                            area: parser.make_area(attr_name_span),
                            attribute: attr_name,
                            valid: paste!(vec![$(hyperlink(
                                "https://spu7nix.net/spwn/#/attributes?id=attributes",
                                Some(stringify!([<$variant:snake>]))
                            )),+])
                        })
                    };

                    Ok(attr)
                }
            }

            $(
                impl IsValidOn<$check> for Vec<$enum> {
                    fn assign_if_valid(&self, node: Spanned<$check>) -> ParseResult<Self> {
                        if !self.is_empty() {

                        }
                        todo!()
                    }
                }
            )?
        )+
    };
}

attributes! {

    pub enum ScriptAttribute {
        Fart,
        CockTuple(String, String),
        TheStruct {
            fart: String,
            dog: String,
        },
    }

    #[check_validity(Expression)]
    pub enum ExprAttribute {
        #[valid_on(TriggerFunction)]
        Cocker(String),
        #[valid_on(TriggerFunction)]
        Bitch,
    }
}

// pub type Attributes<T> = Vec<T>;

// attributes! {
//     #[derive(Debug, Clone)]
//     pub enum ExprAttribute {
//         //#[valid_on(Expression::TriggerFunc)]
//         #[serde]
//         NoOptimize,
//     }
// }

// attributes! {
//     #[derive(Debug, Clone)]
//     pub enum DocAttribute {
//         //#[valid_on(Expression::TriggerFunc, Expression::Method, Expression::Type)]
//         Doc(String),

//         //#[valid_on(Expression::TriggerFunc, Expression::Method, Expression::Type)]
//         Deprecated { since: String, note: String },
//     }
// }

// attributes! {
//     #[derive(Debug, Clone)]
//     pub enum ScriptAttribute {
//         //#[valid_on()]
//         CacheOutput,

//         //#[valid_on()]
//         NoStd,

//         //#[valid_on()]
//         ConsoleOutput,

//         //#[valid_on()]
//         NoLevel,

//         //#[valid_on()]
//         NoBytecodeCache,
//     }
// }
