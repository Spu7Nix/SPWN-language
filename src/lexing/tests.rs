use super::lexer::{LexError, Lexer};
use super::tokens::Token;

fn lex(code: &'static str) -> LexError<(Token, String)> {
    let mut l = Lexer::new(code);

    Ok((l.next().unwrap_or(Ok(Token::Eof))?, l.slice().into()))
}

macro_rules! tok {
    ($t:ident, $tok:path, $slice:literal) => {
        assert_eq!($t.0, $tok);
        assert_eq!(&$t.1, $slice);
    };
}

#[test]
fn test_lexer() -> LexError<()> {
    let t = lex("_")?;
    tok!(t, Token::Any, "_");

    let t = lex("1")?;
    tok!(t, Token::Int, "1");
    let t = lex("0xF")?;
    tok!(t, Token::HexInt, "0xF");
    let t = lex("0xf")?;
    tok!(t, Token::HexInt, "0xf");
    let t = lex("0o7")?;
    tok!(t, Token::OctalInt, "0o7");
    let t = lex("0b1")?;
    tok!(t, Token::BinaryInt, "0b1");

    let t = lex("0s3")?;
    tok!(t, Token::SeximalInt, "0s3");
    let t = lex("0χAB")?;
    tok!(t, Token::DozenalInt, "0χAB");
    let t = lex("0χab")?;
    tok!(t, Token::DozenalInt, "0χab");

    let t = lex("0φ01")?;
    tok!(t, Token::GoldenFloat, "0φ01");

    let t = lex("5.1")?;
    tok!(t, Token::Float, "5.1");

    let t = lex("'a'")?;
    tok!(t, Token::String, "'a'");
    let t = lex(r"uB'a'")?;
    tok!(t, Token::StringFlags, "uB");
    let t = lex("r##'a'b'##")?;
    tok!(t, Token::RawString, "r##'a'b'##");

    let t = lex("1g")?;
    tok!(t, Token::GroupID, "1g");
    let t = lex("1c")?;
    tok!(t, Token::ChannelID, "1c");
    let t = lex("1b")?;
    tok!(t, Token::BlockID, "1b");
    let t = lex("1i")?;
    tok!(t, Token::ItemID, "1i");
    let t = lex("?g")?;
    tok!(t, Token::ArbitraryGroupID, "?g");
    let t = lex("?c")?;
    tok!(t, Token::ArbitraryChannelID, "?c");
    let t = lex("?b")?;
    tok!(t, Token::ArbitraryBlockID, "?b");
    let t = lex("?i")?;
    tok!(t, Token::ArbitraryItemID, "?i");

    let t = lex("@a")?;
    tok!(t, Token::TypeIndicator, "@a");

    let t = lex("mut")?;
    tok!(t, Token::Mut, "mut");

    let t = lex("true")?;
    tok!(t, Token::True, "true");
    let t = lex("false")?;
    tok!(t, Token::False, "false");

    let t = lex("obj")?;
    tok!(t, Token::Obj, "obj");
    let t = lex("trigger")?;
    tok!(t, Token::Trigger, "trigger");

    let t = lex("if")?;
    tok!(t, Token::If, "if");
    let t = lex("else")?;
    tok!(t, Token::Else, "else");

    let t = lex("while")?;
    tok!(t, Token::While, "while");
    let t = lex("for")?;
    tok!(t, Token::For, "for");

    let t = lex("in")?;
    tok!(t, Token::In, "in");

    let t = lex("try")?;
    tok!(t, Token::Try, "try");
    let t = lex("catch")?;
    tok!(t, Token::Catch, "catch");
    let t = lex("throw")?;
    tok!(t, Token::Throw, "throw");

    let t = lex("match")?;
    tok!(t, Token::Match, "match");

    let t = lex("return")?;
    tok!(t, Token::Return, "return");
    let t = lex("break")?;
    tok!(t, Token::Break, "break");
    let t = lex("continue")?;
    tok!(t, Token::Continue, "continue");

    let t = lex("type")?;
    tok!(t, Token::Type, "type");
    let t = lex("impl")?;
    tok!(t, Token::Impl, "impl");

    let t = lex("overload")?;
    tok!(t, Token::Overload, "overload");

    let t = lex("unary")?;
    tok!(t, Token::Unary, "unary");

    #[cfg(debug_assertions)]
    {
        let t = lex("dbg")?;
        tok!(t, Token::Dbg, "dbg");
    }

    let t = lex("private")?;
    tok!(t, Token::Private, "private");

    let t = lex("extract")?;
    tok!(t, Token::Extract, "extract");
    let t = lex("import")?;
    tok!(t, Token::Import, "import");

    let t = lex("$")?;
    tok!(t, Token::Dollar, "$");

    let t = lex("is")?;
    tok!(t, Token::Is, "is");
    let t = lex("as")?;
    tok!(t, Token::As, "as");

    let t = lex("+")?;
    tok!(t, Token::Plus, "+");
    let t = lex("-")?;
    tok!(t, Token::Minus, "-");
    let t = lex("*")?;
    tok!(t, Token::Mult, "*");
    let t = lex("/")?;
    tok!(t, Token::Div, "/");
    let t = lex("%")?;
    tok!(t, Token::Mod, "%");
    let t = lex("**")?;
    tok!(t, Token::Pow, "**");
    let t = lex("+=")?;
    tok!(t, Token::PlusEq, "+=");
    let t = lex("-=")?;
    tok!(t, Token::MinusEq, "-=");
    let t = lex("*=")?;
    tok!(t, Token::MultEq, "*=");
    let t = lex("/=")?;
    tok!(t, Token::DivEq, "/=");
    let t = lex("%=")?;
    tok!(t, Token::ModEq, "%=");
    let t = lex("**=")?;
    tok!(t, Token::PowEq, "**=");
    let t = lex("&=")?;
    tok!(t, Token::BinAndEq, "&=");
    let t = lex("|=")?;
    tok!(t, Token::BinOrEq, "|=");
    let t = lex("<<=")?;
    tok!(t, Token::ShiftLeftEq, "<<=");
    let t = lex(">>=")?;
    tok!(t, Token::ShiftRightEq, ">>=");
    let t = lex("&")?;
    tok!(t, Token::BinAnd, "&");
    let t = lex("|")?;
    tok!(t, Token::BinOr, "|");
    let t = lex("^")?;
    tok!(t, Token::BinXor, "^");
    let t = lex("^=")?;
    tok!(t, Token::BinXorEq, "^=");
    let t = lex("<<")?;
    tok!(t, Token::ShiftLeft, "<<");
    let t = lex(">>")?;
    tok!(t, Token::ShiftRight, ">>");
    let t = lex("&&")?;
    tok!(t, Token::And, "&&");
    let t = lex("||")?;
    tok!(t, Token::Or, "||");

    let t = lex(";")?;
    tok!(t, Token::Eol, ";");

    let t = lex("(")?;
    tok!(t, Token::LParen, "(");
    let t = lex(")")?;
    tok!(t, Token::RParen, ")");
    let t = lex("[")?;
    tok!(t, Token::LSqBracket, "[");
    let t = lex("]")?;
    tok!(t, Token::RSqBracket, "]");
    let t = lex("{")?;
    tok!(t, Token::LBracket, "{");
    let t = lex("}")?;
    tok!(t, Token::RBracket, "}");

    let t = lex("!{")?;
    tok!(t, Token::TrigFnBracket, "!{");

    let t = lex(",")?;
    tok!(t, Token::Comma, ",");

    let t = lex("==")?;
    tok!(t, Token::Eq, "==");
    let t = lex("!=")?;
    tok!(t, Token::Neq, "!=");
    let t = lex(">")?;
    tok!(t, Token::Gt, ">");
    let t = lex(">=")?;
    tok!(t, Token::Gte, ">=");
    let t = lex("<")?;
    tok!(t, Token::Lt, "<");
    let t = lex("<=")?;
    tok!(t, Token::Lte, "<=");
    let t = lex("=")?;
    tok!(t, Token::Assign, "=");

    let t = lex(":")?;
    tok!(t, Token::Colon, ":");
    let t = lex("::")?;
    tok!(t, Token::DoubleColon, "::");

    let t = lex(".")?;
    tok!(t, Token::Dot, ".");
    let t = lex("..")?;
    tok!(t, Token::Range, "..");
    let t = lex("...")?;
    tok!(t, Token::Spread, "...");

    let t = lex("=>")?;
    tok!(t, Token::FatArrow, "=>");
    let t = lex("->")?;
    tok!(t, Token::Arrow, "->");

    let t = lex("?")?;
    tok!(t, Token::QMark, "?");

    let t = lex("!")?;
    tok!(t, Token::ExclMark, "!");

    let t = lex("#")?;
    tok!(t, Token::Hashtag, "#");

    let t = lex("ε")?;
    tok!(t, Token::Epsilon, "ε");

    let t = lex("foo")?;
    tok!(t, Token::Ident, "foo");

    let t = lex("\n")?;
    tok!(t, Token::Newline, "\n");

    let t = lex("")?;
    tok!(t, Token::Eof, "");

    let t = lex("self")?;
    tok!(t, Token::Slf, "self");

    Ok(())
}
