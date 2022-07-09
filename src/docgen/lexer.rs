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
    #[regex(r#"(?:///).*"#)]
    DocComment,

    #[regex(r#"(0[b])?(?&digits)"#, priority = 2)]
    Int,
    #[regex(r#"(?&digits)(\.[\d_]+)?"#)]
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

    #[regex(r"@[a-zA-Z_]\w*")]
    TypeIndicator,

    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*")]
    Ident,

    #[error]
    Error,

    Eof,
}

// impl Into<&str> for Token {
//     fn into(self) -> &'static str {
//         match self {}
//     }
// }
