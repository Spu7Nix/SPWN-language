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
