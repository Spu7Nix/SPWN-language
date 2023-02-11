use crate::lexer;

lexer! {
    ["int literal"]
    Int: regex(r#"0b[01_]+|0o[0-7_]+|0x[0-9a-fA-F_]+|[\d_]+"#),
    ["float literal"]
    Float: regex(r#"[\d_]+(\.[\d_]+)?"#),

    ["string literal"]
    String: regex(r###"([a-zA-Z]\w*)?("(?:\\.|[^\\"])*"|'(?:\\.|[^\\'])*')|([a-zA-Z]\w*_)?(r".*?"|r#".*?"#|r##".*?"##|r'.*?'|r#'.*?'#|r##'.*?'##)"###),

    ["ID literal"]
    Id: regex(r"([0-9]+|\?)[gbci]"),
    ["type indicator"]
    TypeIndicator: regex(r"@[a-zA-Z_]\w*"),

    // ["attribute"]
    //Attribute: regex(r"#\[.*?\]"),

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

    Dbg: text("dbg"),

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
    Range: text(".."),
    Spread: text("..."),

    FatArrow: text("=>"),
    Arrow: text("->"),

    QMark: text("?"),
    ExclMark: text("!"),

    Hashtag: text("#"),

    ["identifier"]
    Ident: regex(r"[a-zA-Z_][a-zA-Z_0-9]*"),

    ["linebreak"]
    Newline: regex(r"(\n|(\r\n))+"),

    ["end of file"]
    Eof,

    @skip: r#"[ \t\f]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*"#;
    @error: Error;
}
