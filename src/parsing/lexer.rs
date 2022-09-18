use logos::Logos;

// a &= b
// a = a + b
// a

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    #[regex(r#"(0[b])?((\d)([\d_]+)?)"#, priority = 2)]
    Int,
    #[regex(r#"((\d)([\d_]+)?)(\.[\d_]+)?"#)]
    Float,
    #[regex(r"([0-9]+|\?)[gbci]")]
    Id,

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
    Type,
    #[token("impl")]
    Impl,

    #[token("import")]
    Import,

    #[token("$")]
    Dollar,

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
    MuLte,
    #[token("/=")]
    DivEq,
    #[token("%=")]
    ModEq,
    #[token("^=")]
    PowEq,

    #[token("&")]
    PatAnd,
    #[token("|")]
    PatOr,

    #[token("&&")]
    And,
    #[token("||")]
    Or,

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
    Neq,
    #[token(">")]
    Gt,
    #[token(">=")]
    Gte,
    #[token("<")]
    Lt,
    #[token("<=")]
    Lte,

    #[token("=")]
    Assign,

    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,

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

// converts tokens to a string
impl ToString for Token {
    fn to_string(&self) -> String {
        let t: &'static str = (*self).into();
        t.to_string()
    }
}

// have to use a wrapper struct since it isn't possible to implement on types you dont own (including built-ins)
#[derive(Clone, Debug)]
pub struct TokenUnion(pub Vec<Token>);

impl From<Token> for TokenUnion {
    fn from(tok: Token) -> Self {
        Self(vec![tok])
    }
}

// formats the tokens in a readable way:
// single token -> `<token>`
// 2 tokens -> `<token> or <token>`
// n tokens `<token>, <token>, ...., or <final token>`
impl ToString for TokenUnion {
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
    type Output = TokenUnion;

    fn bitor(self, rhs: Self) -> Self::Output {
        TokenUnion(Vec::from([self, rhs]))
    }
}

// `Token::A | Token::B` becomes a `TokenUnion(Vec[Token::A, Token::B])`
// that means if you chain together 3 tokens (or more) it becomes
// `TokenUnion(Vec[Token::A, Token::B]) | Token::C`
// so that has to have its own implementation directly on the struct
impl std::ops::BitOr<Token> for TokenUnion {
    type Output = TokenUnion;

    fn bitor(self, rhs: Token) -> Self::Output {
        let mut out = self.0;
        out.push(rhs);
        TokenUnion(out)
    }
}
