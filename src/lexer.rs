use logos::Logos;


fn convert_string(s: &str) -> String {
    s
        .replace("\\n", "\n")
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



    #[regex(r"[a-zA-Z_ඞ][a-zA-Z_0-9ඞ]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[error]
    #[regex(r"[ \t\f\n\r]+|/\*[^*]*\*(([^/\*][^\*]*)?\*)*/|//[^\n]*", logos::skip)]
    Error,

    Eof,
}



impl Token {
    pub fn tok_str(&self) -> &str {
        match self {
            Token::Int(_) => "int literal",
            Token::Float(_) => "float literal",
            Token::String(_) => "string literal",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::Ident(_) => "identifier",
            Token::Error => "unknown",
            Token::Eof => "end of file",
        }
    }
}
