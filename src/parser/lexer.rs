use logos::Logos;

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
#[logos(subpattern string = r#""(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#)]
#[logos(subpattern digits = r#"(\d)([\d_]+)?"#)]
pub enum Token {
    #[regex(r#"(?&digits)"#, |lex| lex.slice().parse(), priority = 2)]
    Int(usize),
    #[regex(r#"(?&digits)(\.[\d_]+)?"#, |lex| lex.slice().parse(), priority = 1)]
    Float(f64),

    // number literals don't match their correct values as their converted (and validated) in the parser
    #[regex(r#"0b\w+"#, |lex| lex.slice().parse())]
    BinaryLiteral(String),
    #[regex(r#"0x\w+"#, |lex| lex.slice().parse())]
    HexLiteral(String),
    #[regex(r#"0o\w+"#, |lex| lex.slice().parse(), priority = 1)]
    OctalLiteral(String),

    #[regex(r#"(?&string)"#, 
        //|s| convert_string(&s.slice()[1..s.slice().len()-1]),
        |s| s.slice().parse(),
        priority = 3 // prioritise normal string over string flags
    )]
    String(String),

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
            Token::BinaryLiteral(b) => b,
            Token::HexLiteral(h) => h,
            Token::OctalLiteral(o) => o,
            Token::String(v) => v,
            Token::TypeIndicator(v) => return format!("@{}", v),
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident(n) => n,
            Token::Error => "unknown",
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

            Int(_) | Float(_) | BinaryLiteral(_) | HexLiteral(_) | OctalLiteral(_) | String(_)
            | True | False => "literal",

            Ident(_) => "identifier",

            Let | Mut | For | While | If | Else | In | Return | Break | Continue | TypeDef
            | Impl | Print | Split => "keyword",

            Error => "",
            TypeIndicator(_) => "type indicator",

            LParen | RParen | RSqBracket | LSqBracket | RBracket | LBracket | Comma | Colon
            | DoubleColon | FatArrow | Arrow | QMark | ExclMark | Eol | Eof => "terminator",
        }
    }
}

pub type Span = (usize, usize);
pub type Tokens = Vec<(Token, Span)>;

pub fn lex(code: String) -> Tokens {
    let mut tokens_iter = Token::lexer(&code);

    let mut tokens = vec![];
    while let Some(t) = tokens_iter.next() {
        tokens.push((t, (tokens_iter.span().start, tokens_iter.span().end)))
    }
    tokens.push((Token::Eof, (code.len(), code.len() + 1)));

    tokens
}
