use std::{ops::Range, path::PathBuf};

use logos::Logos;

use crate::interpreter::value::Value;

use super::parser::{ASTData, ExprKey};

// // ew
// fn string_flag<'s>(tok: &mut logos::Lexer<'s, Token>) -> String {
//     let sliced = tok.slice();

//     // get location of `"` or `'` in the token (end of string flag)
//     // this will never be called on a string without a flag as a normal string has higher priority
//     // so we can `.unwrap()` without issue

//     let end = sliced.find("\"").unwrap_or_else(|| sliced.find("'").unwrap());

//     let flag = &sliced[0..end];
//     let string = &sliced[end+1..sliced.len()];

//     String::new()
// }

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(subpattern string = r#""\w?((?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*')"#)]
#[logos(subpattern digits = r#"(\d)([\d_]+)?"#)]
pub enum Token {
    #[regex(r#"(0[box])(?&digits)"#)]
    Int,
    #[regex(r#"(?&digits)(\.[\d_]+)?"#)]
    Float,

    #[regex(r#"(?&string)"#)]
    String,

    // #[regex(r#"_(\w*)?(?&string)"#,
    //     //|s| s.slice()[0..s.slice().find('"').unwrap_or_else(|| s.slice().find("'").unwrap())].parse(), // take up to `"` or `'`
    //    // |s| string_flag(s),
    //    |s| s.slice().parse(),
    //     priority = 1,
    // )]
    // StringFlag(String),
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
            | DoubleColon | FatArrow | Arrow | QMark | ExclMark | Eol | Eof => "terminator",
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

    pub fn parse(&mut self) -> ASTData {
        let mut data = ASTData::default();
        todo!()
    }

    pub fn parse_int(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        let next_token = self.next().unwrap_or_else(|| todo!("unexpected eof (syntax)"));

        let codespan = &self.tokens.source()[next_token.span.to_range()];
        let int = match &codespan[0..2] {
            "0x" => todo!(),
            "0b" => todo!(),
            "0o" => todo!(),
            n if n.chars().all(char::is_numeric) => n.parse::<u32>(),
            other => return Err(SyntaxError::InvalidLiteral {
                literal: other.to_string(),
                area: todo!(),
            }.wrap()),
        };

        todo!()
    }

    pub fn parse_expr(&mut self, ast_data: &mut ASTData) -> crate::error::Result<ExprKey> {
        match self.peek().unwrap_or_else(|| todo!("unexpected eof (syntax)")).data {
            Token::Int => self.parse_int(ast_data),
            // haven't removed yet because there's so much code that uses it and that code can be reused here with necessary changes
            _ => todo!(),
        }
    }

    /// Used to eliminate having 'a lifetime on Lexer
    unsafe fn make_static<'a>(d: &'a str) -> &'static str {
        std::mem::transmute::<&'a str, &'static str>(d)
    }
}

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
