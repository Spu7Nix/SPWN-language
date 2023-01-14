use crate::lexer;

lexer! {

    Int: regex(r#"0b[01_]+|0o[0-7_]+|0x[0-9a-fA-F_]+|[\d_]+"#),
    Float: regex(r#"[\d_]+(\.[\d_]+)?"#),
    String: regex(r#""(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*'"#),

    Id: regex(r"([0-9]+|\?)[gbci]"),
    TypeIndicator: regex(r"@[a-zA-Z_]\w*"),

    Let: text("let"),
    Mut: text("mut"),

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

    BinAnd: text("&"),
    BinOr: text("|"),
    BinNot: text("~"),

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

    @skip: r#"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*"#;
    @error: Error;
}
