use logos::Logos;

// to be improved, turns literal escape sequences into the actual character
fn convert_string(s: &str) -> String {
    s
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
        .replace("\\'", "'")
        .replace("\\\"", "\"")
}


#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {

    #[regex(r#"\d+"#, |lex| lex.slice().parse(), priority = 2)]
    Int(usize),
    #[regex(r#"\d+(\.[\d]+)?"#, |lex| lex.slice().parse())]
    Float(f64),

    #[regex(r#""(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#, 
        |s| convert_string(&s.slice()[1..s.slice().len()-1])
    )]
    String(String),


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

    #[token("=")]
    Assign,



    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    
    #[error]
    Error,

    Eof,
}



impl Token {
    // used in error messages
    pub fn tok_name(&self) -> String {
        match self {
            Token::Int(v) => return v.to_string(),
            Token::Float(v) => return v.to_string(),
            Token::String(v) => v,
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident(n) => n,
            Token::Error => "",
            Token::Eof => "",
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
            Token::Eol => "",
            Token::If => "if",
            Token::Else => "else",
            Token::While => "while",
            Token::For => "for",
            Token::In => "in",
        }.into()
    }
    // also used in error messages
    pub fn tok_typ(&self) -> &str {
        use Token::*;
        match self {
            Plus | Minus | Mult | Div | Mod | Pow | PlusEq |
            MinusEq | MultEq | DivEq | ModEq | PowEq | Assign => "operator",

            Int(_) | Float(_) | String(_) | True | False => "literal",

            Ident(_) => "identifier",

            Let | Mut | For | While | If | Else | In => "keyword",
            Error => "unknown",
            Eof => "end of file",
            Eol => "end of line",

            LParen | RParen | RSqBracket | LSqBracket | RBracket | LBracket | Comma => "terminator",


        }
    }
}
