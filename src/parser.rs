use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::error::{Result, SyntaxError};
use crate::lexer::{Token, Tokens};
use crate::sources::{CodeArea, SpwnSource};
use crate::value::Value;

new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

// just helper for ASTData::area
pub enum KeyType {
    Expr(ExprKey),
    StmtKey(StmtKey),
}

// just helper for ASTData::area
pub trait ASTKey {
    fn into_key(self) -> KeyType;
}
impl ASTKey for ExprKey {
    fn into_key(self) -> KeyType {
        KeyType::Expr(self)
    }
}
impl ASTKey for StmtKey {
    fn into_key(self) -> KeyType {
        KeyType::StmtKey(self)
    }
}

#[derive(Default)]
pub struct ASTData {
    pub exprs: SlotMap<ExprKey, (Expression, CodeArea)>,
    pub stmts: SlotMap<StmtKey, (Statement, CodeArea)>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,
}
impl ASTData {
    // pub fn insert<T: ASTNode + 'static>(&mut self, node: T, area: CodeArea) -> ASTKey {
    //     self.map.insert((Box::new(node), area))
    // }
    pub fn get_area<K: ASTKey>(&self, k: K) -> &CodeArea {
        match k.into_key() {
            KeyType::Expr(k) => &self.exprs[k].1,
            KeyType::StmtKey(k) => &self.stmts[k].1,
        }
    }
    pub fn get_expr(&self, k: ExprKey) -> Expression {
        self.exprs[k].0.clone()
    }
    pub fn get_stmt(&self, k: StmtKey) -> Statement {
        self.stmts[k].0.clone()
    }
    pub fn insert_expr(&mut self, expr: Expression, area: CodeArea) -> ExprKey {
        self.exprs.insert((expr, area))
    }
    pub fn insert_stmt(&mut self, stmt: Statement, area: CodeArea) -> StmtKey {
        self.stmts.insert((stmt, area))
    }

    pub fn debug(&self, stmts: &Statements) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        debug_str += "-------- exprs --------\n";
        for (k, (e, _)) in &self.exprs {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-------- stmts --------\n";
        for (k, (e, _)) in &self.stmts {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-----------------------\n";

        for i in stmts {
            writeln!(&mut debug_str, "{:?}", i).unwrap();
        }

        let re = regex::Regex::new(r"(ExprKey\([^)]*\))").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Yellow.bold().paint("$1").to_string(),
            )
            .into();
        let re = regex::Regex::new(r"(StmtKey\([^)]*\))").unwrap();
        debug_str = re
            .replace_all(
                &debug_str,
                ansi_term::Color::Blue.bold().paint("$1").to_string(),
            )
            .into();

        println!("{}", debug_str);
    }
}

// holds immutable data relevant to parsing
pub struct ParseData {
    pub tokens: Tokens,
    pub source: SpwnSource,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(usize),
    Float(f64),
    String(String),
    Bool(bool),
}
impl Literal {
    pub fn to_str(&self) -> String {
        match self {
            Literal::Int(v) => v.to_string(),
            Literal::Float(v) => v.to_string(),
            Literal::String(v) => v.to_string(),
            Literal::Bool(v) => v.to_string(),
        }
    }
    pub fn to_value(&self) -> Value {
        match self {
            Literal::Int(v) => Value::Int(*v as isize),
            Literal::Float(v) => Value::Float(*v),
            Literal::String(v) => Value::String(v.clone()),
            Literal::Bool(v) => Value::Bool(*v),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Op(ExprKey, Token, ExprKey),
    Unary(Token, ExprKey),

    Var(String),
    Type(String),

    Array(Vec<ExprKey>),
    Dict(Vec<(String, Option<ExprKey>)>),

    // Index { base: ExprKey, index: ExprKey },
    Empty,

    Block(Statements),

    Func {
        args: Vec<(String, Option<ExprKey>, Option<ExprKey>)>,
        ret_type: Option<ExprKey>,
        code: ExprKey,
    },
    FuncPattern {
        args: Vec<ExprKey>,
        ret_type: ExprKey,
    },

    Ternary {
        cond: ExprKey,
        if_true: ExprKey,
        if_false: ExprKey,
    },

    Index {
        base: ExprKey,
        index: ExprKey,
    },
    Call {
        base: ExprKey,
        params: Vec<ExprKey>,
        named_params: Vec<(String, ExprKey)>,
    },
    TriggerFuncCall(ExprKey),

    Maybe(Option<ExprKey>),

    TriggerFunc(Statements),

    Instance(ExprKey, Vec<(String, Option<ExprKey>)>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(ExprKey),
    Let(String, ExprKey),
    Assign(String, ExprKey),
    If {
        branches: Vec<(ExprKey, Statements)>,
        else_branch: Option<Statements>,
    },
    While {
        cond: ExprKey,
        code: Statements,
    },
    For {
        var: String,
        iterator: ExprKey,
        code: Statements,
    },
    Return(Option<ExprKey>),
    Break,
    Continue,

    TypeDef(String),
    Impl(ExprKey, Vec<(String, ExprKey)>),
}

pub type Statements = Vec<StmtKey>;

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
enum OpType {
    LeftAssoc,
    RightAssoc,
    Unary,
}

macro_rules! operators {
    (
        $(
            $optype:ident <== [$($tok:ident)+],
        )*
    ) => {
        fn infix_prec(tok: &Token) -> usize {
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
        fn unary_prec(tok: &Token) -> usize {
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
        fn is_unary(tok: &Token) -> bool {
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
        fn prec_amount() -> usize {
            let mut amount = 0;
            $(
                amount += 1;
                format!("{:?}", OpType::$optype);
            )*
            amount
        }
        fn prec_type(mut prec: usize) -> OpType {
            $(
                if prec == 0 {
                    return OpType::$optype;
                }
                prec -= 1;
                format!("{}", prec);
            )*
            unreachable!()
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
    // Unary       <==  [ ExclMark ],
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

fn can_be_expr(tok: &Token) -> bool {
    use Token::*;
    matches!(
        tok,
        Int(_)
            | Float(_)
            | True
            | False
            | String(_)
            | Ident(_)
            | TypeIndicator(_)
            | LParen
            | LBracket
            | LSqBracket
    ) || is_unary(tok)
}

// parses one unit value
fn parse_unit(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ExprKey, usize)> {
    parse_util!(parse_data, ast_data, pos);

    let start = span!(0);

    match tok!(0) {
        Token::Int(n) => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::Int(*n)), span_ar!(0)),
            pos + 1,
        )),
        Token::Float(n) => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::Float(*n)), span_ar!(0)),
            pos + 1,
        )),
        Token::True => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::Bool(true)), span_ar!(0)),
            pos + 1,
        )),
        Token::False => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::Bool(false)), span_ar!(0)),
            pos + 1,
        )),
        Token::String(s) => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::String(s.into())), span_ar!(0)),
            pos + 1,
        )),
        Token::Ident(name) => {
            pos += 1;
            if_tok!(== FatArrow: {
                pos += 1;
                parse!(parse_expr => let code);
                Ok((
                    ast_data.insert_expr(
                        Expression::Func {
                            args: vec![(name.clone(), None, None)],
                            ret_type: None,
                            code,
                        },
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    ),
                    pos,
                ))
            } else {
                pos -= 1;
                Ok((
                    ast_data.insert_expr(Expression::Var(name.into()), span_ar!(0)),
                    pos + 1,
                ))
            })
        }
        Token::TypeIndicator(name) => Ok((
            ast_data.insert_expr(Expression::Type(name.into()), span_ar!(0)),
            pos + 1,
        )),

        Token::LParen => {
            pos += 1;

            if matches!(tok!(0), Token::RParen) && !matches!(tok!(1), Token::FatArrow) {
                pos += 1;
                Ok((
                    ast_data.insert_expr(
                        Expression::Empty,
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    ),
                    pos,
                ))
            } else {
                let mut i = 0;
                let mut depth = 1;

                loop {
                    match tok!(i) {
                        Token::LParen => depth += 1,
                        Token::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        Token::Eof => {
                            return Err(SyntaxError::UnmatchedChar {
                                for_char: "(".to_string(),
                                not_found: ")".to_string(),
                                area: parse_data.source.to_area(start),
                            })
                        }
                        _ => (),
                    }
                    i += 1;
                }

                let is_pattern;

                match tok!(i) {
                    Token::FatArrow => {
                        is_pattern = false;
                    }
                    Token::Arrow => {
                        let prev_pos = pos;
                        pos = pos + i as usize + 1;

                        // its ok to parse here as an expression since we'll do it anyway
                        // later, so if we catch an error here everything is fine
                        parse!(parse_expr => let _);

                        is_pattern = !matches!(tok!(0), Token::FatArrow);
                        pos = prev_pos;
                    }
                    _ => {
                        parse!(parse_expr => let value);
                        check_tok!(RParen else ")");
                        return Ok((value, pos));
                    }
                }

                if !is_pattern {
                    let mut args = vec![];
                    while_tok!(!= RParen: {
                        check_tok!(Ident(arg) else "argument name");
                        let mut arg_type = None;
                        if_tok!(== Colon: {
                            pos += 1;
                            parse!(parse_expr => let temp); arg_type = Some(temp);
                        });
                        let mut arg_default = None;
                        if_tok!(== Assign: {
                            pos += 1;
                            parse!(parse_expr => let temp); arg_default = Some(temp);
                        });
                        args.push((arg, arg_type, arg_default));
                        if !matches!(tok!(0), Token::RParen | Token::Comma) {
                            expected_err!(") or ,", tok!(0), span!(0))
                        }
                        skip_tok!(Comma);
                    });
                    let mut ret_type = None;
                    if_tok!(== Arrow: {
                        pos += 1;
                        parse!(parse_expr => let temp); ret_type = Some(temp);
                    });
                    check_tok!(FatArrow else "=>");
                    parse!(parse_expr => let code);

                    Ok((
                        ast_data.insert_expr(
                            Expression::Func {
                                args,
                                ret_type,
                                code,
                            },
                            parse_data.source.to_area((start.0, span!(-1).1)),
                        ),
                        pos,
                    ))
                } else {
                    let mut args = vec![];
                    while_tok!(!= RParen: {
                        parse!(parse_expr => let arg);
                        args.push(arg);
                        if !matches!(tok!(0), Token::RParen | Token::Comma) {
                            expected_err!(") or ,", tok!(0), span!(0))
                        }
                        skip_tok!(Comma);
                    });
                    check_tok!(Arrow else "->");
                    parse!(parse_expr => let ret_type);

                    Ok((
                        ast_data.insert_expr(
                            Expression::FuncPattern { args, ret_type },
                            parse_data.source.to_area((start.0, span!(-1).1)),
                        ),
                        pos,
                    ))
                }
            }

            // if_tok!(== RParen: {
            //     Ok((ast_data.insert_expr(
            //         Expression::Empty,
            //         parse_data.source.to_area( (start.0, span!(-1).1) )
            //     ), pos + 1))
            // } else {
            //     parse!(parse_expr => let value);
            //     check_tok!(RParen else ")");
            //     Ok((value, pos))
            // })
        }

        Token::LSqBracket => {
            pos += 1;

            let mut elements = vec![];
            while_tok!(!= RSqBracket: {
                parse!(parse_expr => let elem);
                elements.push(elem);
                if !matches!(tok!(0), Token::RSqBracket | Token::Comma) {
                    expected_err!("] or ,", tok!(0), span!(0))
                }
                skip_tok!(Comma);
            });

            Ok((
                ast_data.insert_expr(
                    Expression::Array(elements),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                ),
                pos,
            ))
        }

        Token::LBracket => {
            pos += 1;

            if_tok!(== RBracket: {
                pos += 1;
                return Ok((
                    ast_data.insert_expr(
                        Expression::Dict(vec![]),
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    ),
                    pos,
                ))
            });

            if !(matches!(tok!(0), Token::Ident(_)) && matches!(tok!(1), Token::Colon)) {
                parse!(parse_statements => let code);
                check_tok!(RBracket else "}");
                Ok((
                    ast_data.insert_expr(
                        Expression::Block(code),
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    ),
                    pos,
                ))
            } else {
                let mut items = vec![];

                while_tok!(!= RBracket: {
                    check_tok!(Ident(key) else "key");
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

                Ok((
                    ast_data.insert_expr(
                        Expression::Dict(items),
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    ),
                    pos,
                ))
            }
        }

        Token::QMark => Ok((
            ast_data.insert_expr(Expression::Maybe(None), span_ar!(0)),
            pos + 1,
        )),

        Token::ExclMark if matches!(tok!(1), Token::LBracket) => {
            pos += 2;
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            Ok((
                ast_data.insert_expr(
                    Expression::TriggerFunc(code),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                ),
                pos,
            ))
        }

        unary_op if is_unary(unary_op) => {
            pos += 1;
            let prec = unary_prec(unary_op);
            let mut next_prec = if prec + 1 < prec_amount() {
                prec + 1
            } else {
                1000000
            };
            while next_prec != 1000000 {
                if prec_type(next_prec) == OpType::Unary {
                    next_prec += 1
                } else {
                    break;
                }
                if next_prec == prec_amount() {
                    next_prec = 1000000
                }
            }
            let value;
            if next_prec != 1000000 {
                parse!(parse_op(next_prec) => value);
            } else {
                parse!(parse_value => value);
            }

            Ok((
                ast_data.insert_expr(
                    Expression::Unary(unary_op.clone(), value),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                ),
                pos,
            ))
        }

        other => expected_err!("expression", other, span!(0)),
    }

    // match ast_data[ASTKey::default()].0.into_expr() {
    //     Expression::Literal(_) => todo!(),
    //     Expression::Op(_, _, _) => todo!(),
    //     Expression::Unary(_, _) => todo!(),
    // }

    // todo!()
}

// parses a full value, aka stuff after like indexing, calling, member access etc
fn parse_value(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ExprKey, usize)> {
    parse_util!(parse_data, ast_data, pos);

    parse!(parse_unit => let mut value);
    let start = ast_data.get_area(value).span;

    while matches!(
        tok!(0),
        Token::LSqBracket
            | Token::If
            | Token::LParen
            | Token::QMark
            | Token::DoubleColon
            | Token::ExclMark
    ) {
        match tok!(0) {
            Token::LSqBracket => {
                pos += 1;
                parse!(parse_expr => let index);
                check_tok!(RSqBracket else "]");
                value = ast_data.insert_expr(
                    Expression::Index { base: value, index },
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            Token::If => {
                pos += 1;
                parse!(parse_expr => let cond);
                check_tok!(Else else "else");
                parse!(parse_expr => let if_false);
                value = ast_data.insert_expr(
                    Expression::Ternary {
                        cond,
                        if_true: value,
                        if_false,
                    },
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            Token::LParen => {
                pos += 1;
                let mut params = vec![];
                let mut named_params = vec![];
                let mut started_named = false;

                while_tok!(!= RParen: {

                    if !started_named {
                        match (tok!(0), tok!(1)) {
                            (Token::Ident(name), Token::Assign) => {
                                started_named = true;
                                pos += 2;
                                parse!(parse_expr => let arg);
                                named_params.push((name.into(), arg));
                            }
                            _ => {
                                parse!(parse_expr => let arg);
                                params.push(arg);
                            }
                        }
                    } else {
                        check_tok!(Ident(name) else "parameter name");
                        check_tok!(Assign else "=");
                        parse!(parse_expr => let arg);
                        named_params.push((name, arg));
                    }

                    if !matches!(tok!(0), Token::RParen | Token::Comma) {
                        expected_err!(") or ,", tok!(0), span!(0))
                    }
                    skip_tok!(Comma);
                });
                value = ast_data.insert_expr(
                    Expression::Call {
                        base: value,
                        params,
                        named_params,
                    },
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            Token::QMark => {
                pos += 1;
                value = ast_data.insert_expr(
                    Expression::Maybe(Some(value)),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            Token::DoubleColon => {
                pos += 1;
                check_tok!(LBracket else "{");

                let mut items = vec![];

                while_tok!(!= RBracket: {
                    check_tok!(Ident(key) else "key");
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

                value = ast_data.insert_expr(
                    Expression::Instance(value, items),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            Token::ExclMark => {
                pos += 1;
                value = ast_data.insert_expr(
                    Expression::TriggerFuncCall(value),
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );
            }
            _ => unreachable!(),
        }
    }

    Ok((value, pos))
}

// shorthand for expression parsings
fn parse_expr(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    pos: usize,
) -> Result<(ExprKey, usize)> {
    parse_op(parse_data, ast_data, pos, 0)
}

// parses operators and automatically handles precedence
fn parse_op(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
    prec: usize,
) -> Result<(ExprKey, usize)> {
    parse_util!(parse_data, ast_data, pos);

    let mut next_prec = if prec + 1 < prec_amount() {
        prec + 1
    } else {
        1000000
    };
    while next_prec != 1000000 {
        if prec_type(next_prec) == OpType::Unary {
            next_prec += 1
        } else {
            break;
        }
        if next_prec == prec_amount() {
            next_prec = 1000000
        };
    }
    let mut left;
    if next_prec != 1000000 {
        parse!(parse_op(next_prec) => left);
    } else {
        parse!(parse_value => left);
    }

    while infix_prec(tok!(0)) == prec {
        let op = tok!(0).clone();
        pos += 1;
        let right;
        if prec_type(prec) == OpType::LeftAssoc {
            if next_prec != 1000000 {
                parse!(parse_op(next_prec) => right);
            } else {
                parse!(parse_value => right);
            }
        } else {
            parse!(parse_op(prec) => right);
        }
        let (left_span, right_span) = (ast_data.get_area(left).span, ast_data.get_area(right).span);
        left = ast_data.insert_expr(
            Expression::Op(left, op, right),
            parse_data.source.to_area((left_span.0, right_span.1)),
        );
    }
    Ok((left, pos))
}

// parses statements
fn parse_statement(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(StmtKey, usize)> {
    parse_util!(parse_data, ast_data, pos);
    let start = span!(0);

    macro_rules! expr_stmt {
        () => {
            {
                parse!(parse_expr => let value);
                Statement::Expr(value)
            }
        };
    }

    let is_arrow = if let Token::Arrow = tok!(0) {
        pos += 1;
        true
    } else {
        false
    };

    let stmt = match tok!(0) {
        Token::Let => {
            pos += 1;
            check_tok!(Ident(var_name) else "variable name");
            check_tok!(Assign else "=");
            parse!(parse_expr => let value);
            Statement::Let(var_name, value)
        }
        Token::If => {
            pos += 1;

            let mut branches = vec![];
            let mut else_branch = None;

            parse!(parse_expr => let cond);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            branches.push((cond, code));

            while let Token::Else = tok!(0) {
                pos += 1;
                if_tok!(== If: {
                    pos += 1;
                    parse!(parse_expr => let cond);
                    check_tok!(LBracket else "{");
                    parse!(parse_statements => let code);
                    check_tok!(RBracket else "}");
                    branches.push((cond, code));
                } else {
                    check_tok!(LBracket else "{");
                    parse!(parse_statements => let temp); else_branch = Some(temp);
                    check_tok!(RBracket else "}");
                    break;
                })
            }

            Statement::If {
                branches,
                else_branch,
            }
        }
        Token::While => {
            pos += 1;
            parse!(parse_expr => let cond);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            Statement::While { cond, code }
        }
        Token::For => {
            pos += 1;
            check_tok!(Ident(var) else "variable name");
            check_tok!(In else "in");
            parse!(parse_expr => let iterator);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            Statement::For {
                code,
                var,
                iterator,
            }
        }
        Token::Return => {
            pos += 1;
            if matches!(tok!(0), Token::Eol | Token::RBracket) {
                Statement::Return(None)
            } else {
                parse!(parse_expr => let val);
                Statement::Return(Some(val))
            }
        }
        Token::Continue => {
            pos += 1;
            Statement::Continue
        }
        Token::Break => {
            pos += 1;
            Statement::Break
        }
        Token::TypeDef => {
            pos += 1;
            check_tok!(TypeIndicator(name) else "type indicator");
            Statement::TypeDef(name)
        }
        Token::Impl => {
            pos += 1;
            parse!(parse_expr => let typ);
            check_tok!(LBracket else "{");

            let mut items = vec![];

            while_tok!(!= RBracket: {
                check_tok!(Ident(key) else "key");
                check_tok!(Colon else ":");
                parse!(parse_expr => let elem);
                items.push((key, elem));
                if !matches!(tok!(0), Token::RBracket | Token::Comma) {
                    expected_err!("} or ,", tok!(0), span!(0))
                }
                skip_tok!(Comma);
            });
            Statement::Impl(typ, items)
        }
        Token::Ident(name) => match tok!(1) {
            Token::Assign => {
                pos += 2;
                parse!(parse_expr => let val);
                Statement::Assign(name.clone(), val)
            }
            _ => expr_stmt!(),
        },
        _ => expr_stmt!(),
    };

    if !matches!(tok!(-1), Token::RBracket) {
        check_tok!(Eol else ';');
    }
    skip_toks!(Eol);

    let key = ast_data.insert_stmt(stmt, parse_data.source.to_area((start.0, span!(-1).1)));
    ast_data.stmt_arrows.insert(key, is_arrow);

    Ok((key, pos))
}

// parses statements lol
fn parse_statements(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(Statements, usize)> {
    parse_util!(parse_data, ast_data, pos);

    let mut statements = vec![];

    while !matches!(tok!(0), Token::Eof | Token::RBracket) {
        parse!(parse_statement => let stmt);
        statements.push(stmt);
    }

    Ok((statements, pos))
}

// beginning parse function
pub fn parse(parse_data: &ParseData, ast_data: &mut ASTData) -> Result<Statements> {
    let mut pos = 0;
    parse_util!(parse_data, ast_data, pos);

    parse!(parse_statements => let stmts);
    check_tok_static!(Eof else "end of file");
    Ok(stmts)
}
