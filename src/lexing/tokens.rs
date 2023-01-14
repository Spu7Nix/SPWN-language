use crate::lexer;

lexer! {
    Int: regex(r#"0b[01_]+|0o[0-7_]+|0x[0-9a-fA-F_]+|[\d_]+"#),
    Float: regex(r#"[\d_]+(\.[\d_]+)?"#),

    String: regex(r#"([a-zA-Z]\w*)?("(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*')"#),
    RawString: regex(r###"([a-zA-Z]\w*_)?(r".*?"|r#".*?"#|r##".*?"##|r'.*?'|r#'.*?'#|r##'.*?'##)"###),

    Id: regex(r"([0-9]+|\?)[gbci]"),
    TypeIndicator: regex(r"@[a-zA-Z_]\w*"),

    Attribute: regex(r"#\[.*?\]"),

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

    Return: text("return"),
    Break: text("break"),
    Continue: text("continue"),

    Type: text("type"),
    Impl: text("impl"),

    Import: text("import"),
    Dollar: text("$"),

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
    BinNotEq: text("~="),

    ShiftLeftEq: text("<<="),
    ShiftRightEq: text(">>="),

    BinAnd: text("&"),
    BinOr: text("|"),
    BinNot: text("~"),

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
    DotDot: text(".."),

    FatArrow: text("=>"),
    Arrow: text("->"),

    QMark: text("?"),
    ExclMark: text("!"),

    Ident: regex(r"[a-zA-Z_][a-zA-Z_0-9]*"),

    Eof,

    @skip: r#"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*"#;
    @error: Error;
}

impl Token {
    pub fn to_str(self) -> &'static str {
        use Token::*;
        match self {
            Int => "int literal",
            Float => "float literal",
            Id => "ID literal",
            String => "string literal",
            RawString => "raw string literal",
            TypeIndicator => "type indicator",
            Attribute => "attribute",
            Let => "let",
            Ident => "identifier",
            Error => "unknown",
            Eof => "end of file",
            True => "true",
            False => "false",
            Obj => "obj",
            Trigger => "trigger",
            Plus => "+",
            Minus => "-",
            Mult => "*",
            Div => "/",
            Mod => "%",
            Pow => "^",
            PlusEq => "+=",
            MinusEq => "-=",
            MultEq => "*=",
            DivEq => "/=",
            ModEq => "%=",
            PowEq => "^=",
            Assign => "=",
            LParen => "(",
            RParen => ")",

            LSqBracket => "[",
            RSqBracket => "]",
            LBracket => "{",
            RBracket => "}",
            TrigFnBracket => "!{",
            Comma => ",",
            Eol => ";",
            If => "if",
            Else => "else",
            While => "while",
            For => "for",
            In => "in",
            Try => "try",
            Catch => "catch",
            Return => "return",
            Break => "break",
            Continue => "continue",
            Is => "is",
            Eq => "==",
            Neq => "!=",
            Gt => ">",
            Gte => ">=",
            Lt => "<",
            Lte => "<=",
            Colon => ":",
            DoubleColon => "::",
            Dot => ".",
            DotDot => "..",
            FatArrow => "=>",
            Arrow => "->",
            QMark => "?",
            ExclMark => "!",
            Type => "type",
            Impl => "impl",
            Dollar => "$",
            Import => "import",
            As => "as",
            BinAndEq => "&=",
            BinOrEq => "|=",
            BinNotEq => "~=",
            BinAnd => "&",
            BinOr => "|",
            BinNot => "~",
            And => "&&",
            Or => "||",
            ShiftLeftEq => "<<=",
            ShiftRightEq => ">>=",
            ShiftLeft => "<<",
            ShiftRight => ">>",
        }
    }
}
