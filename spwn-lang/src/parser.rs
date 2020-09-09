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
//TODO: add type defining statement
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
    Star,

    #[token("%")]
    Modulo,

    #[token("^")]
    Power,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("/")]
    Slash,

    #[token("!")]
    Exclamation,

    #[token("=")]
    Assign,

    #[token("+=")]
    Add,
    #[token("-=")]
    Subtract,
    #[token("*=")]
    Multiply,
    #[token("/=")]
    Divide,

    //VALUES
    #[regex(r"([a-zA-Z_][a-zA-Z0-9_]*)|\$")]
    Symbol,

    #[regex(r"[0-9]+(\.[0-9]+)?")]
    Number,

    #[regex("\"[^\n\r\"]*\"")] //FIX: make it not match \"
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

    #[token("#")]
    Hash,

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

    #[token("type")]
    Type,

    #[token("let")]
    Let,

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
    //pub defined_vars: HashMap<String, usize>, //Name, num of uses
}

impl ParseNotes {
    pub fn new() -> Self {
        ParseNotes {
            closed_groups: Vec::new(),
            closed_colors: Vec::new(),
            closed_blocks: Vec::new(),
            closed_items: Vec::new(),
        }
    }
}

pub struct Tokens<'a> {
    iter: Lexer<'a, Token>,
    stack: Vec<(Token, String, core::ops::Range<usize>)>,
    line_breaks: Vec<u32>,
    //index 0 = element of iter / last element in stack
    index: usize,
}

impl<'a> Tokens<'a> {
    fn new(iter: Lexer<'a, Token>) -> Self {
        Tokens {
            iter,
            stack: Vec::new(),
            line_breaks: vec![0],
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

    fn position(&self) -> (usize, usize) {
        let file_pos = self.stack[self.stack.len() - self.index - 1].2.start;
        /*println!(
            "file pos: {}, line breaks: {:?}",
            file_pos, self.line_breaks
        );*/
        for (i, lb) in self.line_breaks.iter().enumerate() {
            let line_break = *lb as usize;
            if line_break >= file_pos {
                if i == 0 {
                    return (1, file_pos + 1);
                } else {
                    return (i + 1, file_pos - self.line_breaks[i - 1] as usize);
                }
            }
        }

        (0, file_pos)
    }

    /*fn span(&self) -> core::ops::Range<usize> {
        self.stack[self.stack.len() - self.index - 1].2.clone()
    }*/
}

//type TokenList = Peekable<Lexer<Token>>;

const STATEMENT_SEPARATOR_DESC: &str = "Statement separator (line-break or ';')";

pub fn parse_spwn(mut unparsed: String) -> Result<(Vec<ast::Statement>, ParseNotes), SyntaxError> {
    unparsed = unparsed.replace("\r\n", "\n");

    let tokens_iter = Token::lexer(&unparsed);

    let mut tokens = Tokens::new(tokens_iter);
    let mut statements = Vec::<ast::Statement>::new();

    let mut notes = ParseNotes::new();

    let mut line_breaks = Vec::<u32>::new();
    let mut current_index: u32 = 0;

    for line in unparsed.lines() {
        current_index += line.len() as u32;
        line_breaks.push(current_index);
        current_index += 1; //line break char
    }

    tokens.line_breaks = line_breaks;

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
                    pos: tokens.position(),
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
                    pos: tokens.position(),
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
                    pos: tokens.position(),
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
            if tokens.next(false) == Some(Token::Arrow) {
                //double arrow (throw error)
                return Err(SyntaxError::UnexpectedErr {
                    found: "double arrow (-> ->)".to_string(),
                    pos: tokens.position(),
                });
            }

            tokens.previous();

            let rest_of_statement = parse_statement(tokens, notes)?;

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
                        pos: tokens.position(),
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
                            pos: tokens.position(),
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
                        pos: tokens.position(),
                    })
                }

                None => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "iterator variable name".to_string(),
                        found: "None".to_string(),
                        pos: tokens.position(),
                    })
                }
            };

            match tokens.next(false) {
                Some(Token::In) => {}
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "keyword 'in'".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: tokens.position(),
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
                        pos: tokens.position(),
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

        Some(Token::Type) => match tokens.next(false) {
            Some(Token::Symbol) => ast::StatementBody::TypeDef(tokens.slice()),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "type name".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: tokens.position(),
                })
            }
        },

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
                        pos: tokens.position(),
                    })
                }
            }
        }

        Some(Token::Extract) => ast::StatementBody::Extract(parse_expr(tokens, notes)?),

        Some(Token::Let) => {
            tokens.next(false);
            let symbol = tokens.slice();

            match tokens.next(false) {
                Some(Token::Assign) => (),
                _ => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'='".to_string(),
                        found: tokens.slice(),
                        pos: tokens.position(),
                    })
                }
            };

            let value = parse_expr(tokens, notes)?;
            //tokens.next(false);
            ast::StatementBody::Definition(ast::Definition {
                symbol,
                value,
                //mutable: true,
            })
        }

        Some(_) => {
            //expression or call
            //constant statements are just assign, handled in compiler

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

        None => {
            //end of input
            unimplemented!()
        }
    };

    Ok(ast::Statement {
        body,
        arrow,
        line: tokens.position(),
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
        Token::Star => Some(ast::Operator::Star),
        Token::Power => Some(ast::Operator::Power),
        Token::Plus => Some(ast::Operator::Plus),
        Token::Minus => Some(ast::Operator::Minus),
        Token::Slash => Some(ast::Operator::Slash),
        Token::Modulo => Some(ast::Operator::Modulo),

        Token::Assign => Some(ast::Operator::Assign),
        Token::Add => Some(ast::Operator::Add),
        Token::Subtract => Some(ast::Operator::Subtract),
        Token::Multiply => Some(ast::Operator::Multiply),
        Token::Divide => Some(ast::Operator::Divide),
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
            Some(Token::Symbol) | Some(Token::Type) => {
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
                            pos: tokens.position(),
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
                    pos: tokens.position(),
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
                pos: tokens.position(),
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
                pos: tokens.position(),
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
                    pos: tokens.position(),
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
                pos: tokens.position(),
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
                    pos: tokens.position(),
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
                    pos: tokens.position(),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro arguments".to_string(),
                    pos: tokens.position(),
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
) -> Result<Vec<ast::ArgDef>, SyntaxError> {
    let mut args = Vec::<ast::ArgDef>::new();
    loop {
        let properties = check_for_tag(tokens, notes)?;
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

                (symbol, value, properties)
            }

            Some(_) => {
                tokens.previous();

                (tokens.slice(), None, properties)
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: tokens.position(),
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
                    pos: tokens.position(),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: tokens.position(),
                })
            }
        }
    }
    //tokens.previous();

    Ok(args)
}

fn check_for_tag(tokens: &mut Tokens, notes: &mut ParseNotes) -> Result<ast::Tag, SyntaxError> {
    match tokens.next(false) {
        Some(Token::Hash) => {
            //parse tag
            match tokens.next(false) {
                Some(Token::OpenSquareBracket) => (),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'['".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: tokens.position(),
                    })
                }
            };

            let mut contents = ast::Tag::new();

            loop {
                match tokens.next(false) {
                    Some(Token::ClosingSquareBracket) => break,
                    Some(Token::Symbol) => {
                        let name = tokens.slice();
                        let args = match tokens.next(false) {
                            Some(Token::OpenBracket) => parse_args(tokens, notes)?,
                            Some(Token::Comma) => Vec::new(),
                            Some(Token::ClosingSquareBracket) => break,
                            a => {
                                return Err(SyntaxError::ExpectedErr {
                                    expected: "either '(', ']' or comma (',')".to_string(),
                                    found: format!("{:?}: {:?}", a, tokens.slice()),
                                    pos: tokens.position(),
                                })
                            }
                        };
                        contents.tags.push((name, args));
                    }
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "either Symbol or ']'".to_string(),
                            found: format!("{:?}: {:?}", a, tokens.slice()),
                            pos: tokens.position(),
                        })
                    }
                };
            }

            Ok(contents)
        }
        _ => {
            tokens.previous();
            Ok(ast::Tag::new())
        }
    }
}

fn parse_variable(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<ast::Variable, SyntaxError> {
    let properties = check_for_tag(tokens, notes)?;
    let mut first_token = tokens.next(false);

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

                    pos: tokens.position(),
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

                                pos: tokens.position(),
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
                                pos: tokens.position(),
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
                    pos: tokens.position(),
                })
            }
        }),

        Some(Token::At) => ast::ValueLiteral::TypeIndicator(match tokens.next(false) {
            Some(Token::Symbol) => tokens.slice(),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "type name".to_string(),
                    found: format!("{:?}: {:?}", a, tokens.slice()),
                    pos: tokens.position(),
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
                            pos: tokens.position(),
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
                            pos: tokens.position(),
                        })
                    }
                };

                Ok(ast::ValueLiteral::Macro(ast::Macro {
                    args,
                    body: ast::CompoundStatement { statements: body },
                    properties,
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

                Some(Token::Hash) => {
                    //CHANGE THIS
                    tokens.previous();
                    //tokens.previous();

                    parse_macro_def(tokens, notes)?
                }
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
                pos: tokens.position(),
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
                            pos: tokens.position(),
                        })
                    }
                }
            }
            Some(Token::OpenBracket) => path.push(ast::Path::Call(parse_args(tokens, notes)?)),
            Some(Token::Period) => match tokens.next(false) {
                Some(Token::Symbol) | Some(Token::Type) => {
                    path.push(ast::Path::Member(tokens.slice()))
                }
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "member name".to_string(),
                        found: format!("{:?}: {:?}", a, tokens.slice()),
                        pos: tokens.position(),
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
