use crate::ast;
/*use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;*/

//use crate::builtin::TYPE_MEMBER_NAME;

//use std::collections::HashMap;
use std::path::PathBuf;

use logos::Lexer;
use logos::Logos;

use std::error::Error;
use std::fmt;

use crate::compiler::error_intro;

pub type FileRange = ((usize, usize), (usize, usize));
//TODO: add type defining statement
#[derive(Debug)]
pub enum SyntaxError {
    ExpectedErr {
        expected: String,
        found: String,
        pos: FileRange,
        file: PathBuf,
    },
    UnexpectedErr {
        found: String,
        pos: FileRange,
        file: PathBuf,
    },
    SyntaxError {
        message: String,
        pos: FileRange,
        file: PathBuf,
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
                file,
            } => write!(
                f,
                "{}SyntaxError: Expected {}, found {}",
                error_intro(*pos, file),
                expected,
                found
            ),

            SyntaxError::UnexpectedErr { found, pos, file } => write!(
                f,
                "{}SyntaxError: Unexpected {}",
                error_intro(*pos, file),
                found
            ),

            SyntaxError::SyntaxError { message, pos, file } => {
                write!(f, "{}SyntaxError: {}", error_intro(*pos, file), message)
            }
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

    #[token("as")]
    As,

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

    #[token("::")]
    DoubleColon,

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

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("obj")]
    Object,

    #[token("trigger")]
    Trigger,

    #[token("import")]
    Import,

    #[token("extract")]
    Extract,

    #[token("extend")]
    Extend,

    #[token("null")]
    Null,

    #[token("type")]
    Type,

    #[token("let")]
    Let,

    #[token("self")]
    SelfVal,

    //COMMENT
    #[regex(r"/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*")]
    Comment,

    //STATEMENT SEPARATOR
    #[regex(r"[\n\r;][ \t\f\n\r]*")]
    StatementSeparator,

    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,
}

impl Token {
    fn typ(&self) -> &'static str {
        use Token::*;
        match self {
            Arrow | Or | And | Equal | NotEqual | MoreOrEqual | LessOrEqual | MoreThan
            | LessThan | Star | Modulo | Power | Plus | Minus | Slash | Exclamation | Assign
            | Add | Subtract | Multiply | Divide | As => "operator",
            Symbol => "identifier",
            Number => "number literal",
            StringLiteral => "string literal",
            True | False => "boolean literal",
            ID => "ID literal",

            Comma | OpenCurlyBracket | ClosingCurlyBracket | OpenSquareBracket
            | ClosingSquareBracket | OpenBracket | ClosingBracket | Colon | DoubleColon
            | Period | DotDot | At | Hash => "terminator",

            Return | Implement | For | In | ErrorStatement | If | Else | Object | Trigger
            | Import | Extract | Null | Type | Let | SelfVal | Break | Continue | Extend => {
                "keyword"
            }
            Comment => "comment",
            StatementSeparator => "statement separator",
            Error => "unknown",
        }
    }
}

pub struct ParseNotes {
    pub tag: ast::Tag,
    pub file: PathBuf,
}

impl ParseNotes {
    pub fn new(path: PathBuf) -> Self {
        ParseNotes {
            tag: ast::Tag::new(),
            file: path,
        }
    }
}

#[derive(Clone)]
pub struct Tokens<'a> {
    iter: Lexer<'a, Token>,
    stack: Vec<(Option<Token>, String, core::ops::Range<usize>)>,
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
    fn next(&mut self, ss: bool, comment: bool) -> Option<Token> {
        if self.index == 0 {
            let next_element = self.iter.next();
            /* {
                Some(e) => Some(e),
                None => {
                    if ss {
                        Some(Token::StatementSeparator)
                    } else {
                        None
                    }
                }
            };*/

            let slice = self.iter.slice().to_string();
            let range = self.iter.span();
            /*if self.stack.len() > 4 {
                self.stack.remove(0);
            }*/
            self.stack.push((next_element, slice, range));

            if (!ss && next_element == Some(Token::StatementSeparator))
                || (!comment && next_element == Some(Token::Comment))
            {
                self.next(ss, comment)
            } else {
                next_element
            }
        } else {
            self.index -= 1;
            if (!ss
                && self.stack[self.stack.len() - self.index - 1].0
                    == Some(Token::StatementSeparator))
                || (!comment
                    && self.stack[self.stack.len() - self.index - 1].0 == Some(Token::Comment))
            {
                self.next(ss, comment)
            } else {
                self.stack[self.stack.len() - self.index - 1].0
            }
        }
    }
    fn previous(&mut self) -> Option<Token> {
        /*self.index += 1;
        let len = self.stack.len();
        if len > self.index {
            if self.stack[len - self.index - 1].0 == Token::StatementSeparator
                || self.stack[len - self.index - 1].0 == Token::Comment
            {
                self.previous()
            } else if len - self.index >= 1 {
                Some(self.stack[len - self.index - 1].0)
            } else {
                None
            }
        } else {
            None
        }*/
        self.previous_no_ignore(false, false)
    }

    fn previous_no_ignore(&mut self, ss: bool, comment: bool) -> Option<Token> {
        self.index += 1;
        let len = self.stack.len();
        if len > self.index {
            if (!ss && self.stack[len - self.index - 1].0 == Some(Token::StatementSeparator))
                || (!comment && self.stack[len - self.index - 1].0 == Some(Token::Comment))
            {
                self.previous_no_ignore(ss, comment)
            } else if len - self.index >= 1 {
                self.stack[len - self.index - 1].0
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
        } else if len - self.index < 1 {
            None
        } else {
            Some(self.stack[len - self.index - 1].0)
        }
    }*/

    fn slice(&self) -> String {
        self.stack[self.stack.len() - self.index - 1].1.clone()
    }

    fn position(&self) -> ((usize, usize), (usize, usize)) {
        if self.stack.len() - self.index == 0 {
            return ((1, 0), (1, 0));
        }
        let file_pos1 = self.stack[self.stack.len() - self.index - 1].2.start;
        let file_pos2 = self.stack[self.stack.len() - self.index - 1].2.end;
        /*println!(
            "file pos: {}, line breaks: {:?}",
            file_pos, self.line_breaks
        );*/
        let mut found_pos_1 = false;
        let mut found_pos_2 = false;
        let mut out = ((1, file_pos1), (1, file_pos2));

        for (i, lb) in self.line_breaks.iter().enumerate() {
            let line_break = *lb as usize;
            if !found_pos_1 && line_break >= file_pos1 {
                if i == 0 {
                    out.0 = (1, file_pos1);
                } else {
                    out.0 = (i + 1, file_pos1 - self.line_breaks[i - 1] as usize - 1);
                }
                found_pos_1 = true;
            }

            if !found_pos_2 && line_break >= file_pos2 {
                if i == 0 {
                    out.1 = (1, file_pos2);
                } else {
                    out.1 = (i + 1, file_pos2 - self.line_breaks[i - 1] as usize - 1);
                }
                found_pos_2 = true;
            }
        }

        out
    }

    /*fn abs_position(&self) -> usize {
        self.stack[self.stack.len() - self.index - 1].2.start
    }*/

    /*fn span(&self) -> core::ops::Range<usize> {
        self.stack[self.stack.len() - self.index - 1].2.clone()
    }*/
}

//type TokenList = Peekable<Lexer<Token>>;

const STATEMENT_SEPARATOR_DESC: &str = "Statement separator (line-break or ';')";

pub fn parse_spwn(
    mut unparsed: String,
    path: PathBuf,
) -> Result<(Vec<ast::Statement>, ParseNotes), SyntaxError> {
    unparsed = unparsed.replace("\r\n", "\n");

    let tokens_iter = Token::lexer(&unparsed);

    let mut tokens = Tokens::new(tokens_iter);

    /*{
        let mut test = tokens.clone();

        for _ in 0..100 {
            println!("{:?}", test.next(true, true));
        }

        /*println!(
            "{:?}",
            test.iter
                .spanned()
                .collect::<Vec<(Token, std::ops::Range<usize>)>>()
        );*/
    }*/
    let mut statements = Vec::<ast::Statement>::new();

    let mut notes = ParseNotes::new(path);

    let mut line_breaks = Vec::<u32>::new();
    let mut current_index: u32 = 0;

    for line in unparsed.lines() {
        current_index += line.len() as u32;
        line_breaks.push(current_index);
        current_index += 1; //line break char
    }

    tokens.line_breaks = line_breaks;

    let start_tag = check_for_tag(&mut tokens, &mut notes)?;
    notes.tag = start_tag;
    loop {
        //tokens.next(false, false);
        match tokens.next(false, true) {
            Some(_) => {
                tokens.previous_no_ignore(false, true);

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

        match tokens.next(true, false) {
            Some(Token::StatementSeparator) => {}
            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: STATEMENT_SEPARATOR_DESC.to_string(),
                    found: format!("{}: \"{}\"", a.typ(), tokens.slice()),
                    pos: tokens.position(),
                    file: notes.file,
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
        match tokens.next(false, true) {
            Some(Token::ClosingCurlyBracket) => break,
            Some(_) => {
                tokens.previous_no_ignore(false, true);
                statements.push(parse_statement(tokens, notes)?);
                //println!("statement done");
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing a closure".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        }

        match tokens.next(true, false) {
            Some(Token::StatementSeparator) => {}
            Some(Token::ClosingCurlyBracket) => break,
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: STATEMENT_SEPARATOR_DESC.to_string(),
                    found: format!(
                        "{}: \"{}\"",
                        match a {
                            Some(t) => t.typ(),
                            None => "EOF",
                        },
                        tokens.slice()
                    ),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                });
            }
        }
    }
    //tokens.next(false, false);
    Ok(statements)
}

pub fn parse_statement(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<ast::Statement, SyntaxError> {
    let preceding_comment = check_for_comment(tokens);

    let first = tokens.next(false, false);

    let (start_pos, _) = tokens.position();

    let mut arrow = false;
    let body = match first {
        Some(Token::Arrow) => {
            //parse async statement
            if tokens.next(false, false) == Some(Token::Arrow) {
                //double arrow (throw error)
                return Err(SyntaxError::UnexpectedErr {
                    found: "double arrow (-> ->)".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                });
            }

            tokens.previous();

            let rest_of_statement = parse_statement(tokens, notes)?;

            arrow = true;
            rest_of_statement.body
        }

        Some(Token::Extend) => {
            let expr = parse_expr(tokens, notes, true, true)?;
            let next = tokens.next(false, false);
            if next != Some(Token::OpenCurlyBracket) {
                return Err(SyntaxError::ExpectedErr {
                    expected: "'{'".to_string(),
                    found: if let Some(t) = next {
                        t.typ().to_string()
                    } else {
                        String::from("end of file")
                    },
                    pos: tokens.position(),
                    file: notes.file.clone(),
                });
            }
            let cmp = parse_cmp_stmt(tokens, notes)?;
            ast::StatementBody::Extend(expr, ast::CompoundStatement { statements: cmp })
        }

        Some(Token::Return) => {
            //parse return statement

            match tokens.next(true, false) {
                Some(Token::StatementSeparator) | Some(Token::ClosingCurlyBracket) => {
                    tokens.previous();
                    ast::StatementBody::Return(None)
                }

                _ => {
                    tokens.previous();
                    ast::StatementBody::Return(Some(parse_expr(tokens, notes, true, true)?))
                }
            }
        }

        Some(Token::Break) => ast::StatementBody::Break,

        Some(Token::If) => {
            //parse if statement

            // println!("if statement");

            let condition = parse_expr(tokens, notes, true, true)?;
            match tokens.next(false, false) {
                Some(Token::OpenCurlyBracket) => (),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            }
            let if_body = parse_cmp_stmt(tokens, notes)?;
            let else_body = match tokens.next(false, false) {
                Some(Token::Else) => match tokens.next(false, false) {
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
                            found: format!(
                                "{}: \"{}\"",
                                match a {
                                    Some(t) => t.typ(),
                                    None => "EOF",
                                },
                                tokens.slice()
                            ),
                            pos: tokens.position(),
                            file: notes.file.clone(),
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

            let symbol = match tokens.next(false, false) {
                Some(Token::Symbol) => tokens.slice(),
                Some(a) => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "iterator variable name".to_string(),
                        found: format!("{}: \"{}\"", a.typ(), tokens.slice()),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }

                None => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "iterator variable name".to_string(),
                        found: "None".to_string(),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            };

            match tokens.next(false, false) {
                Some(Token::In) => {}
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "keyword 'in'".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            };

            let array = parse_expr(tokens, notes, true, true)?;
            match tokens.next(false, false) {
                Some(Token::OpenCurlyBracket) => {}
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            };
            let mut body = parse_cmp_stmt(tokens, notes)?;

            //fix confusing gd behavior
            if body
                .iter()
                .all(|x| matches!(x.body, ast::StatementBody::Call(_)))
            {
                //maybe not the fastest way, but the syntax tree is just too large to just paste in
                let new_statement = parse_spwn(
                    String::from(
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
                    ),
                    PathBuf::new(),
                )?;

                body.push(new_statement.0[0].clone());
            }

            ast::StatementBody::For(ast::For {
                symbol,
                array,
                body,
            })
        }

        Some(Token::ErrorStatement) => ast::StatementBody::Error(ast::Error {
            message: parse_expr(tokens, notes, true, true)?,
        }),

        Some(Token::Type) => match tokens.next(false, false) {
            Some(Token::Symbol) => ast::StatementBody::TypeDef(tokens.slice()),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "type name".to_string(),
                    found: format!(
                        "{}: \"{}\"",
                        match a {
                            Some(t) => t.typ(),
                            None => "EOF",
                        },
                        tokens.slice()
                    ),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        },

        Some(Token::Implement) => {
            //parse impl statement
            let symbol = parse_variable(tokens, notes, true)?;
            match tokens.next(false, false) {
                Some(Token::OpenCurlyBracket) => ast::StatementBody::Impl(ast::Implementation {
                    symbol,
                    members: parse_dict(tokens, notes)?,
                }),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'{'".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            }
        }

        Some(Token::Extract) => ast::StatementBody::Extract(parse_expr(tokens, notes, true, true)?),

        /*Some(Token::Let) => {
            tokens.next(false, false);
            let symbol = tokens.slice();

            match tokens.next(false, false) {
                Some(Token::Assign) => (),
                _ => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'='".to_string(),
                        found: tokens.slice(),
                        pos: tokens.position(),
                    })
                }
            };

            /*(*notes)
            .defined_vars
            .insert(symbol.clone(), tokens.abs_position());*/

            let value = parse_expr(tokens, notes, true, true)?;

            ast::StatementBody::Definition(ast::Definition {
                symbol,
                value,
                //mutable: true,
            })
        }*/
        Some(_) => {
            //either expression, call or definition, FIGURE OUT
            //parse it

            //expression or call
            //tokens.previous();
            tokens.previous_no_ignore(false, true);
            let expr = parse_expr(tokens, notes, true, true)?;
            if tokens.next(false, false) == Some(Token::Exclamation) {
                //call
                // println!("found call");
                ast::StatementBody::Call(ast::Call {
                    function: expr.values[0].clone(),
                })
            } else {
                //expression statement
                // println!("found expr");
                tokens.previous_no_ignore(false, true);
                ast::StatementBody::Expr(expr)
            }
        }

        None => {
            //end of input
            unimplemented!()
        }
    };
    let (_, end_pos) = tokens.position();

    let comment_after = check_for_comment(tokens);
    /*println!(
        "current token after stmt post comment: {}: ",
        tokens.slice()
    );*/

    Ok(ast::Statement {
        body,
        arrow,
        pos: (start_pos, end_pos),
        comment: (preceding_comment, comment_after),
    })
}

fn check_for_comment(tokens: &mut Tokens) -> Option<String> {
    let mut comment_found = false;
    //let mut line_break_start = false;

    let mut result = String::new();
    /*match tokens.next(false, true) {
        Some(Token::Comment) => tokens.slice(),
        //Some(Token::StatementSeparator) => String::from("\r\n"),
        _ => {
            //println!("comment not found: \"{}\"", tokens.slice());
            tokens.previous_no_ignore(false, true);

            return None;
        }
    };*/

    loop {
        result += &match tokens.next(true, true) {
            Some(Token::Comment) => {
                comment_found = true;
                tokens.slice()
            }
            Some(Token::StatementSeparator) => {
                if !result.ends_with('\n') {
                    String::from("\r\n")
                } else {
                    String::new()
                }
            }
            _ => {
                tokens.previous_no_ignore(false, true);

                break;
            }
        };
    }

    if comment_found {
        Some(result)
    } else {
        None
    }
}

fn operator_precedence(op: &ast::Operator) -> u8 {
    use ast::Operator::*;
    match op {
        As => 9,
        Power => 8,

        Modulo => 7,
        Star => 7,
        Slash => 7,

        Plus => 6,
        Minus => 6,

        Range => 5,

        MoreOrEqual => 4,
        LessOrEqual => 4,
        More => 3,
        Less => 3,

        Equal => 2,
        NotEqual => 2,

        Or => 1,
        And => 1,

        Assign => 0,
        Add => 0,
        Subtract => 0,
        Multiply => 0,
        Divide => 0,
    }
}

fn fix_precedence(mut expr: ast::Expression) -> ast::Expression {
    if expr.operators.len() <= 1 {
        expr
    } else {
        let mut lowest = 10;

        for op in &expr.operators {
            let p = operator_precedence(op);
            if p < lowest {
                lowest = p
            };
        }

        let mut new_expr = ast::Expression {
            operators: Vec::new(),
            values: Vec::new(),
        };

        loop {
            let mut didnt_break = true;
            for (i, op) in expr.operators.iter().enumerate() {
                if operator_precedence(op) == lowest {
                    new_expr.operators.push(*op);
                    new_expr.values.push(if i == 0 {
                        expr.values[0].clone()
                    } else {
                        fix_precedence(ast::Expression {
                            operators: expr.operators[..i].to_vec(),
                            values: expr.values[..(i + 1)].to_vec(),
                        })
                        .to_variable()
                    });

                    expr = ast::Expression {
                        operators: expr.operators[(i + 1)..].to_vec(),
                        values: expr.values[(i + 1)..].to_vec(),
                    };
                    didnt_break = false;
                    break;
                }
            }
            //println!("{:?}", expr);
            if didnt_break || expr.operators.is_empty() {
                break;
            }
        }

        new_expr.values.push(expr.to_variable());

        new_expr
    }
}

fn parse_expr(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
    allow_mut_op: bool,
    check_for_comments: bool,
) -> Result<ast::Expression, SyntaxError> {
    let mut values = Vec::<ast::Variable>::new();
    let mut operators = Vec::<ast::Operator>::new();

    values.push(parse_variable(tokens, notes, check_for_comments)?);
    loop {
        let op = match tokens.next(false, false) {
            Some(t) => match parse_operator(&t) {
                Some(o) => {
                    if allow_mut_op {
                        o
                    } else {
                        match o {
                            ast::Operator::Assign
                            | ast::Operator::Add
                            | ast::Operator::Subtract
                            | ast::Operator::Multiply
                            | ast::Operator::Divide => break,
                            _ => o,
                        }
                    }
                }
                None => break,
            },
            None => break,
        };

        operators.push(op);
        values.push(parse_variable(tokens, notes, check_for_comments)?);
    }
    tokens.previous_no_ignore(false, true);

    Ok(fix_precedence(ast::Expression { values, operators }))
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
        Token::As => Some(ast::Operator::As),
        _ => None,
    }
}

fn parse_dict(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
) -> Result<Vec<ast::DictDef>, SyntaxError> {
    let mut defs = Vec::<ast::DictDef>::new();

    loop {
        match tokens.next(false, false) {
            Some(Token::Symbol) | Some(Token::Type) => {
                let symbol = tokens.slice();

                match tokens.next(false, false) {
                    Some(Token::Colon) => {
                        let expr = parse_expr(tokens, notes, true, true)?;
                        defs.push(ast::DictDef::Def((symbol, expr)));
                    }

                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "':'".to_string(),
                            found: format!(
                                "{}: \"{}\"",
                                match a {
                                    Some(t) => t.typ(),
                                    None => "EOF",
                                },
                                tokens.slice()
                            ),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        });
                    }
                }
            }

            Some(Token::DotDot) => {
                let expr = parse_expr(tokens, notes, true, true)?;
                defs.push(ast::DictDef::Extract(expr))
            }

            Some(Token::ClosingCurlyBracket) => break,

            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "member definition, '..' or '}'".to_string(),
                    found: format!(
                        "{}: \"{}\"",
                        match a {
                            Some(t) => t.typ(),
                            None => "EOF",
                        },
                        tokens.slice()
                    ),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                });
            }
        };
        let next = tokens.next(false, false);

        if next == Some(Token::ClosingCurlyBracket) {
            break;
        }

        if next != Some(Token::Comma) {
            return Err(SyntaxError::ExpectedErr {
                expected: "comma (',')".to_string(),
                found: format!("{:?}: {:?}", next, tokens.slice()),
                pos: tokens.position(),
                file: notes.file.clone(),
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

    match tokens.next(false, false) {
        Some(Token::OpenCurlyBracket) => (),
        a => {
            return Err(SyntaxError::ExpectedErr {
                expected: "'{'".to_string(),
                found: format!(
                    "{}: \"{}\"",
                    match a {
                        Some(t) => t.typ(),
                        None => "EOF",
                    },
                    tokens.slice()
                ),
                pos: tokens.position(),
                file: notes.file.clone(),
            })
        }
    }

    loop {
        if tokens.next(false, false) == Some(Token::ClosingCurlyBracket) {
            break;
        } else {
            tokens.previous();
        }
        let key = parse_expr(tokens, notes, true, true)?;
        match tokens.next(false, false) {
            Some(Token::Colon) => (),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "':'".to_string(),
                    found: format!(
                        "{}: \"{}\"",
                        match a {
                            Some(t) => t.typ(),
                            None => "EOF",
                        },
                        tokens.slice()
                    ),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        }
        let val = parse_expr(tokens, notes, true, true)?;

        defs.push((key, val));

        let next = tokens.next(false, false);

        if next == Some(Token::ClosingCurlyBracket) {
            break;
        }

        if next != Some(Token::Comma) {
            return Err(SyntaxError::ExpectedErr {
                expected: "comma (',')".to_string(),
                found: format!("{:?}: {:?}", next, tokens.slice()),
                pos: tokens.position(),
                file: notes.file.clone(),
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
        if tokens.next(false, false) == Some(Token::ClosingBracket) {
            break;
        };

        args.push(match tokens.next(false, false) {
            Some(Token::Assign) => {
                // println!("assign ");
                match tokens.previous() {
                    Some(Token::Symbol) => (),
                    Some(a) => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "Argument name".to_string(),
                            found: format!("{}: \"{}\"", a.typ(), tokens.slice()),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        })
                    }

                    None => unreachable!(),
                };
                let symbol = Some(tokens.slice());
                tokens.next(false, false);
                let value = parse_expr(tokens, notes, true, true)?;
                //tokens.previous();

                ast::Argument { symbol, value }
            }

            Some(_) => {
                tokens.previous();
                tokens.previous();
                // println!("arg with no val");

                let value = parse_expr(tokens, notes, true, true)?;

                ast::Argument {
                    symbol: None,
                    value,
                }
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro arguments".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        });

        match tokens.next(false, false) {
            Some(Token::Comma) => (),
            Some(Token::ClosingBracket) => {
                break;
            }

            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "comma (',') or ')'".to_string(),
                    found: format!("{}: \"{}\"", a.typ(), tokens.slice()),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro arguments".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
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
        if tokens.next(false, false) == Some(Token::ClosingBracket) {
            break;
        };
        args.push(match tokens.next(false, false) {
            Some(Token::Assign) => {
                if tokens.previous() == Some(Token::SelfVal) {
                    return Err(SyntaxError::SyntaxError {
                        message: "\"self\" argument cannot have a default value".to_string(),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    });
                }
                let symbol = tokens.slice();
                tokens.next(false, false);
                let value = Some(parse_expr(tokens, notes, true, true)?);
                //tokens.previous();

                (symbol, value, properties, None)
            }

            Some(Token::Colon) => {
                if tokens.previous() == Some(Token::SelfVal) {
                    return Err(SyntaxError::SyntaxError {
                        message: "\"self\" argument cannot have explicit type".to_string(),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    });
                }
                let symbol = tokens.slice();
                tokens.next(false, false);
                let type_value = Some(parse_expr(tokens, notes, false, true)?);
                //tokens.previous();

                match tokens.next(false, false) {
                    Some(Token::Assign) => {
                        let value = Some(parse_expr(tokens, notes, true, true)?);

                        //tokens.previous();

                        (symbol, value, properties, type_value)
                    }
                    Some(_) => {
                        tokens.previous();

                        (symbol, None, properties, type_value)
                    }
                    None => {
                        return Err(SyntaxError::SyntaxError {
                            message: "File ended while parsing macro signature".to_string(),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        })
                    }
                }
            }

            Some(_) => {
                if tokens.previous() == Some(Token::SelfVal) && !args.is_empty() {
                    return Err(SyntaxError::SyntaxError {
                        message: "\"self\" argument must be the first argument".to_string(),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    });
                }

                (tokens.slice(), None, properties, None)
            }
            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        });

        match tokens.next(false, false) {
            Some(Token::Comma) => (),
            Some(Token::ClosingBracket) => break,

            Some(a) => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "comma (',') or ')'".to_string(),
                    found: format!("{}: \"{}\"", a.typ(), tokens.slice()),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }

            None => {
                return Err(SyntaxError::SyntaxError {
                    message: "File ended while parsing macro signature".to_string(),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        }
    }
    //tokens.previous();

    Ok(args)
}

fn check_for_tag(tokens: &mut Tokens, notes: &mut ParseNotes) -> Result<ast::Tag, SyntaxError> {
    let first = tokens.next(false, false);

    match first {
        Some(Token::Hash) => {
            //parse tag
            match tokens.next(false, false) {
                Some(Token::OpenSquareBracket) => (),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "'['".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            };

            let mut contents = ast::Tag::new();

            loop {
                match tokens.next(false, false) {
                    Some(Token::ClosingSquareBracket) => break,
                    Some(Token::Symbol) => {
                        let name = tokens.slice();
                        let args = match tokens.next(false, false) {
                            Some(Token::OpenBracket) => parse_args(tokens, notes)?,
                            Some(Token::Comma) => Vec::new(),
                            Some(Token::ClosingSquareBracket) => {
                                contents.tags.push((name, Vec::new()));
                                break;
                            }
                            a => {
                                return Err(SyntaxError::ExpectedErr {
                                    expected: "either '(', ']' or comma (',')".to_string(),
                                    found: format!(
                                        "{}: \"{}\"",
                                        match a {
                                            Some(t) => t.typ(),
                                            None => "EOF",
                                        },
                                        tokens.slice()
                                    ),
                                    pos: tokens.position(),
                                    file: notes.file.clone(),
                                })
                            }
                        };
                        contents.tags.push((name, args));
                    }
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "either Symbol or ']'".to_string(),
                            found: format!(
                                "{}: \"{}\"",
                                match a {
                                    Some(t) => t.typ(),
                                    None => "EOF",
                                },
                                tokens.slice()
                            ),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        })
                    }
                };
            }

            Ok(contents)
        }
        _ => {
            tokens.previous_no_ignore(false, true);
            Ok(ast::Tag::new())
        }
    }
}

fn parse_variable(
    tokens: &mut Tokens,
    notes: &mut ParseNotes,
    check_for_comments: bool,
) -> Result<ast::Variable, SyntaxError> {
    let preceding_comment = if check_for_comments {
        check_for_comment(tokens)
    } else {
        None
    };

    let properties = check_for_tag(tokens, notes)?;

    /*if tokens.stack.len() - tokens.index > 0 {
        println!("current token after val pre comment: {}: ", tokens.slice());
    }*/
    let mut first_token = tokens.next(false, false);
    let (start_pos, _) = tokens.position();

    let operator = match first_token {
        Some(Token::Minus) => {
            first_token = tokens.next(false, false);
            Some(ast::UnaryOperator::Minus)
        }
        Some(Token::Exclamation) => {
            first_token = tokens.next(false, false);
            Some(ast::UnaryOperator::Not)
        }

        Some(Token::DotDot) => {
            first_token = tokens.next(false, false);
            Some(ast::UnaryOperator::Range)
        }

        Some(Token::Let) => {
            first_token = tokens.next(false, false);
            Some(ast::UnaryOperator::Let)
        }
        _ => None,
    };

    let value = match first_token {
        Some(Token::Number) => ast::ValueBody::Number(match tokens.slice().parse() {
            Ok(n) => n,
            Err(err) => {
                //println!("{}", tokens.slice());
                return Err(SyntaxError::SyntaxError {
                    message: format!("Error when parsing number: {}", err),

                    pos: tokens.position(),
                    file: notes.file.clone(),
                });
            }
        }),
        Some(Token::StringLiteral) => ast::ValueBody::Str(ast::str_content(tokens.slice())),
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
                                file: notes.file.clone(),
                            });
                        }
                    },
                ),
            };

            ast::ValueBody::ID(ast::ID {
                class_name,
                unspecified,
                number,
            })
        }
        Some(Token::True) => ast::ValueBody::Bool(true),
        Some(Token::False) => ast::ValueBody::Bool(false),
        Some(Token::Null) => ast::ValueBody::Null,
        Some(Token::SelfVal) => ast::ValueBody::SelfVal,
        Some(Token::Symbol) => ast::ValueBody::Symbol(tokens.slice()),

        Some(Token::OpenSquareBracket) => {
            //Array
            let mut arr = Vec::new();

            if tokens.next(false, false) != Some(Token::ClosingSquareBracket) {
                tokens.previous();
                loop {
                    arr.push(parse_expr(tokens, notes, true, true)?);
                    match tokens.next(false, false) {
                        Some(Token::Comma) => {
                            //accounting for trailing comma
                            if let Some(Token::ClosingSquareBracket) = tokens.next(false, false) {
                                break;
                            } else {
                                tokens.previous();
                            }
                        }
                        Some(Token::ClosingSquareBracket) => break,
                        a => {
                            return Err(SyntaxError::ExpectedErr {
                                expected: "comma (',') or ']'".to_string(),
                                found: format!(
                                    "{}: \"{}\"",
                                    match a {
                                        Some(t) => t.typ(),
                                        None => "EOF",
                                    },
                                    tokens.slice()
                                ),
                                pos: tokens.position(),
                                file: notes.file.clone(),
                            })
                        }
                    }
                }
            }

            ast::ValueBody::Array(arr)
        }

        Some(Token::Import) => ast::ValueBody::Import(match tokens.next(false, false) {
            Some(Token::StringLiteral) => PathBuf::from(ast::str_content(tokens.slice())),
            a => {
                return Err(SyntaxError::ExpectedErr {
                    expected: "literal string".to_string(),
                    found: format!(
                        "{}: \"{}\"",
                        match a {
                            Some(t) => t.typ(),
                            None => "EOF",
                        },
                        tokens.slice()
                    ),
                    pos: tokens.position(),
                    file: notes.file.clone(),
                })
            }
        }),

        Some(Token::At) => {
            let type_name = match tokens.next(false, false) {
                Some(Token::Symbol) => tokens.slice(),
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "type name".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            };

            ast::ValueBody::TypeIndicator(type_name)
        }

        Some(Token::OpenBracket) => {
            let parse_macro_def = |tokens: &mut Tokens,
                                   notes: &mut ParseNotes|
             -> Result<ast::ValueBody, SyntaxError> {
                let args = parse_arg_def(tokens, notes)?;

                let body = match tokens.next(false, false) {
                    Some(Token::OpenCurlyBracket) => parse_cmp_stmt(tokens, notes)?,
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "'{'".to_string(),
                            found: format!(
                                "{}: \"{}\"",
                                match a {
                                    Some(t) => t.typ(),
                                    None => "EOF",
                                },
                                tokens.slice()
                            ),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        })
                    }
                };

                Ok(ast::ValueBody::Macro(ast::Macro {
                    args,
                    body: ast::CompoundStatement { statements: body },
                    properties,
                }))
            };

            let mut test_tokens = tokens.clone();

            match parse_expr(&mut test_tokens, notes, true, true) {
                Ok(expr) => {
                    //macro def
                    match test_tokens.next(false, false) {
                        Some(Token::ClosingBracket) => match test_tokens.next(false, false) {
                            Some(Token::OpenCurlyBracket) => parse_macro_def(tokens, notes)?,
                            _ => {
                                test_tokens.previous();
                                (*tokens) = test_tokens;
                                ast::ValueBody::Expression(expr)
                            }
                        },
                        Some(Token::Comma) => parse_macro_def(tokens, notes)?,
                        Some(Token::Colon) => parse_macro_def(tokens, notes)?,
                        a => {
                            return Err(SyntaxError::ExpectedErr {
                                expected: "')', ':' or comma (',')".to_string(),
                                found: format!("{:?}: {:?}", a, test_tokens.slice()),
                                pos: tokens.position(),
                                file: notes.file.clone(),
                            })
                        }
                    }
                }

                Err(_) => match parse_macro_def(tokens, notes) {
                    Ok(mac) => mac,
                    Err(e) => return Err(e),
                },
            }

            /*
            //Either enclosed expression or macro definition
            let parse_closed_expr = |tokens: &mut Tokens,
                                     notes: &mut ParseNotes|
             -> Result<ast::ValueBody, SyntaxError> {
                //expr
                let expr = ast::ValueBody::Expression(parse_expr(tokens, notes)?);
                //consume closing bracket
                match tokens.next(false, false) {
                    Some(Token::ClosingBracket) => Ok(expr),
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "')'".to_string(),
                            found: format!("{}: \"{}\"", match a {Some(t) => t.typ(), None => "EOF"}, tokens.slice()),
                            pos: tokens.position(),
                        })
                    }
                }
            };

            let parse_macro_def = |tokens: &mut Tokens,
                                   notes: &mut ParseNotes|
             -> Result<ast::ValueBody, SyntaxError> {
                let args = parse_arg_def(tokens, notes)?;

                let body = match tokens.next(false, false) {
                    Some(Token::OpenCurlyBracket) => parse_cmp_stmt(tokens, notes)?,
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "'{'".to_string(),
                            found: format!("{}: \"{}\"", match a {Some(t) => t.typ(), None => "EOF"}, tokens.slice()),
                            pos: tokens.position(),
                        })
                    }
                };

                Ok(ast::ValueBody::Macro(ast::Macro {
                    args,
                    body: ast::CompoundStatement { statements: body },
                    properties,
                }))
            };
            //tokens.next(false, false);
            match tokens.next(false, false) {
                Some(Token::ClosingBracket) => {
                    tokens.previous();
                    //tokens.previous();

                    parse_macro_def(tokens, notes)?
                }

                Some(Token::Symbol) => match tokens.next(false, false) {
                    Some(Token::Comma) | Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();

                        parse_macro_def(tokens, notes)?
                    }

                    Some(Token::ClosingBracket) => match tokens.next(false, false) {
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
            */
        }
        Some(Token::OpenCurlyBracket) => {
            //either dictionary or function
            match tokens.next(false, false) {
                Some(Token::DotDot) => {
                    tokens.previous();
                    ast::ValueBody::Dictionary(parse_dict(tokens, notes)?)
                }
                _ => match tokens.next(false, false) {
                    Some(Token::Colon) => {
                        tokens.previous();
                        tokens.previous();
                        ast::ValueBody::Dictionary(parse_dict(tokens, notes)?)
                    }
                    _ => {
                        tokens.previous();
                        tokens.previous();

                        ast::ValueBody::CmpStmt(ast::CompoundStatement {
                            statements: parse_cmp_stmt(tokens, notes)?,
                        })
                    }
                },
            }
        }

        Some(Token::Object) => ast::ValueBody::Obj(ast::ObjectLiteral {
            props: parse_object(tokens, notes)?,
            mode: ast::ObjectMode::Object,
        }),

        Some(Token::Trigger) => ast::ValueBody::Obj(ast::ObjectLiteral {
            props: parse_object(tokens, notes)?,
            mode: ast::ObjectMode::Trigger,
        }),

        a => {
            return Err(SyntaxError::ExpectedErr {
                expected: "a value".to_string(),
                found: format!(
                    "{}: \"{}\"",
                    match a {
                        Some(t) => t.typ(),
                        None => "EOF",
                    },
                    tokens.slice()
                ),
                pos: tokens.position(),
                file: notes.file.clone(),
            })
        }
    };

    let mut path = Vec::<ast::Path>::new();

    loop {
        match tokens.next(true, false) {
            Some(Token::OpenSquareBracket) => {
                let index = parse_expr(tokens, notes, true, true)?;
                match tokens.next(false, false) {
                    Some(Token::ClosingSquareBracket) => path.push(ast::Path::Index(index)),
                    a => {
                        return Err(SyntaxError::ExpectedErr {
                            expected: "]".to_string(),
                            found: format!(
                                "{}: \"{}\"",
                                match a {
                                    Some(t) => t.typ(),
                                    None => "EOF",
                                },
                                tokens.slice()
                            ),
                            pos: tokens.position(),
                            file: notes.file.clone(),
                        })
                    }
                }
            }
            Some(Token::OpenBracket) => path.push(ast::Path::Call(parse_args(tokens, notes)?)),
            Some(Token::Period) => match tokens.next(false, false) {
                Some(Token::Symbol) | Some(Token::Type) => {
                    path.push(ast::Path::Member(tokens.slice()))
                }
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "member name".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            },

            Some(Token::DoubleColon) => match tokens.next(false, false) {
                Some(Token::Symbol) | Some(Token::Type) => {
                    path.push(ast::Path::Associated(tokens.slice()))
                }
                Some(Token::OpenCurlyBracket) => {
                    path.push(ast::Path::Constructor(parse_dict(tokens, notes)?))
                }
                a => {
                    return Err(SyntaxError::ExpectedErr {
                        expected: "associated member name".to_string(),
                        found: format!(
                            "{}: \"{}\"",
                            match a {
                                Some(t) => t.typ(),
                                None => "EOF",
                            },
                            tokens.slice()
                        ),
                        pos: tokens.position(),
                        file: notes.file.clone(),
                    })
                }
            },

            _ => break,
        }
    }
    tokens.previous_no_ignore(false, true);

    let (_, end_pos) = tokens.position();

    let comment_after = if check_for_comments {
        check_for_comment(tokens)
    } else {
        None
    };

    /*if tokens.stack.len() - tokens.index > 0 {
        println!("current token after val post comment: {}: ", tokens.slice());
    }*/

    Ok(ast::Variable {
        operator,
        value: ast::ValueLiteral { body: value },
        pos: (start_pos, end_pos),
        comment: (preceding_comment, comment_after),
        path,
    })
}
