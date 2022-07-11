use logos::Logos;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
#[logos(subpattern digits = r#"(\d)([\d_]+)?"#)]
pub enum Token {
    #[regex(r#"(0[b])?(?&digits)"#, priority = 2)]
    Int,
    #[regex(r#"(?&digits)(\.[\d_]+)?"#)]
    Float,

    #[regex(r#"\w*"(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#)]
    String,

    #[token("let")]
    Let,
    #[token("mut")]
    Mut,

    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("obj")]
    Obj,
    #[token("trigger")]
    Trigger,

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
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,

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
    #[token("add")]
    Add,

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

impl From<Token> for &str {
    fn from(tok: Token) -> Self {
        match tok {
            Token::Int => "int literal",
            Token::Float => "float literal",
            Token::String => "string literal",
            Token::TypeIndicator => "type indicator",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident => "identifier",
            Token::Error => "invalid",
            Token::Eof => "end of file",
            Token::True => "true",
            Token::False => "false",
            Token::Obj => "obj",
            Token::Trigger => "trigger",
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
            Token::Eol => ";",
            Token::If => "if",
            Token::Else => "else",
            Token::While => "while",
            Token::For => "for",
            Token::In => "in",
            Token::Try => "try",
            Token::Catch => "catch",
            Token::Return => "return",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Print => "print",
            Token::Add => "add",
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
        }
    }
}

// converts tokens to a string
impl ToString for Token {
    fn to_string(&self) -> String {
        let t: &'static str = (*self).into();
        t.to_string()
    }
}

// have to use a wrapper struct since it isn't possible to implement on types you dont own (including built-ins)
#[derive(Clone, Debug)]
pub struct Tokens(pub Vec<Token>);

impl From<Token> for Tokens {
    fn from(tok: Token) -> Self {
        Self(vec![tok])
    }
}

// formats the tokens in a readable way:
// single token -> `<token>`
// 2 tokens -> `<token> or <token>`
// n tokens `<token>, <token>, ...., or <final token>`
impl ToString for Tokens {
    fn to_string(&self) -> String {
        if self.0.len() == 1 {
            self.0[0].to_string()
        } else {
            let comma = &self.0[..(self.0.len() - 1)];
            // we know there is always going to be a last element in the array
            // since the parser will never check for less than 1 tokens
            // (and a length 1 array has its own formatting above anyway)
            let last = self.0.last().unwrap();
            format!(
                "{} or {}",
                comma
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                last.to_string()
            )
        }
    }
}

// implementes bitwise or to allow multiple tokens to be chained together like
// `Token::A | Token::B`
// this impl alone only allows 2 tokens to be chained together
impl std::ops::BitOr<Token> for Token {
    type Output = Tokens;

    fn bitor(self, rhs: Self) -> Self::Output {
        Tokens(Vec::from([self, rhs]))
    }
}

// `Token::A | Token::B` becomes a `Tokens(Vec[Token::A, Token::B])`
// that means if you chain together 3 tokens (or more) it becomes
// `Tokens(Vec[Token::A, Token::B]) | Token::C`
// so that has to have its own implementation directly on the struct
impl std::ops::BitOr<Token> for Tokens {
    type Output = Tokens;

    fn bitor(self, rhs: Token) -> Self::Output {
        let mut out = self.0;
        out.push(rhs);
        Tokens(out)
    }
}
