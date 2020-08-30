use crate::ast;
/*use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;*/

use std::path::PathBuf;
//use std::collections::HashMap;

use logos::Lexer;
use logos::Logos;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedErr {
        expected: String,
        found: String,
        pos: (usize, usize),
    },
    UnexpectedErr {
        found: String,
        pos: (usize, usize),
    },
    SyntaxError {
        message: String,
        pos: (usize, usize),
    },
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "SuperErrorSideKick is here!")
        match self {
            SyntaxError::ExpectedErr {
                expected,
                found,
                pos,
            } => write!(
                f,
                "SyntaxError: Expected {}, found {} (line {}, position {})",
                expected, found, pos.0, pos.1
            ),

            SyntaxError::UnexpectedErr { found, pos } => write!(
                f,
                "SyntaxError: Unexpected {} on line {}, position {}",
                found, pos.0, pos.1
            ),

            SyntaxError::SyntaxError { message, pos } => write!(
                f,
                "SyntaxError: {}, (line {}, position {})",
                message, pos.0, pos.1
            ),
        }
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum Token {
    //OPERATORS
    #[token("->")]
    Arrow,

    #[token("||")]
    Or,

    #[token("&&")]
    And,

    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token(">=")]
    MoreOrEqual,

    #[token("<=")]
    LessOrEqual,

    #[token(">")]
    MoreThan,

    #[token("<")]
    LessThan,

    #[token("*")]
    Multiply,

    #[token("%")]
    Modulo,

    #[token("^")]
    Power,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("/")]
    Divide,

    #[token("!")]
    Exclamation,

    #[token("=")]
    Assign,

    //VALUES
    #[regex(r"([a-zA-Z_][a-zA-Z0-9_]*)|\$")]
    Symbol,

    #[regex(r"[0-9]+(\.[0-9]+)?")]
    Number,

    #[regex("\"[^\n\r\"]*\"")]
    StringLiteral,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex(r"[0-9?]+[gbci]")]
    ID,

    //TERMINATORS
    #[token(",")]
    Comma,

    #[token("{")]
    OpenCurlyBracket,

    #[token("}")]
    ClosingCurlyBracket,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    ClosingSquareBracket,

    #[token("(")]
    OpenBracket,

    #[token(")")]
    ClosingBracket,

    #[token(":")]
    Colon,

    #[token(".")]
    Period,

    #[token("..")]
    DotDot,

    #[token("@")]
    At,

    //KEY WORDS
    #[token("return")]
    Return,

    /*#[token("<+")]
    Add,*/
    #[token("impl")]
    Implement,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("error")]
    ErrorStatement,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("obj")]
    Object,

    #[token("import")]
    Import,

    #[token("extract")]
    Extract,

    #[token("null")]
    Null,

    //STATEMENT SEPARATOR
    #[regex(r"[\n\r;]([ \t\f]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*|[\n\r;])*")]
    StatementSeparator,

    #[error]
    #[regex(r"[ \t\f]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    Error,
}

pub struct ParseNotes {
    pub closed_groups: Vec<u16>,
    pub closed_colors: Vec<u16>,
    pub closed_blocks: Vec<u16>,
    pub closed_items: Vec<u16>,
}

pub struct Tokens<'a> {
    iter: Lexer<'a, Token>,
    stack: Vec<(Token, String, core::ops::Range<usize>)>,
    //index 0 = element of iter / last element in stack
    index: usize,
}

impl<'a> Tokens<'a> {
    fn new(iter: Lexer<'a, Token>) -> Self {
        Tokens {
            iter,
            stack: Vec::new(),
            index: 0,
        }
    }
    fn next(&mut self, dont_skip_ss: bool) -> Option<Token> {
        if self.index == 0 {
            let next_element = match self.iter.next() {
                Some(e) => Some(e),
                None => {
                    if dont_skip_ss {
                        Some(Token::StatementSeparator)
                    } else {
                        None
                    }
                }
            };

            if let Some(e) = next_element {
                let slice = self.iter.slice().to_string();
                let range = self.iter.span();
                self.stack.push((e, slice, range));
            }

            if !dont_skip_ss && next_element == Some(Token::StatementSeparator) {
                self.next(false)
            } else {
                next_element
            }
        } else {
            self.index -= 1;
            if !dont_skip_ss
                && self.stack[self.stack.len() - self.index - 1].0 == Token::StatementSeparator
            {
                self.next(false)
            } else {
                Some(self.stack[self.stack.len() - self.index - 1].0)
            }
        }
    }
    fn previous(&mut self) -> Option<Token> {
        self.index += 1;
        let len = self.stack.len();
        if len > self.index {
            if self.stack[len - self.index - 1].0 == Token::StatementSeparator {
                self.index += 1;
            }
            if len - self.index >= 1 {
                Some(self.stack[len - self.index - 1].0)
            } else {
                None
            }
        } else {
            None
        }
    }

    /*fn current(&self) -> Option<Token> {
        let len = self.stack.len();
        if len == 0 {
            None
        } else {
            Some(self.stack[len - self.index - 1].0)
        }
    }*/

    fn slice(&self) -> String {
        self.stack[self.stack.len() - self.index - 1].1.clone()
    }

    /*fn span(&self) -> core::ops::Range<usize> {
        self.stack[self.stack.len() - self.index - 1].2.clone()
    }*/
}

//type TokenList = Peekable<Lexer<Token>>;

const STATEMENT_SEPARATOR_DESC: &str = "Statement separator (line-break or ';')";

pub fn parse_spwn(unparsed: String) -> Result<(Vec<ast::Statement>, ParseNotes), SyntaxError> {
    let tokens_iter = Token::lexer(&unparsed);

    let mut tokens = Tokens::new(tokens_iter);
    let mut statements = Vec::<ast::Statement>::new();

    let mut notes = ParseNotes {
        closed_groups: Vec::new(),
        closed_colors: Vec::new(),
        closed_blocks: Vec::new(),
        closed_items: Vec::new(),
    };

    loop {
        //tokens.next(false);
        match tokens.next(false) {
            Some(_) => {
                tokens.previous();

                //tokens.previous();

                statements.push(parse_statement(&mut tokens, &mut notes)?)
            }
            None => break,
        }

        /*println!(
            "\n{:?}\ncurrent: {:?}, {:?}",
            statements.last(),
            tokens.current(),
            tokens.slice()
        );*/

        match tokens.next(true) {
            Some(Token::StatementSeparator) => {}
            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: STATEMENT_SEPARATOR_DESC.to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }
            None => break,
        }
    }

    Ok((statements, notes))
}

fn parse_cmp_stmt(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<ast::Statement>, SyntaxError> {
    let mut statements = Vec::<ast::Statement>::new();
    loop {
        match tokens.next(false) {
            Some(Token::ClosingCurlyBracket) => break,
            Some(_) => {
                tokens.previous();
                statements.push(parse_statement(tokens, notes)?);
                //println!("statement done");
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing a closure".to_string(),
                    pos: (0, 0),
                })
            }
        }

        match tokens.next(true) {
            Some(Token::StatementSeparator) => {}
            Some(Token::ClosingCurlyBracket) => break,
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: STATEMENT_SEPARATOR_DESC.to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                });
            }
        }
    }
    //tokens.next(false);
    Ok(statements)
}

pub fn parse_statement(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<ast::Statement, SyntaxError> {
    let first = tokens.next(false);
    // println!(
    //     "First token in statement: {:?}: {:?}",
    //     first,
    //     tokens.slice()
    // );
    let mut arrow = false;
    let body = match first {
        Some(Token::Arrow) => {
            //parse async statement
            let rest_of_statement = parse_statement(tokens, notes)?;
            if rest_of_statement.arrow {
                //double arrow (throw error)
                return Err(SyntaxError::UnexpectedErr {
                    found: "double arrow (-> ->)".to_string(),
                    pos: (0, 0),
                });
            }
            arrow = true;
            rest_of_statement.body
        }

        Some(Token::Return) => {
            //parse return statement

            match tokens.next(true) {
                Some(Token::StatementSeparator) | Some(Token::ClosingCurlyBracket) => {
                    tokens.previous();
                    ast::StatementBody::Return(None)
                }

                _ => {
                    tokens.previous();
                    ast::StatementBody::Return(Some(parse_expr(tokens, notes)?))
                }
            }
        }

        Some(Token::If) => {
            //parse if statement

            // println!("if statement");

            let condition = parse_expr(tokens, notes)?;
            match tokens.next(false) {
                Some(Token::OpenCurlyBracket) => (),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }
            }
            let if_body = parse_cmp_stmt(tokens, notes)?;
            let else_body = match tokens.next(false) {
                Some(Token::Else) => match tokens.next(false) {
                    Some(Token::OpenCurlyBracket) => {
                        // println!("else");
                        Some(parse_cmp_stmt(tokens, notes)?)
                    }
                    Some(Token::If) => {
                        tokens.previous();
                        // println!("else if");

                        Some(vec![parse_statement(tokens, notes)?])
                    }

                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "'{' or 'if'".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: (0, 0),
                        })
                    }
                },

                _ => {
                    // println!("token after if stmt: {:?}", a);
                    tokens.previous();
                    None
                }
            };

            let if_statement = ast::If {
                condition,
                if_body,
                else_body,
            };

            ast::StatementBody::If(if_statement)
        }

        Some(Token::For) => {
            //parse for statement

            let symbol = match tokens.next(false) {
                Some(Token::Symbol) => tokens.slice(),
                Some(a) => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "iterator variable name".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }

                None => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "iterator variable name".to_string(),
                        found: "None".to_string(),
                        pos: (0, 0),
                    })
                }
            };

            match tokens.next(false) {
                Some(Token::In) => {}
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "keyword 'in'".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }
            };

            let array = parse_expr(tokens, notes)?;
            match tokens.next(false) {
                Some(Token::OpenCurlyBracket) => {}
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }
            };
            let mut body = parse_cmp_stmt(tokens, notes)?;

            //fix confusing gd behavior
            if body.iter().all(|x| match x.body {
                ast::StatementBody::Call(_) => true,
                _ => false,
            }) {
                //maybe not the fastest way, but the syntax tree is just too large to just paste in
                let new_statement = parse_spwn(String::from(
                    "
(){
    $.add(obj {
        1: 1268,
        63: 0.05,
        51: {
            return
        },
    })
}()
                ",
                ))?;

                body.push(new_statement.0[0].clone());
            }

            ast::StatementBody::For(ast::For {
                symbol,
                array,
                body,
            })
        }

        Some(Token::ErrorStatement) => ast::StatementBody::Error(ast::Error {
            message: parse_expr(tokens, notes)?,
        }),

        Some(Token::Implement) => {
            //parse impl statement
            let symbol = parse_variable(tokens, notes)?;
            match tokens.next(false) {
                Some(Token::OpenCurlyBracket) => ast::StatementBody::Impl(ast::Implementation {
                    symbol,
                    members: parse_dict(tokens, notes)?,
                }),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }
            }
        }

        Some(Token::Extract) => ast::StatementBody::Extract(parse_expr(tokens, notes)?),

        Some(_) => {
            //either expression, call or definition, FIGURE OUT
            //parse it

            if tokens.next(false) == Some(Token::Assign) {
                //Def
                tokens.previous();

                // println!("found def, current val: {:?}", tokens.current());

                let symbol = tokens.slice();

                tokens.next(false);

                let value = parse_expr(tokens, notes)?;
                //tokens.next(false);
                ast::StatementBody::Definition(ast::Definition { symbol, value })
            } else {
                //expression or call
                tokens.previous();
                tokens.previous();
                let expr = parse_expr(tokens, notes)?;
                if tokens.next(false) == Some(Token::Exclamation) {
                    //call
                    // println!("found call");
                    ast::StatementBody::Call(ast::Call {
                        function: expr.values[0].clone(),
                    })
                } else {
                    //expression statement
                    // println!("found expr");
                    tokens.previous();
                    ast::StatementBody::Expr(expr)
                }
            }
        }

        None => {
            //end of input
            unimplemented!()
        }
    };

    Ok(ast::Statement {
        body,
        arrow,
        line: (0, 0),
    })
}

fn parse_expr(tokens: &mut Tokens, notes: &mut ParseNotes) -> Result<ast::Expression, SyntaxError> {
    let mut values = Vec::<ast::Variable>::new();
    let mut operators = Vec::<ast::Operator>::new();

    values.push(parse_variable(tokens, notes)?);
    loop {
        let op = match tokens.next(false) {
            Some(t) => match parse_operator(&t) {
                Some(o) => o,
                None => break,
            },
            None => break,
        };

        operators.push(op);
        values.push(parse_variable(tokens, notes)?);
    }
    tokens.previous();

    Ok(ast::Expression { values, operators })
}

fn parse_operator(token: &Token) -> Option<ast::Operator> {
    match token {
        Token::DotDot => Some(ast::Operator::Range),
        Token::Or => Some(ast::Operator::Or),
        Token::And => Some(ast::Operator::And),
        Token::Equal => Some(ast::Operator::Equal),
        Token::NotEqual => Some(ast::Operator::NotEqual),
        Token::MoreOrEqual => Some(ast::Operator::MoreOrEqual),
        Token::LessOrEqual => Some(ast::Operator::LessOrEqual),
        Token::LessThan => Some(ast::Operator::Less),
        Token::MoreThan => Some(ast::Operator::More),
        Token::Multiply => Some(ast::Operator::Multiply),
        Token::Power => Some(ast::Operator::Power),
        Token::Plus => Some(ast::Operator::Plus),
        Token::Minus => Some(ast::Operator::Minus),
        Token::Divide => Some(ast::Operator::Divide),
        Token::Modulo => Some(ast::Operator::Modulo),
        _ => None,
    }
}

fn parse_dict(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<ast::DictDef>, SyntaxError> {
    let mut defs = Vec::<ast::DictDef>::new();

    loop {
        match tokens.next(false) {
            Some(Token::Symbol) => {
                let symbol = tokens.slice();

                match tokens.next(false) {
                    Some(Token::Colon) => {
                        let expr = parse_expr(tokens, notes)?;
                        defs.push(ast::DictDef::Def((symbol, expr)));
                    }

                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "':'".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: (0, 0),
                        });
                    }
                }
            }

            Some(Token::DotDot) => {
                let expr = parse_expr(tokens, notes)?;
                defs.push(ast::DictDef::Extract(expr))
            }

            Some(Token::ClosingCurlyBracket) => break,

            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "member definition, '..' or '}'".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                });
            }
        };
        let next = tokens.next(false);

        if next == Some(Token::ClosingCurlyBracket) {
            break;
        }

        if next != Some(Token::Comma) {
            return Err(SyntaxError::ExpectedErr {
                expected: "comma (',')".to_string(),
                found: format!("{:?}: {:?}", next, tokens.slice()),
                pos: (0, 0),
            });
        }
    }
    Ok(defs)
}

fn parse_object(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<(ast::Expression, ast::Expression)>, SyntaxError> {
    let mut defs = Vec::<(ast::Expression, ast::Expression)>::new();

    match tokens.next(false) {
        Some(Token::OpenCurlyBracket) => (),
        a => {
            return Err(SyntaxError::ExpectedErr {
                expected: "'{'".to_string(),
                found: format!("{:?}: {:?}", a, tokens.slice()),
                pos: (0, 0),
            })
        }
    }

    loop {
        if tokens.next(false) == Some(Token::ClosingCurlyBracket) {
            break;
        } else {
            tokens.previous();
        }
        let key = parse_expr(tokens, notes)?;
        match tokens.next(false) {
            Some(Token::Colon) => (),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "':'".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }
        }
        let val = parse_expr(tokens, notes)?;

        defs.push((key, val));

        let next = tokens.next(false);

        if next == Some(Token::ClosingCurlyBracket) {
            break;
        }

        if next != Some(Token::Comma) {
            return Err(SyntaxError::ExpectedErr {
                expected: "comma (',')".to_string(),
                found: format!("{:?}: {:?}", next, tokens.slice()),
                pos: (0, 0),
            });
        }
    }
    Ok(defs)
}

fn parse_args(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<ast::Argument>, SyntaxError> {
    let mut args = Vec::<ast::Argument>::new();
    loop {
        if tokens.next(false) == Some(Token::ClosingBracket) {
            break;
        };
        args.push(match tokens.next(false) {
            Some(Token::Assign) => {
                // println!("assign ");
                tokens.previous();
                let symbol = Some(tokens.slice());
                tokens.next(false);
                let value = parse_expr(tokens, notes)?;
                //tokens.previous();

                ast::Argument { symbol, value }
            }

            Some(_) => {
                tokens.previous();
                tokens.previous();
                // println!("arg with no val");
                let value = parse_expr(tokens, notes)?;

                ast::Argument {
                    symbol: None,
                    value,
                }
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro arguments".to_string(),
                    pos: (0, 0),
                })
            }
        });

        match tokens.next(false) {
            Some(Token::Comma) => (),
            Some(Token::ClosingBracket) => {
                break;
            }

            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "comma (',') or ')'".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro arguments".to_string(),
                    pos: (0, 0),
                })
            }
        }
    }
    //tokens.previous();

    Ok(args)
}

fn parse_arg_def(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<(String, Option<ast::Expression>)>, SyntaxError> {
    let mut args = Vec::<(String, Option<ast::Expression>)>::new();
    loop {
        if tokens.next(false) == Some(Token::ClosingBracket) {
            break;
        };
        args.push(match tokens.next(false) {
            Some(Token::Colon) => {
                tokens.previous();
                let symbol = tokens.slice();
                tokens.next(false);
                let value = Some(parse_expr(tokens, notes)?);
                //tokens.previous();

                (symbol, value)
            }

            Some(_) => {
                tokens.previous();

                (tokens.slice(), None)
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: (0, 0),
                })
            }
        });

        match tokens.next(false) {
            Some(Token::Comma) => (),
            Some(Token::ClosingBracket) => break,

            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "comma (',') or ')'".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: (0, 0),
                })
            }
        }
    }
    //tokens.previous();

    Ok(args)
}

fn parse_variable(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<ast::Variable, SyntaxError> {
    let mut first_token = tokens.next(false);
    //println!("first token in var: {:?}, {}", first_token, tokens.slice());
    let operator = match first_token {
        Some(Token::Minus) => {
            first_token = tokens.next(false);
            Some(ast::UnaryOperator::Minus)
        }
        Some(Token::Exclamation) => {
            first_token = tokens.next(false);
            Some(ast::UnaryOperator::Not)
        }

        Some(Token::DotDot) => {
            first_token = tokens.next(false);
            Some(ast::UnaryOperator::Range)
        }
        _ => None,
    };

    let value = match first_token {
        Some(Token::Number) => ast::ValueLiteral::Number(match tokens.slice().parse() {
            Ok(n) => n,
            Err(err) => {
                //println!("{}", tokens.slice());
                return Err(SyntaxError::SyntaxError {
                    message: format!("Error when parsing number: {}", err),

                    pos: (0, 0),
                });
            }
        }),
        Some(Token::StringLiteral) => ast::ValueLiteral::Str(ast::str_content(tokens.slice())),
        Some(Token::ID) => {
            let mut text = tokens.slice();
            let class_name = match text.pop().unwrap() {
                'g' => ast::IDClass::Group,
                'c' => ast::IDClass::Color,
                'i' => ast::IDClass::Item,
                'b' => ast::IDClass::Block,
                _ => unreachable!(),
            };

            let (unspecified, number) = match text.as_ref() {
                "?" => (true, 0),
                _ => (
                    false,
                    match text.parse() {
                        Ok(n) => n,
                        Err(err) => {
                            return Err(SyntaxError::SyntaxError {
                                message: format!("Error when parsing number: {}", err),

                                pos: (0, 0),
                            });
                        }
                    },
                ),
            };

            if !unspecified {
                match class_name {
                    ast::IDClass::Group => (*notes).closed_groups.push(number),
                    ast::IDClass::Color => (*notes).closed_colors.push(number),
                    ast::IDClass::Item => (*notes).closed_items.push(number),
                    ast::IDClass::Block => (*notes).closed_blocks.push(number),
                }
            }
            ast::ValueLiteral::ID(ast::ID {
                class_name,
                unspecified,
                number,
            })
        }
        Some(Token::True) => ast::ValueLiteral::Bool(true),
        Some(Token::False) => ast::ValueLiteral::Bool(false),
        Some(Token::Null) => ast::ValueLiteral::Null,
        Some(Token::Symbol) => ast::ValueLiteral::Symbol(tokens.slice()),

        Some(Token::OpenSquareBracket) => {
            //Array
            let mut arr = Vec::new();

            if tokens.next(false) != Some(Token::ClosingSquareBracket) {
                tokens.previous();
                loop {
                    arr.push(parse_expr(tokens, notes)?);
                    match tokens.next(false) {
                        Some(Token::Comma) => {
                            //accounting for trailing comma
                            if let Some(Token::ClosingSquareBracket) = tokens.next(false) {
                                break;
                            } else {
                                tokens.previous();
                            }
                        }
                        Some(Token::ClosingSquareBracket) => break,
                        a => {
                            return Err(SyntaxError::ExpectedErr {
                                expected: "comma (',') or ']'".to_string(),
                                found: format!("{:?}: {:?}", a, tokens.slice()),
                                pos: (0, 0),
                            })
                        }
                    }
                }
            }

            ast::ValueLiteral::Array(arr)
        }

        Some(Token::Import) => ast::ValueLiteral::Import(match tokens.next(false) {
            Some(Token::StringLiteral) => PathBuf::from(ast::str_content(tokens.slice())),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "literal string".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }
        }),

        Some(Token::At) => ast::ValueLiteral::TypeIndicator(match tokens.next(false) {
            Some(Token::Symbol) => tokens.slice(),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "type name".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: (0, 0),
                })
            }
        }),

        Some(Token::OpenBracket) => {
            //Either enclosed expression or macro definition
            let parse_closed_expr = |tokens: &mut Tokens,
                                     notes: &mut ParseNotes|
             -> Result<ast::ValueLiteral, SyntaxError> {
                //expr
                let expr = ast::ValueLiteral::Expression(parse_expr(tokens, notes)?);
                //consume closing bracket
                match tokens.next(false) {
                    Some(Token::ClosingBracket) => Ok(expr),
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "')'".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: (0, 0),
                        })
                    }
                }
            };

            let parse_macro_def = |tokens: &mut Tokens,
                                   notes: &mut ParseNotes|
             -> Result<ast::ValueLiteral, SyntaxError> {
                let args = parse_arg_def(tokens, notes)?;

                let body = match tokens.next(false) {
                    Some(Token::OpenCurlyBracket) => parse_cmp_stmt(tokens, notes)?,
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "'{'".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: (0, 0),
                        })
                    }
                };

                Ok(ast::ValueLiteral::Macro(ast::Macro {
                    args,
                    body: ast::CompoundStatement { statements: body },
                }))
            };
            //tokens.next(false);
            match tokens.next(false) {
                Some(Token::ClosingBracket) => {
                    tokens.previous();
                    //tokens.previous();

                    parse_macro_def(tokens, notes)?
                }

                Some(Token::Symbol) => match tokens.next(false) {
                    Some(Token::Comma) | Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();

                        parse_macro_def(tokens, notes)?
                    }

                    Some(Token::ClosingBracket) => match tokens.next(false) {
                        Some(Token::OpenCurlyBracket) => {
                            tokens.previous();
                            tokens.previous();
                            tokens.previous();

                            parse_macro_def(tokens, notes)?
                        }
                        _ => {
                            tokens.previous();
                            tokens.previous();
                            tokens.previous();

                            parse_closed_expr(tokens, notes)?
                        }
                    },

                    _ => {
                        tokens.previous();
                        tokens.previous();

                        parse_closed_expr(tokens, notes)?
                    }
                },
                _ => {
                    tokens.previous();

                    parse_closed_expr(tokens, notes)?
                }
            }
        }
        Some(Token::OpenCurlyBracket) => {
            //either dictionary or function
            match tokens.next(false) {
                Some(Token::DotDot) => {
                    tokens.previous();
                    ast::ValueLiteral::Dictionary(parse_dict(tokens, notes)?)
                }
                _ => match tokens.next(false) {
                    Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();
                        ast::ValueLiteral::Dictionary(parse_dict(tokens, notes)?)
                    }
                    _ => {
                        tokens.previous();
                        tokens.previous();

                        ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                            statements: parse_cmp_stmt(tokens, notes)?,
                        })
                    }
                },
            }
        }

        Some(Token::Object) => ast::ValueLiteral::Obj(parse_object(tokens, notes)?),

        a => {
            return Err(SyntaxError::ExpectedErr {
                expected: "a value".to_string(),
                found: format!("{:?}: {:?}", a, tokens.slice()),
                pos: (0, 0),
            })
        }
    };

    let mut path = Vec::<ast::Path>::new();

    loop {
        match tokens.next(true) {
            Some(Token::OpenSquareBracket) => {
                let index = parse_expr(tokens, notes)?;
                match tokens.next(false) {
                    Some(Token::ClosingSquareBracket) => path.push(ast::Path::Index(index)),
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "]".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: (0, 0),
                        })
                    }
                }
            }
            Some(Token::OpenBracket) => path.push(ast::Path::Call(parse_args(tokens, notes)?)),
            Some(Token::Period) => match tokens.next(false) {
                Some(Token::Symbol) => path.push(ast::Path::Member(tokens.slice())),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "member name".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: (0, 0),
                    })
                }
            },

            _ => break,
        }
    }
    tokens.previous();

    Ok(ast::Variable {
        operator,
        value,
        path,
    })
}
