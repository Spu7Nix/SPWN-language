use logos::Logos;

// to be improved, turns literal escape sequences into the actual character
fn convert_string(s: &str) -> String {
    s.replace("\\n", "\n")
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

    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

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

    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    #[regex(r"@[a-zA-Z_]\w*", |lex| lex.slice()[1..].to_string())]
    TypeIndicator(String),

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
            Token::TypeIndicator(v) => return format!("@{}", v),
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
            Token::Return => "return",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Eq => "==",
            Token::NotEq => "!=",
            Token::Greater => ">",
            Token::GreaterEq => ">=",
            Token::Lesser => "<",
            Token::LesserEq => "<=",
            Token::Colon => ":",
            Token::FatArrow => "=>",
            Token::Arrow => "->",
        }
        .into()
    }
    // also used in error messages
    pub fn tok_typ(&self) -> &str {
        use Token::*;
        match self {
            Plus | Minus | Mult | Div | Mod | Pow | PlusEq | MinusEq | MultEq | DivEq | ModEq
            | PowEq | Assign | Eq | NotEq | Greater | GreaterEq | Lesser | LesserEq => "operator",

            Int(_) | Float(_) | String(_) | True | False => "literal",

            Ident(_) => "identifier",

            Let | Mut | For | While | If | Else | In | Return | Break | Continue => "keyword",
            Error => "unknown",
            Eof => "end of file",
            Eol => "end of line",
            TypeIndicator(_) => "type indicator",

            LParen | RParen | RSqBracket | LSqBracket | RBracket | LBracket | Comma | Colon
            | FatArrow | Arrow => "terminator",
        }
    }
}
