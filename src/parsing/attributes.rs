use std::ops::{Index, IndexMut};

macro_rules! attribute_enum {
    () => {};
}

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

enum Foo<'a> {
    NoOptimize,

    This { bar: String },

    Doc(String),

    _Phantom(std::marker::PhantomData<&'a ()>),
}

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

use crate::lexing::tokens::Token;
use crate::parsing::parser::{ParseResult, Parser};
use paste::paste;

macro_rules! attributes {
    (
        $(
            pub enum $enum:ident {
                $(

                    $variant:ident $( ( $typ1:ty $($typ:ty),+ ) )? $( { $($field:ident: $field_typ:ty,)+ } )?,

                )+
            }
        )+
    ) => {

        $(
            pub enum $enum {
                $(
                    $variant $( ($($typ,)*) )? $( {$($field: $field_typ,)*} )?,
                )+
            }

            impl $enum {
                pub fn attribute_parse(parser: &mut Parser) -> ParseResult<$enum> {
                    parser.expect_tok(Token::Ident)?;
                    let attr_name = parser.slice();

                    let attr = loop {
                        $(
                            if paste!(attr_name == [< $name:snake>]) {
                                // let has_fields = {false $(; stringify!($($typ)*); true)? $(; stringify!($($field)*); true)?};

                                $(
                                    parser.expect_tok(Token::LParen)?;



                                )?
                            }
                        )+
                    };

                }
            }

        )+

    };
}

attributes! {
    pub enum Cock {
        TheUnit,
        TheTuple(usize, String),
        TheStruct {
            fart: u16,
            dog: String,
        },
    }
}

// #[poop()]
