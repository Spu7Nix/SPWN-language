use std::str::Chars;

use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::error::SyntaxError;
use super::lexer::{Token, Tokens};

use crate::interpreter::value::Value;

use crate::parse_util;
use crate::parser::parse_util::parse_dictlike;
use crate::sources::{CodeArea, SpwnSource};

use super::parse_util::{operators, OpType};

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
    fn to_key(&self) -> KeyType;
}
impl ASTKey for ExprKey {
    fn to_key(&self) -> KeyType {
        KeyType::Expr(*self)
    }
}
impl ASTKey for StmtKey {
    fn to_key(&self) -> KeyType {
        KeyType::StmtKey(*self)
    }
}

#[derive(Default)]
pub struct ASTData {
    pub exprs: SlotMap<ExprKey, (Expression, CodeArea)>,
    pub stmts: SlotMap<StmtKey, (Statement, CodeArea)>,

    pub stmt_arrows: SecondaryMap<StmtKey, bool>,

    pub for_loop_iter_areas: SecondaryMap<StmtKey, CodeArea>,
    pub func_arg_areas: SecondaryMap<ExprKey, Vec<CodeArea>>,

    pub dictlike_areas: SecondaryMap<ExprKey, Vec<CodeArea>>,
}
impl ASTData {
    // pub fn insert<T: ASTNode + 'static>(&mut self, node: T, area: CodeArea) -> ASTKey {
    //     self.map.insert((Box::new(node), area))
    // }
    pub fn get_area<K: ASTKey>(&self, k: K) -> &CodeArea {
        match k.to_key() {
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
    Print(ExprKey),
}

pub type Statements = Vec<StmtKey>;

// parses one unit value
fn parse_unit(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let start = span!(0);

    match tok!(0) {
        Token::Int(n) => Ok((
            ast_data.insert_expr(Expression::Literal(Literal::Int(*n)), span_ar!(0)),
            pos + 1,
        )),
        Token::BinaryLiteral(b) => Ok((
            ast_data.insert_expr(
                Expression::Literal(Literal::Int(parse_number_radix(b, 2, "0b", span_ar!(0))?)),
                span_ar!(0),
            ),
            pos + 1,
        )),
        Token::HexLiteral(h) => Ok((
            ast_data.insert_expr(
                Expression::Literal(Literal::Int(parse_number_radix(h, 16, "0x", span_ar!(0))?)),
                span_ar!(0),
            ),
            pos + 1,
        )),
        Token::OctalLiteral(o) => Ok((
            ast_data.insert_expr(
                Expression::Literal(Literal::Int(parse_number_radix(o, 8, "0o", span_ar!(0))?)),
                span_ar!(0),
            ),
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
            ast_data.insert_expr(
                Expression::Literal(Literal::String(parse_string(s, span_ar!(0))?)),
                span_ar!(0),
            ),
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
                    let mut arg_areas = vec![];
                    while_tok!(!= RParen: {
                        check_tok!(Ident(arg):arg_span else "argument name");

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
                        arg_areas.push(parse_data.source.to_area(arg_span));
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

                    let key = ast_data.insert_expr(
                        Expression::Func {
                            args,
                            ret_type,
                            code,
                        },
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    );

                    ast_data.func_arg_areas.insert(key, arg_areas);

                    Ok((key, pos))
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
                parse!(parse_dictlike => let info);

                Ok((
                    ast_data.exprs.insert_with_key(|key| {
                        ast_data.dictlike_areas.insert(key, info.item_areas);
                        (
                            Expression::Dict(info.items),
                            parse_data.source.to_area((start.0, span!(-1).1)),
                        )
                    }),
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

        unary_op if operators::is_unary(unary_op) => {
            pos += 1;
            let prec = operators::unary_prec(unary_op);
            let mut next_prec = if prec + 1 < operators::prec_amount() {
                prec + 1
            } else {
                1000000
            };
            while next_prec != 1000000 {
                if operators::prec_type(next_prec) == OpType::Unary {
                    next_prec += 1
                } else {
                    break;
                }
                if next_prec == operators::prec_amount() {
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
) -> Result<(ExprKey, usize), SyntaxError> {
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
                let mut param_areas = vec![];

                while_tok!(!= RParen: {

                    if !started_named {
                        match (tok!(0), tok!(1)) {
                            (Token::Ident(name), Token::Assign) => {
                                started_named = true;
                                let start = span!(0);
                                pos += 2;
                                parse!(parse_expr => let arg);
                                param_areas.push(parse_data.source.to_area((start.0, span!(-1).1)));
                                named_params.push((name.into(), arg));
                            }
                            _ => {
                                let start = span!(0);
                                parse!(parse_expr => let arg);
                                param_areas.push(parse_data.source.to_area((start.0, span!(-1).1)));
                                params.push(arg);
                            }
                        }
                    } else {
                        let start = span!(0);
                        check_tok!(Ident(name) else "parameter name");
                        check_tok!(Assign else "=");
                        parse!(parse_expr => let arg);
                        param_areas.push(parse_data.source.to_area((start.0, span!(-1).1)));
                        named_params.push((name, arg));
                    }

                    if !matches!(tok!(0), Token::RParen | Token::Comma) {
                        expected_err!(") or ,", tok!(0), span!(0))
                    }
                    skip_tok!(Comma);
                });

                let key = ast_data.insert_expr(
                    Expression::Call {
                        base: value,
                        params,
                        named_params,
                    },
                    parse_data.source.to_area((start.0, span!(-1).1)),
                );

                ast_data.func_arg_areas.insert(key, param_areas);

                value = key;
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
                parse!(parse_dictlike => let info);

                value = ast_data.exprs.insert_with_key(|key| {
                    ast_data.dictlike_areas.insert(key, info.item_areas);
                    (
                        Expression::Instance(value, info.items),
                        parse_data.source.to_area((start.0, span!(-1).1)),
                    )
                });
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
pub fn parse_expr(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    pos: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_op(parse_data, ast_data, pos, 0)
}

// parses operators and automatically handles precedence
fn parse_op(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
    prec: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let mut next_prec = if prec + 1 < operators::prec_amount() {
        prec + 1
    } else {
        1000000
    };
    while next_prec != 1000000 {
        if operators::prec_type(next_prec) == OpType::Unary {
            next_prec += 1
        } else {
            break;
        }
        if next_prec == operators::prec_amount() {
            next_prec = 1000000
        };
    }
    let mut left;
    if next_prec != 1000000 {
        parse!(parse_op(next_prec) => left);
    } else {
        parse!(parse_value => left);
    }

    while operators::infix_prec(tok!(0)) == prec {
        let op = tok!(0).clone();
        pos += 1;
        let right;
        if operators::prec_type(prec) == OpType::LeftAssoc {
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
pub fn parse_statement(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(StmtKey, usize), SyntaxError> {
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

    // dummy thing just so we can get a key so stuff can insert extra info
    let stmt_key = ast_data.insert_stmt(Statement::Break, parse_data.source.to_area(start));

    let stmt = match tok!(0) {
        Token::Let => {
            pos += 1;
            check_tok!(Ident(var_name) else "variable name");
            check_tok!(Assign else "=");
            parse!(parse_expr => let value);
            Statement::Let(var_name, value)
        }
        Token::Print => {
            pos += 1;
            parse!(parse_expr => let value);
            Statement::Print(value)
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
            check_tok!(Ident(var):var_span else "variable name");
            check_tok!(In else "in");
            parse!(parse_expr => let iterator);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");

            ast_data
                .for_loop_iter_areas
                .insert(stmt_key, parse_data.source.to_area(var_span));

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

    ast_data.stmts[stmt_key] = (stmt, parse_data.source.to_area((start.0, span!(-1).1)));
    ast_data.stmt_arrows.insert(stmt_key, is_arrow);

    Ok((stmt_key, pos))
}

// parses statements lol
pub fn parse_statements(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(Statements, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let mut statements = vec![];

    while !matches!(tok!(0), Token::Eof | Token::RBracket) {
        parse!(parse_statement => let stmt);
        statements.push(stmt);
    }

    Ok((statements, pos))
}

fn parse_number_radix(
    string: &String,
    radix: u32,
    prefix: &str,
    area: CodeArea,
) -> Result<usize, SyntaxError> {
    let mstring = string.clone();
    usize::from_str_radix(&mstring.replace(prefix, ""), radix).map_err(|_| {
        SyntaxError::InvalidLiteral {
            literal: string.to_owned(),
            area,
        }
    })
}

fn parse_unicode(chars: &mut Chars, area: CodeArea) -> Result<char, SyntaxError> {
    let mut out = String::new();

    for c in chars.take(4) {
        out.push(c);
    }

    let hex = parse_number_radix(&out, 16, "", area)?;

    Ok(char::from_u32(hex as u32).unwrap_or('�'))
}

// deals with parsing string escape sequences
fn parse_string(s: &str, area: CodeArea) -> Result<String, SyntaxError> {
    let mut out = String::new();
    let mut chars = s[1..s.len() - 1].chars();

    loop {
        match chars.next() {
            Some('\\') => out.push(match chars.next() {
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('"') => '"',
                Some('\'') => '\'',
                Some('\\') => '\\',
                // parse it as in a separate fn for readability
                Some('u') => parse_unicode(&mut chars, area.clone())?,
                Some(a) => return Err(SyntaxError::InvalidEscape { character: a, area }),

                None => unreachable!(),
            }),
            Some(c) => out.push(c),
            None => break,
        }
    }

    println!("{out}");

    Ok(out)
}

// beginning parse function
pub fn parse(parse_data: &ParseData, ast_data: &mut ASTData) -> Result<Statements, SyntaxError> {
    let mut pos = 0;
    parse_util!(parse_data, ast_data, pos);

    parse!(parse_statements => let stmts);
    check_tok_static!(Eof else "end of file");
    Ok(stmts)
}
