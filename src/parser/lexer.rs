use std::str::Chars;
use std::{ops::Range, path::PathBuf};

use base64;
use lasso::{Rodeo, Spur};
use logos::Logos;

use super::ast::{ASTData, ExprKey, Expression};

const INVALID_CHARACTER: char = '\u{FFFD}'; // `�`

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(subpattern digits = r#"(\d)([\d_])*"#)]
pub enum Token {
    #[regex(r#"(0[b])(?&digits)"#)]
    Int,
    #[regex(r#"(?&digits)(\.[\d_])*"#)]
    Float,

    #[regex(r#""\w*((?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*')"#)]
    String,

    #[token("let")]
    Let,
    #[token("mut")]
    Mut,

    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,

    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    #[token("type")]
    TypeDef,
    #[token("impl")]
    Impl,

    #[token("print")]
    Print,
    #[token("split")]
    Split,

    #[token("is")]
    Is,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mult,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token("^")]
    Pow,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    MultEq,
    #[token("/=")]
    DivEq,
    #[token("%=")]
    ModEq,
    #[token("^=")]
    PowEq,

    #[token(";")]
    Eol,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LSqBracket,
    #[token("]")]
    RSqBracket,
    #[token("{")]
    LBracket,
    #[token("}")]
    RBracket,
    #[token("!{")]
    TrigFnBracket,

    #[token(",")]
    Comma,

    #[token("==")]
    Eq,
    #[token("!=")]
    NotEq,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEq,
    #[token("<")]
    Lesser,
    #[token("<=")]
    LesserEq,

    #[token("=")]
    Assign,

    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,

    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    #[token("?")]
    QMark,
    #[token("!")]
    ExclMark,

    #[regex(r"@[a-zA-Z_]\w*")]
    TypeIndicator,

    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*")]
    Ident,

    #[regex(r"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    #[error]
    Error,

    Eof,
}

impl Token {
    // used in error messages
    pub fn tok_name(&self) -> String {
        match self {
            Token::Int => "int",
            Token::Float => "float",
            Token::String => "string",
            Token::TypeIndicator => "type indicator",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident => "identifier",
            Token::Error => "invalid",
            Token::Eof => "end of file",
            Token::True => "true",
            Token::False => "false",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Mult => "*",
            Token::Div => "/",
            Token::Mod => "%",
            Token::Pow => "^",
            Token::PlusEq => "+=",
            Token::MinusEq => "-=",
            Token::MultEq => "*=",
            Token::DivEq => "/=",
            Token::ModEq => "%=",
            Token::PowEq => "^=",
            Token::Assign => "=",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LSqBracket => "[",
            Token::RSqBracket => "]",
            Token::LBracket => "{",
            Token::RBracket => "}",
            Token::TrigFnBracket => "!{",
            Token::Comma => ",",
            Token::Eol => "end of line",
            Token::If => "if",
            Token::Else => "else",
            Token::While => "while",
            Token::For => "for",
            Token::In => "in",
            Token::Return => "return",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Print => "print",
            Token::Split => "split",
            Token::Is => "is",
            Token::Eq => "==",
            Token::NotEq => "!=",
            Token::Greater => ">",
            Token::GreaterEq => ">=",
            Token::Lesser => "<",
            Token::LesserEq => "<=",
            Token::Colon => ":",
            Token::DoubleColon => "::",
            Token::FatArrow => "=>",
            Token::Arrow => "->",
            Token::QMark => "?",
            Token::ExclMark => "!",
            Token::TypeDef => "type",
            Token::Impl => "impl",
            //Token::StringFlag(v) => v,
        }
        .into()
    }
    // also used in error messages
    pub fn tok_typ(&self) -> &str {
        use Token::*;
        match self {
            Plus | Minus | Mult | Div | Mod | Pow | PlusEq | MinusEq | MultEq | DivEq | ModEq
            | PowEq | Assign | Eq | NotEq | Greater | GreaterEq | Lesser | LesserEq | Is => {
                "operator"
            }

            Int | Float | String | True | False => "literal",

            Ident => "identifier",

            Let | Mut | For | While | If | Else | In | Return | Break | Continue | TypeDef
            | Impl | Print | Split => "keyword",

            Error => "",
            TypeIndicator => "type indicator",

            LParen | RParen | RSqBracket | LSqBracket | RBracket | LBracket | Comma | Colon
            | DoubleColon | FatArrow | Arrow | QMark | ExclMark | Eol | Eof | TrigFnBracket => {
                "terminator"
            }
        }
    }
}

pub type Span = (usize, usize);
pub type Tokens = Vec<(Token, Span)>;

const EOL: &[Token] = &[Token::Eol];

pub fn lex(code: String) -> Tokens {
    let mut tokens_iter = Token::lexer(&code);

    let mut tokens = vec![];
    while let Some(t) = tokens_iter.next() {
        tokens.push((t, (tokens_iter.span().start, tokens_iter.span().end)))
    }
    tokens.push((Token::Eof, (code.len(), code.len() + 1)));

    tokens
}

#[derive(Clone)]
pub struct Lexer {
    tokens: logos::Lexer<'static, Token>,
    file: Option<PathBuf>,
}

use super::error::SyntaxError;
use crate::sources::CodeArea;
impl Lexer {
    pub fn new<S: AsRef<str>, P: Into<PathBuf>>(src: S, file: Option<P>) -> Self {
        let src = unsafe { Lexer::make_static(src.as_ref()) };

        let file = file.map(|p| p.into());
        let tokens = logos::Lexer::new(src);

        Lexer { tokens, file }
    }

    pub fn next(&mut self) -> Option<Spanned<Token>> {
        let next_token = self.tokens.next()?;
        let span = self.tokens.span();

        Some(Spanned::<Token> {
            data: next_token,
            span: span.into(),
        })
    }

    pub fn peek(&mut self) -> Option<Spanned<Token>> {
        let tokens = self.clone();
        tokens.next()
    }
    pub fn peek_n(&mut self, n: u32) -> Option<Spanned<Token>> {
        let tokens = self.clone();
        for _ in n {
            tokens.next()
        }
    }

    pub fn expected_err(
        &mut self,
        expected: String,
        found: Option<Spanned<Token>>,
    ) -> crate::error::Error {
        SyntaxError::Expected {
            expected,
            found: if let Some(t),
            typ: found.tok_typ(),
            area: CodeArea {
                span,
                source: self.tokens.source(),
            },
        }
        .wrap()
    }

    pub fn expect(&mut self, expected: Token) -> crate::error::Result<CodeSpan> {
        let next = self.next().unwrap_or_else(todo!("dfgsdfgdf"));
        if !matches!(self.next(), tok) {
            return Err(self.expected_err(expected.tok_name(), next.data, next.span));
        }
        Ok(next.span)
    }
    pub fn slice(&self, span: CodeSpan) -> &str {
        self.tokens.source()[span]
    }

    pub fn parse(&mut self) -> ASTData {
        let mut data = ASTData::default();
        let r: Rodeo<Spur> = Rodeo::new();
        todo!()
    }

    pub fn parse_int(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let src = &self.tokens.source()[span.to_range()];

        let int: u64 = match &src[0..2] {
            "0x" => self.parse_int_radix(&src[2..], 16, span)?,
            "0o" => self.parse_int_radix(&src[2..], 8, span)?,
            "0b" => self.parse_int_radix(&src[2..], 2, span)?,
            n if n.chars().all(char::is_numeric) => n.parse::<u64>().unwrap(),
            other => {
                return Err(SyntaxError::InvalidLiteral {
                    literal: other.to_string(),
                    area: CodeArea {
                        span,
                        source: self.file,
                    },
                }
                .wrap());
            }
        };
        Ok(ast_data.exprs.insert((Expression::Int(int), span)))
    }

    pub fn parse_int_radix(&self, src: &str, radix: u32, span: CodeSpan) -> u64 {
        u64::from_str_radix(src, radix).unwrap()
    }

    pub fn parse_float(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let src = &self.tokens.source()[span.to_range()];

        Ok(ast_data
            .exprs
            .insert((Expression::Float(src.parse::<f64>().unwrap()), span)))
    }

    pub fn parse_bool(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let src = &self.tokens.source()[span.to_range()];

        Ok(ast_data
            .exprs
            .insert((Expression::Bool(src.parse::<bool>().unwrap()), span)))
    } // wa

    pub fn parse_string(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let src = &self.tokens.source()[span.to_range()];

        let mut chars = src.chars();
        chars.next_back();

        let flag = &*chars
            .take_while(|c| !matches!(c, '"' | '\''))
            .collect::<String>();

        match flag {
            "b" => {
                let b_array = chars.collect::<String>().as_bytes();
                let b_expr_array = b_array
                    .iter()
                    .map(|b| ast_data.exprs.insert(Expression::Byte(b)))
                    .collect();

                Ok(ast_data.exprs.insert(Expression::Array(b_expr_array)))
            }
            "r" => todo!("All escapes but \""),
            "u" => { // "Unindents the string" (wish me luck)
            }
            "b64" => {
                let content = chars.collect::<String>();
                let out_string = base64::encode(&content).unwrap();
                Ok(ast_data.exprs.insert(Expression::String(out_string)))
            }
            "\"" => {
                let out_string: String = chars.collect();
                Ok(ast_data.exprs.insert(Expression::String(out_string)))
            }
            _ => todo!("Invalid string flag"),
        }
    }

    fn parse_escapes(&self, span: CodeSpan, string: String) -> crate::error::Result<String> {
        let mut out = String::new();
        let mut chars = string.chars();

        loop {
            match chars.next() {
                Some('\\') => out.push(match chars.next() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('"') => '"',
                    Some('\'') => '\'',
                    Some('\\') => '\\',
                    Some('u') => self.parse_unicode(span, &mut chars),
                    Some(c) => {
                        return Err(SyntaxError::InvalidEscape {
                            character: c,
                            area: span,
                        })
                    }

                    None => unreachable!(),
                }),
                Some(c) => out.push(c),
                None => break,
            }
        }

        Ok(out)
    }

    fn parse_unicode(&self, span: CodeSpan, chars: &mut Chars) -> crate::error::Result<String> {
        let opening_brace = chars.next().unwrap();

        if !matches!(opening_brace, Some('{')) {
            return Err(SyntaxError::ExpectedErr {
                expected: "{".into(),
                found: opening_brace,
                area: span,
            });
        }

        let hex = chars
            .take_while(|c| matches!(*c, '0'..='9' | 'a'..='f' | 'A'..='F'))
            .collect::<String>();

        let closing_brace = chars.next().unwrap();

        if !matches!(closing_brace, Some('}')) {
            return Err(SyntaxError::ExpectedErr {
                expected: "}".into(),
                found: closing_brace.to_string(),
                area: span,
            });
        }

        Ok(char::from_u32(self.parse_int_radix(&hex, 16, span)?).unwrap_or(INVALID_CHARACTER))
    }

    pub fn parse_identifier(
        &mut self,
        ast_data: &mut ASTData,
        r: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let src = &self.tokens.source()[span.to_range()];
        let s = r.get_or_intern(src);

        Ok(ast_data.exprs.insert((Expression::Ident(s), span)))
    }

    pub fn parse_var_or_macro(
        &mut self,
        ast_data: &mut ASTData,
        interner: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        let span = self.next().unwrap().span;
        let name = interner.get_or_intern(&self.tokens.source()[span.to_range()]);

        match self
            .peek()
            .unwrap_or_else(|| todo!("unexpected eof (syntax)"))
            .data
        {
            Token::FatArrow => {
                self.next();
                let code = self.parse_expr(ast_data, interner)?;
                Ok(ast_data.exprs.insert((
                    Expression::Func {
                        args: vec![(name.into(), None, None)],
                        ret_type: None,
                        code,
                    },
                    span.start..ast_data.exprs[code].1,
                )))
            }
            _ => Ok(ast_data.exprs.insert((Expression::Var(name), span))),
        }
    }

    pub fn parse_type_indicator(
        &mut self,
        ast_data: &mut ASTData,
        interner: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap();

        let span = next_token.span;
        let name = interner.get_or_intern(&self.tokens.source()[span.to_range()]);

        Ok(ast_data.exprs.insert((Expression::Type(name), span)))
    }

    pub fn parse_macro(
        &mut self,
        ast_data: &mut ASTData,
        interner: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        self.next();
        let mut args = vec![];
        let mut arg_spans = vec![];
        while self.peek() != Token::RParen {
            let ident = self.expect(Token::Ident)?;
            let name = self.slice(ident);

            let arg_type = if self.peek() == Token::Colon {
                self.next();
                Some(self.parse_expr(ast_data, interner)?)
            } else {
                None
            };
            let arg_default = if self.peek() == Token::Assign {
                self.next();
                Some(self.parse_expr(ast_data, interner)?)
            } else {
                None
            };
            args.push((name.into(), arg_type, arg_default));
            arg_spans.push(ident);
            if !matches!(self.peek(), Token::Comma | Token::RParen) {
                return Err(self.expected_err(") or ,", self.next(), ));
            }
        }
    }

    pub fn parse_paren_or_macro(
        &mut self,
        ast_data: &mut ASTData,
        interner: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        let depth = 0;
        let check = self.clone();
        loop {
            match check.next().unwrap_or_else(|| todo!("unmatched char")).data {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }
        let is_pattern;
        match check.next() {
            Token::FatArrow => is_pattern = false,
            Token::Arrow => {
                check.parse_expr(ast_data, interner)?;
                is_pattern = matches!(check.next(), Token::FatArrow);
            }
            _ => {
                self.next();
                let value = self.parse_expr(ast_data, interner)?;
                self.expect(Token::RParen);

                return Ok(value);
            }
        }
        if is_pattern {
            self.parse_macro(ast_data, interner)
        } else {
            self.parse_macro_pattern(ast_data, interner)
        }
    }

    pub fn parse_expr(
        &mut self,
        ast_data: &mut ASTData,
        interner: &mut Rodeo,
    ) -> crate::error::Result<ExprKey> {
        match self
            .peek()
            .unwrap_or_else(|| todo!("unexpected eof (syntax)"))
            .data
        {
            Token::Int => self.parse_int(ast_data),
            Token::Float => self.parse_float(ast_data),
            Token::True => {
                let span = self.next().unwrap().span;
                Ok(ast_data.exprs.insert((Expression::Bool(true), span)))
            }
            Token::False => {
                let span = self.next().unwrap().span;
                Ok(ast_data.exprs.insert((Expression::Bool(false), span)))
            }
            Token::String => self.parse_string(ast_data),
            Token::Ident => self.parse_var_or_macro(ast_data, interner),
            Token::TypeIndicator => self.parse_type_indicator(ast_data, interner),
            Token::LParen => self.parse_type_indicator(ast_data, interner),
            Token::LSqBracket => todo!(),
            Token::LBracket => todo!(),
            Token::QMark => todo!(),
            Token::TrigFnBracket => todo!(),
            Token::Split => todo!(),
            _ => todo!(),
        }
    }

    /// Used to eliminate having 'a lifetime on Lexer
    unsafe fn make_static<'a>(d: &'a str) -> &'static str {
        std::mem::transmute::<&'a str, &'static str>(d)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct CodeSpan {
    start: usize,
    end: usize,
}

pub struct Spanned<T> {
    data: T,
    span: CodeSpan,
}

impl CodeSpan {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn to_range(self) -> Range<usize> {
        self.into()
    }
}

impl From<Range<usize>> for CodeSpan {
    fn from(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }
}

impl From<CodeSpan> for Range<usize> {
    fn from(CodeSpan { start, end }: CodeSpan) -> Self {
        start..end
    }
}
