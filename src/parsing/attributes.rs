use std::ops::{Index, IndexMut};

// #[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumVariantNames, EnumProperty)]
// #[strum(serialize_all = "snake_case")]
// pub enum ExprAttribute {
//     NoOptimize,

//     #[strum(props(args = "2", arg0 = "since", arg1 = "note"))]
//     Deprecated {
//         since: Option<String>,
//         note: String,
//     },
// }

// enum_variant.jjhjh = jjhjh;

// impl<'a> Index<String> for Foo<'a> {
//     type Output = Option<&'a String>;

//     fn index(&self, index: String) -> &Option<&'a String> {
//         match self {
//             Self::NoOptimize => &None,
//             Self::This { ref bar } => match &index[..] {
//                 "bar" => &Some(bar),
//                 _ => &None,
//             },
//             Self::Doc(ref doc) => match &index[..] {
//                 "0" => &Some(doc),
//                 _ => &None,
//             },

//             _ => unreachable!(),
//         }
//     }
// }

// impl IndexMut<String> for Foo {
//     fn index_mut(&mut self, index: String) -> &mut Self::Output {
//         todo!()
//     }
// }

use crate::error::hyperlink;
use crate::lexing::tokens::Token;
use crate::parsing::{
    error::SyntaxError,
    parser::{ParseResult, Parser},
};
use crate::sources::CodeSpan;
use ahash::AHashMap;
use paste::paste;

pub trait AttributeEnum {
    fn attribute_parse(parser: &mut Parser) -> ParseResult<Self>
    where
        Self: std::marker::Sized;
}

macro_rules! parse_string {
    ($parser:ident) => {
        $parser.parse_string($parser.slice(), $parser.span())?
    };
}

macro_rules! attributes {
    (
        $(
            type @$type_name:ident($arg:ident) -> $type_ret:ty $type_code:block;
        )*

        $(
            pub enum $enum:ident {
                $(

                    $variant:ident $( ( @$typ1:ident $(,@$typ:ident)*) )? $( { $field1:ident: @$f_typ1:ident, $($field:ident: @$f_typ:ident,)* } )?,

                )+
            }
        )+
    ) => {

        paste!(
            $(

                type [<TypeRet $type_name:camel>] = $type_ret;

                fn [<func_ $type_name>]($arg: String, span: CodeSpan, parser: &Parser) -> ParseResult<$type_ret> {

                    let code = || {
                        $type_code
                    };

                    code().map_err(|s: &str| SyntaxError::InvalidAttributeValue {
                        area: parser.make_area(span),
                        message: s.into(),
                    })
                }
            )*
        );

        $(
            paste!(
                #[derive(Debug, Clone)]
                pub enum $enum {
                    $(
                        $variant $( ( [<TypeRet $typ1:camel>] $(,[<TypeRet $typ:camel>])*) )? $( { $field1: [<TypeRet $f_typ1:camel>], $($field: [<TypeRet $f_typ:camel>],)* } )?,
                    )+
                }
            );

            impl AttributeEnum for $enum {
                #[allow(redundant_semicolons)]
                fn attribute_parse(parser: &mut Parser) -> ParseResult<$enum> {
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
                                                    parser.expect_tok(Token::LParen)?;
                                                    parser.expect_tok(Token::String)?;

                                                    [<func_ $typ1>](parse_string!(parser), parser.span(), parser)?
                                                }
                                                $(
                                                    ,{
                                                        parser.expect_tok(Token::Comma)?;
                                                        parser.expect_tok(Token::String)?;
                                                        [<func_ $typ>](parse_string!(parser), parser.span(), parser)?
                                                    }
                                                )*
                                            );
                                            parser.skip_tok(Token::Comma);
                                            parser.expect_tok(Token::RParen)?;
                                        )?
                                        $(
                                            {
                                                $field1: [<func_ $f_typ1>](field_map[stringify!($field1)].0.clone(), field_map[stringify!($field1)].2, parser)?,
                                                $(
                                                    $field: [<func_ $f_typ>](field_map[stringify!($field)].0.clone(), field_map[stringify!($field)].2, parser)?,
                                                )*
                                            };
                                        )?
                                    ;
                                );

                                break r;
                            }
                        )+

                        // let attr = T::from_str(attr_name).map_err(|_| SyntaxError::UnknownAttribute {
                        //     attr: attr_name.into(),
                        //     area: self.make_area(self.span()),

                        //     help: format!(
                        //         "The valid attributes are: {}",
                        //         T::VARIANTS
                        //             .iter()
                        //             .map(|v| hyperlink(
                        //                 "https://spu7nix.net/spwn/#/attributes?id=attributes",
                        //                 Some(v)
                        //             ))
                        //             .collect::<Vec<_>>()
                        //             .join(", ")
                        //     ),
                        // })?;

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

        )+

    };
}

attributes! {

    type @string(s) -> String {
        Ok(s)
    };

    type @version(s) -> (u32, u32, u32) {
        let nums = s.split('.').collect::<Vec<_>>();
        if nums.len() != 3 {
            return Err("Value must be in `x.y.z` version format")
        }
        let msg = "Versions must be unsigned integers";
        let x = nums[0].parse::<u32>().map_err(|_| msg)?;
        let y = nums[1].parse::<u32>().map_err(|_| msg)?;
        let z = nums[2].parse::<u32>().map_err(|_| msg)?;

        Ok((x, y, z))
    };

    pub enum ScriptAttribute {
        Fart,
        CockTuple(@version, @string),
        TheStruct {
            fart: @string,
            dog: @string,
        },
    }




    pub enum ExprAttribute {
        Cocker(@string),
        Bitch,
    }
}

// #[poop()]
