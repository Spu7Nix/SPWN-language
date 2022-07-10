use logos::Logos;

// doc comments on:
// lib comment (only first lines onwards)
// global constant vars

// type defintions
// type implementation members (vars and fns)
// macros

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(subpattern digits = r#"(\d)([\d_]+)?"#)]
pub enum Token {
    #[regex(r#"(?:///).*"#, priority = 3)]
    DocComment,

    #[regex(r"(0[b])?(?&digits)", priority = 2)]
    Int,
    #[regex(r"(?&digits)(\.[\d_]+)?")]
    Float,

    #[regex(r#"\w*"(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#)]
    String,

    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("type")]
    TypeDef,
    #[token("impl")]
    Impl,

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

    #[token("=")]
    Assign,

    #[token(":")]
    Colon,

    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    #[regex(r"@[a-zA-Z_]\w*")]
    TypeIndicator,

    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*")]
    Ident,

    #[regex(r"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    #[error]
    Error,

    Eof,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        (match self {
            Token::Int => "int literal",
            Token::Float => "float literal",
            Token::String => "string literal",
            Token::True => "true",
            Token::False => "false",
            Token::TypeDef => "type",
            Token::Impl => "impl",
            Token::Eol => ";",
            Token::LParen => ")",
            Token::RParen => "(",
            Token::LSqBracket => "]",
            Token::RSqBracket => "[",
            Token::RBracket => "}",
            Token::LBracket => "{",
            Token::Comma => ",",
            Token::Assign => "=",
            Token::Colon => ":",
            Token::FatArrow => "=>",
            Token::Arrow => "->",
            Token::TypeIndicator => "type indicator",
            Token::Ident => "identifier",
            _ => unreachable!(),
        })
        .into()
    }
}
