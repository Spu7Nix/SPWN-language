use crate::{
    parser::{lexer::Token, parser::parse_expr},
    sources::CodeArea,
};

use super::{
    error::SyntaxError,
    parser::{ASTData, ExprKey, ParseData},
};

#[macro_export]
macro_rules! parse_util {
    ($parse_data:expr, $ast_data:expr, $pos:expr) => {
        #[allow(unused_macros)]

        // returns an "Expected {}, found {} {}" syntax error
        macro_rules! expected_err {
            ($exp:expr, $tok:expr, $area:expr) => {
                return Err(SyntaxError::Expected {
                    expected: $exp.to_string(),
                    typ: $tok.tok_typ().to_string(),
                    found: $tok.tok_name().to_string(),
                    area: CodeArea {
                        source: $parse_data.source.clone(),
                        span: $area,
                    },
                })
            };
        }
        // gets a token (index 0 means current, index 1 the next one, its all relative)
        #[allow(unused_macros)]
        macro_rules! tok {
            ($index:expr) => {
                &$parse_data.tokens[{
                    let le_index = (($pos as i32) + $index);
                    if le_index < 0 {
                        0
                    } else {
                        le_index
                    }
                } as usize]
                    .0
            };
        }
        // gets a token span
        #[allow(unused_macros)]
        macro_rules! span {
            ($index:expr) => {
                $parse_data.tokens[{
                    let le_index = (($pos as i32) + $index);
                    if le_index < 0 {
                        0
                    } else {
                        le_index
                    }
                } as usize]
                    .1
            };
        }
        // gets a token span and turns it into a CodeArea automatically
        #[allow(unused_macros)]
        macro_rules! span_ar {
            ($index:expr) => {
                CodeArea {
                    source: $parse_data.source.clone(),
                    span: span!($index),
                }
            };
        }
        // #[allow(unused_macros)]
        // macro_rules! ret {
        //     ($node_type:expr => $span:expr) => {
        //         return Ok((ASTNode {
        //             node: $node_type,
        //             span: $span,
        //          }, $pos))
        //     };
        //     ($node_type:expr => $start:expr, $end:expr) => {
        //         return Ok((ASTNode {
        //             node: $node_type,
        //             span: ($start, $end),
        //         }, $pos))
        //     };
        // }

        // checks if the current token is something, other returns an `expected` error
        // if it matches it moves forwards
        // can also destructure in case of stuff like Ident token
        #[allow(unused_macros)]
        macro_rules! check_tok {
            ($token:ident else $expected:literal) => {
                if !matches!(tok!(0), Token::$token) {
                    expected_err!($expected, tok!(0), span!(0))
                }
                $pos += 1;
            };
            ($token:ident($val:ident) else $expected:literal) => {
                let $val;
                if let Token::$token(v) = tok!(0) {
                    $val = v.clone();
                } else {
                    expected_err!($expected, tok!(0), span!(0))
                }
                $pos += 1;
            };
            ($token:ident($val:ident):$sp:ident else $expected:literal) => {
                let $val;
                let $sp;
                if let (Token::$token(v), sp) = (tok!(0), span!(0)) {
                    $val = v.clone();
                    $sp = sp.clone();
                } else {
                    expected_err!($expected, tok!(0), span!(0))
                }
                $pos += 1;
            };
        }
        // same thing as before but if it matches it doesnt go forwards
        #[allow(unused_macros)]
        macro_rules! check_tok_static {
            ($token:ident else $expected:literal) => {
                if !matches!(tok!(0), Token::$token) {
                    expected_err!($expected, tok!(0), span!(0))
                }
            };
            ($token:ident($val:ident) else $expected:literal) => {
                let $val;
                if let Token::$token(v) = tok!(0) {
                    $val = v.clone();
                } else {
                    expected_err!($expected, tok!(0), span!(0))
                }
            };
            ($token:ident($val:ident):$sp:ident else $expected:literal) => {
                let $val;
                let $sp;
                if let (Token::$token(v), sp) = (tok!(0), span!(0)) {
                    $val = v.clone();
                    $sp = sp.clone();
                } else {
                    expected_err!($expected, tok!(0), span!(0))
                }
            };
        }

        // skips one token if it matches
        #[allow(unused_macros)]
        macro_rules! skip_tok {
            ($token:ident) => {
                if matches!(tok!(0), Token::$token) {
                    $pos += 1;
                }
            };
        }
        // skips all tokens that match
        #[allow(unused_macros)]
        macro_rules! skip_toks {
            ($token:ident) => {
                while matches!(tok!(0), Token::$token) {
                    $pos += 1;
                }
            };
        }
        // executes the code while the current token matches or doesnt match
        #[allow(unused_macros)]
        macro_rules! while_tok {
            (== $token:ident: $code:block) => {
                loop {
                    match tok!(0) {
                        Token::$token => $code,
                        _ => break,
                    }
                }
            };
            (!= $token:ident: $code:block) => {
                loop {
                    match tok!(0) {
                        Token::$token => break,
                        _ => $code,
                    }
                }
                $pos += 1;
            };
        }
        // runs code if the current token matches or you get it
        #[allow(unused_macros)]
        macro_rules! if_tok {
            (== $token:ident: $code:block) => {
                match tok!(0) {
                    Token::$token => $code,
                    _ => (),
                }
            };
            (!= $token:ident: $code:block) => {
                match tok!(0) {
                    Token::$token => (),
                    _ => $code,
                }
            };
            (== $token:ident: $code:block else $else_code:block) => {
                match tok!(0) {
                    Token::$token => $code,
                    _ => $else_code,
                }
            };
            (!= $token:ident: $code:block else $else_code:block) => {
                match tok!(0) {
                    Token::$token => $else_code,
                    _ => $code,
                }
            };
        }

        // calls a parsing function and automatically handles updating the position and destructuring
        // can also pass in one argument such as in the case of parse_op
        #[allow(unused_macros)]
        macro_rules! parse {
            ($fn:ident => let $p:pat) => {
                let parsed = $fn($parse_data, $ast_data, $pos)?;
                $pos = parsed.1;
                let $p = parsed.0;
            };
            ($fn:ident => $v:ident) => {
                let parsed = $fn($parse_data, $ast_data, $pos)?;
                $pos = parsed.1;
                $v = parsed.0;
            };
            ($fn:ident ($arg:expr) => let $p:pat) => {
                let parsed = $fn($parse_data, $ast_data, $pos, $arg)?;
                $pos = parsed.1;
                let $p = parsed.0;
            };
            ($fn:ident ($arg:expr) => $v:ident) => {
                let parsed = $fn($parse_data, $ast_data, $pos, $arg)?;
                $pos = parsed.1;
                $v = parsed.0;
            };
        }
    };
}

#[derive(PartialEq, Debug)]
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
            use super::super::lexer::Token;
            use super::OpType;

            pub fn infix_prec(tok: &Token) -> usize {
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
                1000000
            }
            pub fn unary_prec(tok: &Token) -> usize {
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
                1000000
            }
            pub fn is_unary(tok: &Token) -> bool {
                let mut utoks = vec![];
                $(
                    if OpType::$optype == OpType::Unary {
                        $(
                            utoks.push( Token::$tok );
                        )+
                    }
                )*
                return utoks.contains( tok );
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
    pub item_areas: Vec<CodeArea>,
}

// helper function to parse anything that acts like a dict (dicts, instances, impls)
pub fn parse_dictlike(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ParsedDictlike, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let mut items = vec![];
    let mut item_areas = vec![];

    while_tok!(!= RBracket: {
        check_tok!(Ident(key):key_span else "key");
        item_areas.push( parse_data.source.to_area(key_span) );
        let mut elem = None;
        if_tok!(== Colon: {
            pos += 1;
            parse!(parse_expr => let temp); elem = Some(temp);
        });
        items.push((key, elem));
        if !matches!(tok!(0), Token::RBracket | Token::Comma) {
            expected_err!("} or ,", tok!(0), span!(0))
        }
        skip_tok!(Comma);
    });

    Ok((ParsedDictlike { items, item_areas }, pos))

    // match ast_data[ASTKey::default()].0.into_expr() {
    //     Expression::Literal(_) => todo!(),
    //     Expression::Op(_, _, _) => todo!(),
    //     Expression::Unary(_, _) => todo!(),
    // }

    // todo!()
}
