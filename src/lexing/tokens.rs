use std::cmp::Ordering;

use crate::lexer;

lexer! {
    Any: text("_"),

    Int: regex(r#"0b[01_]+|0o[0-7_]+|0x[0-9a-fA-F_]+|[\d_]+"#),
    Float: regex(r#"[\d_]+(\.[\d_]+)?"#),

    String: regex(r###"([a-zA-Z]\w*)?("(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*')|([a-zA-Z]\w*_)?(r".*?"|r#".*?"#|r##".*?"##|r'.*?'|r#'.*?'#|r##'.*?'##)"###),

    Id: regex(r"([0-9]+|\?)[gbci]"),
    TypeIndicator: regex(r"@[a-zA-Z_]\w*"),

    Let: text("let"),

    True: text("true"),
    False: text("false"),
    Obj: text("obj"),
    Trigger: text("trigger"),

    If: text("if"),
    Else: text("else"),
    While: text("while"),
    For: text("for"),
    In: text("in"),
    Try: text("try"),
    Catch: text("catch"),
    Throw: text("throw"),

    Return: text("return"),
    Break: text("break"),
    Continue: text("continue"),

    Type: text("type"),
    Impl: text("impl"),
    Overload: text("overload"),
    Unary: text("unary"),

    Dbg: text("dbg"),

    Private: text("private"),
    Extract: text("extract"),
    Import: text("import"),
    Dollar: text("$"),

    Slf: text("self"),

    Is: text("is"),
    As: text("as"),

    Plus: text("+"),
    Minus: text("-"),
    Mult: text("*"),
    Div: text("/"),
    Mod: text("%"),
    Pow: text("^"),
    PlusEq: text("+="),
    MinusEq: text("-="),
    MultEq: text("*="),
    DivEq: text("/="),
    ModEq: text("%="),
    PowEq: text("^="),

    BinAndEq: text("&="),
    BinOrEq: text("|="),

    ShiftLeftEq: text("<<="),
    ShiftRightEq: text(">>="),

    BinAnd: text("&"),
    BinOr: text("|"),

    ShiftLeft: text("<<"),
    ShiftRight: text(">>"),

    And: text("&&"),
    Or: text("||"),

    Eol: text(";"),

    LParen: text("("),
    RParen: text(")"),
    LSqBracket: text("["),
    RSqBracket: text("]"),
    LBracket: text("{"),
    RBracket: text("}"),
    TrigFnBracket: text("!{"),

    Comma: text(","),

    Eq: text("=="),
    Neq: text("!="),
    Gt: text(">"),
    Gte: text(">="),
    Lt: text("<"),
    Lte: text("<="),

    Assign: text("="),

    Colon: text(":"),
    DoubleColon: text("::"),
    Dot: text("."),
    Range: text(".."),
    Spread: text("..."),

    FatArrow: text("=>"),
    Arrow: text("->"),

    QMark: text("?"),
    ExclMark: text("!"),

    Hashtag: text("#"),
    Epsilon: text("ε"),

    Ident: regex(r"[a-zA-Z_][a-zA-Z_0-9]*"),

    Newline: regex(r"(\n|(\r\n))+"),

    Eof,

    @skip: r#"[ \t\f]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*"#;
    @error: Error;
}

// generate this in macro???
impl Token {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Int => "int literal",
            Self::Float => "float literal",
            Self::Id => "ID literal",
            Self::String => "string literal",
            Self::TypeIndicator => "type indicator",
            Self::Let => "let",
            Self::Ident => "identifier",
            Self::Error => "unknown",
            Self::Eof => "end of file",
            Self::True => "true",
            Self::False => "false",
            Self::Obj => "obj",
            Self::Trigger => "trigger",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Mult => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Pow => "^",
            Self::PlusEq => "+=",
            Self::MinusEq => "-=",
            Self::MultEq => "*=",
            Self::DivEq => "/=",
            Self::ModEq => "%=",
            Self::PowEq => "^=",
            Self::Assign => "=",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LSqBracket => "[",
            Self::RSqBracket => "]",
            Self::LBracket => "{",
            Self::RBracket => "}",
            Self::TrigFnBracket => "!{",
            Self::Comma => ",",
            Self::Eol => ";",
            Self::If => "if",
            Self::Else => "else",
            Self::While => "while",
            Self::For => "for",
            Self::In => "in",
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Return => "return",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Is => "is",
            Self::Eq => "==",
            Self::Neq => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Colon => ":",
            Self::DoubleColon => "::",
            Self::Dot => ".",
            Self::Range => "..",
            Self::FatArrow => "=>",
            Self::Arrow => "->",
            Self::QMark => "?",
            Self::ExclMark => "!",
            Self::Type => "type",
            Self::Impl => "impl",
            Self::Dollar => "$",
            Self::Import => "import",
            Self::As => "as",
            Self::BinAndEq => "&=",
            Self::BinOrEq => "|=",
            Self::BinAnd => "&",
            Self::BinOr => "|",
            Self::And => "&&",
            Self::Or => "||",
            Self::ShiftLeftEq => "<<=",
            Self::ShiftRightEq => ">>=",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::Hashtag => "#",
            Self::Extract => "extract",
            Self::Newline => "linebreak",
            Self::Spread => "...",
            Self::Dbg => "dbg",
            Self::Slf => "self",
            Self::Private => "private",
            Self::Any => "_",
            Self::Overload => "overload",
            Self::Unary => "unary",
            Self::Throw => "throw",
            Self::Epsilon => "ε",
        }
    }
}

fn parse_raw_string(lex: &mut logos::Lexer<'_, Token2>) -> logos::FilterResult<(), Error> {
    let ht_count = lex.slice()[1..].len();

    let mut chars = lex.remainder().chars().peekable();

    match chars.next() {
        Some('"') | Some('\'') => (),
        _ => return logos::FilterResult::Error(Error::UnexpectedStringFlags),
    }

    let mut out = String::new();
    let mut found_end = false;

    while let Some(ch) = chars.next() {
        match (ch, chars.peek()) {
            (a, Some('"')) | (a, Some('\'')) => {
                out.push(a);
                chars.next();
                lex.bump(1);

                let mut c = 0;
                while let Some('#') = chars.next() {
                    c += 1
                }

                if c >= ht_count {
                    found_end = true;
                    lex.bump(ht_count);

                    for _ in 0..ht_count {
                        lex.next();
                    }

                    break;
                }
            },
            (a, Some(b)) => {
                out.push(a);
                out.push(*b);
                chars.next();
                lex.bump(2);
            },
            (a, None) => {
                out.push(a);
                lex.bump(1);
                break;
            },
        }
    }

    if !found_end {
        return logos::FilterResult::Error(Error::UnterminatedRawString);
    }

    logos::FilterResult::Emit(())
}

#[test]
fn test() {
    use logos::{Lexer, Logos};

    let mut a = Token2::lexer(r#####" "aaa" "#####);

    while let Some(t) = a.next() {
        dbg!(t, a.slice());
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
enum Error {
    #[default]
    Error,
    UnterminatedRawString,
    UnexpectedStringFlags,
}

#[derive(Debug, logos::Logos, PartialEq)]
#[logos(
    error = Error,
    //extras = LexerExtras,
    skip r"[ \t\f]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*"
)]
enum Token2 {
    #[token("_", priority = 0)]
    Any,

    #[regex(r#"0b[01_]+|0o[0-7_]+|0x[0-9a-fA-F_]+|[0-9][0-9]*"#, priority = 3)]
    Int,
    #[regex(r#"[0-9][0-9_]*(\.[0-9_]+)?"#, priority = 1)]
    Float,

    #[regex(r#"([a-zA-Z][a-zA-Z0-9_]*)?"(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#)]
    #[regex(r#"r[#]*"#, parse_raw_string)]
    String,

    #[regex(r"([0-9]+|\?)[gbci]")]
    Id,
    #[regex(r"@[a-zA-Z][a-zA-Z0-9_]*")]
    TypeIndicator,

    #[token("let")]
    Let,

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
    #[token("throw")]
    Throw,

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
    #[token("overload")]
    Overload,
    #[token("unary")]
    Unary,

    #[token("dbg")]
    Dbg,

    #[token("private")]
    Private,
    #[token("extract")]
    Extract,
    #[token("import")]
    Import,
    #[token("$")]
    Dollar,

    #[token("self")]
    Slf,

    #[token("is")]
    Is,
    #[token("as")]
    As,

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

    #[token("&=")]
    BinAndEq,
    #[token("|=")]
    BinOrEq,

    #[token("<<=")]
    ShiftLeftEq,
    #[token(">>=")]
    ShiftRightEq,

    #[token("&")]
    BinAnd,
    #[token("|")]
    BinOr,

    #[token("<<")]
    ShiftLeft,
    #[token(">>")]
    ShiftRight,

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
    Range,
    #[token("...")]
    Spread,

    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,

    #[token("?")]
    QMark,
    #[token("!")]
    ExclMark,

    #[token("#")]
    Hashtag,
    #[token("ε")]
    Epsilon,
    #[regex(r"[a-zA-Z][a-zA-Z_0-9]*", priority = 1)]
    Ident,
    #[regex(r"(\n|(\r\n))+")]
    Newline,

    Eof,
}
