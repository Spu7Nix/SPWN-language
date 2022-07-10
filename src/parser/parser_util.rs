use crate::sources::CodeSpan;

use super::ast::ExprKey;

#[derive(PartialEq, Eq, Debug)]
pub enum OpType {
    LeftAssoc,
    RightAssoc,
    Unary,
}

#[macro_export]
macro_rules! operators {
    (
        $(
            $optype:ident <== [$($tok:ident)+],
        )*
    ) => {
        pub mod operators {

            pub const PREC_MAX: usize = 1000000;

            use super::super::lexer::Token;
            use super::OpType;

            pub fn infix_prec(tok: Token) -> usize {
                let mut prec = 0;
                $(
                    match tok {
                        $(
                            Token::$tok => if OpType::$optype != OpType::Unary {return prec},
                        )+
                        _ => (),
                    };
                    prec += 1;
                    format!("{}", prec);
                )*
                PREC_MAX
            }
            pub fn unary_prec(tok: Token) -> usize {
                let mut prec = 0;
                $(
                    match tok {
                        $(
                            Token::$tok => if OpType::$optype == OpType::Unary {return prec},
                        )+
                        _ => (),
                    };
                    prec += 1;
                    format!("{}", prec);
                )*
                PREC_MAX
            }
            pub fn is_unary(tok: Token) -> bool {
                let mut utoks = vec![];
                $(
                    if OpType::$optype == OpType::Unary {
                        $(
                            utoks.push( Token::$tok );
                        )+
                    }
                )*
                return utoks.contains( &tok );
            }
            pub fn prec_amount() -> usize {
                let mut amount = 0;
                $(
                    amount += 1;
                    format!("{:?}", OpType::$optype);
                )*
                amount
            }
            pub fn prec_type(mut prec: usize) -> OpType {
                $(
                    if prec == 0 {
                        return OpType::$optype;
                    }
                    prec -= 1;
                    format!("{}", prec);
                )*
                unreachable!()
            }
        }
    };
}

// epic operator precedence macro
// unary precedence is the difference between for example -3+4 being parsed as (-3)+4 and -3*4 as -(3*4)

operators!(
    // RightAssoc  <==  [ Assign ],
    // RightAssoc  <==  [ PlusEq MinusEq MultEq DivEq ModEq PowEq EuclModEq ],
    // LeftAssoc   <==  [ And Or ],
    // LeftAssoc   <==  [ Pipe ],
    LeftAssoc   <==  [ Is ],
    Unary       <==  [ ExclMark ],
    LeftAssoc   <==  [ Eq NotEq Greater GreaterEq Lesser LesserEq ],
    // LeftAssoc   <==  [ DoubleDot ],
    // Unary       <==  [ DoubleDot ],
    // Unary       <==  [ TripleDot ],
    LeftAssoc   <==  [ Plus Minus ],
    Unary       <==  [ Minus ],
    LeftAssoc   <==  [ Mult Div Mod ],
    RightAssoc  <==  [ Pow ],
    // LeftAssoc   <==  [ As ],
);

pub struct ParsedDictlike {
    pub items: Vec<(String, Option<ExprKey>)>,
    pub item_spans: Vec<CodeSpan>,
}

pub struct ParsedObjlike {
    pub items: Vec<(ExprKey, ExprKey)>,
    pub item_spans: Vec<CodeSpan>,
}

