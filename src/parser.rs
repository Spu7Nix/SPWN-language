use slotmap::{SlotMap, new_key_type};

use crate::{CodeArea, lexer::Token, SpwnSource, error::SyntaxError};





pub type Tokens = Vec<(Token, (usize, usize))>;


new_key_type! {
    pub struct ExprKey;
    pub struct StmtKey;
}

enum KeyType {
    Expr(ExprKey),
    StmtKey(StmtKey),
}

trait ASTKey {
    fn into_key(self) -> KeyType;
}
impl ASTKey for ExprKey {
    fn into_key(self) -> KeyType {
        KeyType::Expr(self)
    }
}
impl ASTKey for StmtKey {
    fn into_key(self) -> KeyType {
        KeyType::StmtKey(self)
    }
}


#[derive(Default)]
pub struct ASTData {
    pub exprs: SlotMap<ExprKey, (Expression, CodeArea)>,
    pub stmts: SlotMap<StmtKey, (Statement, CodeArea)>,
}
impl ASTData {
    // pub fn insert<T: ASTNode + 'static>(&mut self, node: T, area: CodeArea) -> ASTKey {
    //     self.map.insert((Box::new(node), area))
    // }
    fn area<K: ASTKey>(&self, k: K) -> &CodeArea {
        match k.into_key() {
            KeyType::Expr(k) => &self.exprs[k].1,
            KeyType::StmtKey(k) => &self.stmts[k].1,
        }
    }
    pub fn get_expr(&self, k: ExprKey) -> &Expression {
        &self.exprs[k].0
    }
    pub fn get_stmt(&self, k: StmtKey) -> &Statement {
        &self.stmts[k].0
    }
    pub fn insert_expr(&mut self, expr: Expression, area: CodeArea) -> ExprKey {
        self.exprs.insert((expr, area))
    }
    pub fn insert_stmt(&mut self, stmt: Statement, area: CodeArea) -> StmtKey {
        self.stmts.insert((stmt, area))
    }

    pub fn debug(&self, stmts: &Statements) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        debug_str += "-------- exprs --------\n";
        for (k, (e, _)) in &self.exprs {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-------- stmts --------\n";
        for (k, (e, _)) in &self.stmts {
            writeln!(&mut debug_str, "{:?}:\t\t{:?}", k, e).unwrap();
        }
        debug_str += "-----------------------\n";

        for i in stmts {
            writeln!(&mut debug_str, "{:?}", i).unwrap();
        }

        let re = regex::Regex::new(r"(ExprKey\([^)]*\))").unwrap();
        debug_str = re.replace_all(&debug_str, ansi_term::Color::Yellow.bold().paint("$1").to_string()).into();
        let re = regex::Regex::new(r"(StmtKey\([^)]*\))").unwrap();
        debug_str = re.replace_all(&debug_str, ansi_term::Color::Blue.bold().paint("$1").to_string()).into();

        println!("{}", debug_str);
    }

}


pub struct ParseData {
    pub tokens: Tokens,
    pub source: SpwnSource,
}


#[derive(Debug, Clone)]
pub enum Literal {
    Int(usize),
    Float(f64),
    String(String),
    Bool(bool),
}
impl Literal {
    pub fn to_str(&self) -> String {
        match self {
            Literal::Int(v) => v.to_string(),
            Literal::Float(v) => v.to_string(),
            Literal::String(v) => v.to_string(),
            Literal::Bool(v) => v.to_string(),
        }
    }
}




#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Op(ExprKey, Token, ExprKey),
    Unary(Token, ExprKey),

    Var(String),
    
    Array(Vec<ExprKey>),

    Index {
        base: ExprKey,
        index: ExprKey,
    },

    Empty,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(ExprKey),
    Declaration(String, ExprKey),
    If {
        branches: Vec<(ExprKey, Statements)>,
        else_branch: Option<Statements>,
    },
    While {
        cond: ExprKey,
        code: Statements,
    },
    For {
        var: String,
        iterator: ExprKey,
        code: Statements,
    }
}

pub type Statements = Vec<StmtKey>;

// impl Expression {
//     pub fn debug_str(&self, ast_data: &ASTData) -> String {
//         use ansi_term::Color::*;
//         match self {
//             Expression::Literal(l) => format!(
//                 "{} < {} >",
//                 Blue.bold().paint("Literal"), l.to_str()
//             ),
//             Expression::Op(a, op, b) => format!(
//                 "{} <\n\ta: {}\n\top: {}\n\tb: {}\n>",
//                 Blue.bold().paint("Op"),
//                 ast_data.get_expr(*a).debug_str(ast_data).replace('\n', "\n\t"),
//                 op.tok_name().replace("\n", "\n\t"),
//                 ast_data.get_expr(*b).debug_str(ast_data).replace('\n', "\n\t"),
//             ),
//             Expression::Unary(op, v) => format!(
//                 "{} <\n\top: {}\n\tvalue: {}\n>",
//                 Blue.bold().paint("Unary"),
//                 op.tok_name().replace("\n", "\n\t"),
//                 ast_data.get_expr(*v).debug_str(ast_data).replace('\n', "\n\t"),
//             ),
//             Expression::Array(arr) => format!(
//                 "{} <[\n\t{}\n]>",
//                 Blue.bold().paint("Array"),
//                 arr.iter().map(
//                     |k| ast_data.get_expr(*k).debug_str(ast_data).replace('\n', "\n\t")
//                 ).collect::<Vec<_>>().join(",\n\t"),
//             ),
//         }
//     }
// }







macro_rules! parse_util {
    ($parse_data:expr, $ast_data:expr, $pos:expr) => {
        #[allow(unused_macros)]
        macro_rules! expected_err {
            ($exp:expr, $tok:expr, $area:expr) => {
                return Err( SyntaxError::Expected {
                    expected: $exp.to_string(),
                    typ: $tok.tok_typ().to_string(),
                    found: $tok.tok_name().to_string(),
                    area: CodeArea {source: $parse_data.source.clone(), span: $area}
                } )
            };
        }
        #[allow(unused_macros)]
        macro_rules! tok {
            ($index:expr) => {
                &$parse_data.tokens[{
                    let le_index = (($pos as i32) + $index);
                    if le_index < 0 {0} else {le_index}
                } as usize].0
            }
        }
        #[allow(unused_macros)]
        macro_rules! span {
            ($index:expr) => {
                $parse_data.tokens[{
                    let le_index = (($pos as i32) + $index);
                    if le_index < 0 {0} else {le_index}
                } as usize].1
            }
        }
        #[allow(unused_macros)]
        macro_rules! span_ar {
            ($index:expr) => {
                CodeArea {
                    source: $parse_data.source.clone(),
                    span: span!($index)
                }
            }
        }
        // #[allow(unused_macros)]
        // macro_rules! ret {
        //     ($node_type:expr => $span:expr) => {
        //         return Ok((ASTNode { 
        //             node: $node_type,
        //             span: $span,
        //          }, $pos))
        //     };
        //     ($node_type:expr => $start:expr, $end:expr) => {
        //         return Ok((ASTNode { 
        //             node: $node_type,
        //             span: ($start, $end),
        //         }, $pos))
        //     };
        // }


        #[allow(unused_macros)]
        macro_rules! check_tok {
            ($token:ident else $expected:literal) => {
                if !matches!(tok!(0), Token::$token) {
                    expected_err!($expected, tok!(0), span!(0))
                }
                $pos += 1;
            };
            ($token:ident($val:ident) else $expected:literal) => {
                let $val;
                if let Token::$token(v) = tok!(0) {
                    $val = v.clone();
                } else { expected_err!($expected, tok!(0), span!(0)) }
                $pos += 1;
            };
            ($token:ident($val:ident):$sp:ident else $expected:literal) => {
                let $val;
                let $sp;
                if let (Token::$token(v), sp) = (tok!(0), span!(0)) {
                    $val = v.clone();
                    $sp = sp.clone();
                } else { expected_err!($expected, tok!(0), span!(0)) }
                $pos += 1;
            };
        }
        #[allow(unused_macros)]
        macro_rules! check_tok_static {
            ($token:ident else $expected:literal) => {
                if !matches!(tok!(0), Token::$token) {
                    expected_err!($expected, tok!(0), span!(0))
                }
            };
            ($token:ident($val:ident) else $expected:literal) => {
                let $val;
                if let Token::$token(v) = tok!(0) {
                    $val = v.clone();
                } else { expected_err!($expected, tok!(0), span!(0)) }
            };
            ($token:ident($val:ident):$sp:ident else $expected:literal) => {
                let $val;
                let $sp;
                if let (Token::$token(v), sp) = (tok!(0), span!(0)) {
                    $val = v.clone();
                    $sp = sp.clone();
                } else { expected_err!($expected, tok!(0), span!(0)) }
            };
        }

        
        #[allow(unused_macros)]
        macro_rules! skip_tok {
            ($token:ident) => {
                if matches!(tok!(0), Token::$token) {
                    $pos += 1;
                }
            };
        }
        #[allow(unused_macros)]
        macro_rules! skip_toks {
            ($token:ident) => {
                while matches!(tok!(0), Token::$token) {
                    $pos += 1;
                }
            };
        }
        #[allow(unused_macros)]
        macro_rules! while_tok {
            (== $token:ident: $code:block) => {
                loop {
                    match tok!(0) {
                        Token::$token => $code,
                        _ => break
                    }
                }
            };
            (!= $token:ident: $code:block) => {
                loop {
                    match tok!(0) {
                        Token::$token => break,
                        _ => $code
                    }
                }
                $pos += 1;
            };
        }
        #[allow(unused_macros)]
        macro_rules! if_tok {
            (== $token:ident: $code:block) => {
                match tok!(0) {
                    Token::$token => $code,
                    _ => (),
                }
            };
            (!= $token:ident: $code:block) => {
                match tok!(0) {
                    Token::$token => (),
                    _ => $code,
                }
            };
            (== $token:ident: $code:block else $else_code:block) => {
                match tok!(0) {
                    Token::$token => $code,
                    _ => $else_code,
                }
            };
            (!= $token:ident: $code:block else $else_code:block) => {
                match tok!(0) {
                    Token::$token => $else_code,
                    _ => $code,
                }
            };
        }

        #[allow(unused_macros)]
        macro_rules! parse {
            ($fn:ident => let $p:pat) => {
                let parsed = $fn($parse_data, $ast_data, $pos)?;
                $pos = parsed.1;
                let $p = parsed.0;
            };
            ($fn:ident => $v:ident) => {
                let parsed = $fn($parse_data, $ast_data, $pos)?;
                $pos = parsed.1;
                $v = parsed.0;
            };
            ($fn:ident ($arg:expr) => let $p:pat) => {
                let parsed = $fn($parse_data, $ast_data, $pos, $arg)?;
                $pos = parsed.1;
                let $p = parsed.0;
            };
            ($fn:ident ($arg:expr) => $v:ident) => {
                let parsed = $fn($parse_data, $ast_data, $pos, $arg)?;
                $pos = parsed.1;
                $v = parsed.0;
            };
        }
    };
}

#[derive(PartialEq, Debug)]
enum OpType {
    LeftAssoc,
    RightAssoc,
    Unary
}

macro_rules! operators {
    (
        $(
            $optype:ident <== [$($tok:ident)+],
        )*
    ) => {
        fn infix_prec(tok: &Token) -> usize {
            let mut prec = 0;
            $(
                match tok {
                    $(
                        Token::$tok => if OpType::$optype != OpType::Unary {return prec},
                    )+
                    _ => (),
                };
                prec += 1;
                format!("{}", prec);
            )*
            1000000
        }
        fn unary_prec(tok: &Token) -> usize {
            let mut prec = 0;
            $(
                match tok {
                    $(
                        Token::$tok => if OpType::$optype == OpType::Unary {return prec},
                    )+
                    _ => (),
                };
                prec += 1;
                format!("{}", prec);
            )*
            1000000
        }
        fn is_unary(tok: &Token) -> bool {
            let mut utoks = vec![];
            $(
                if OpType::$optype == OpType::Unary {
                    $(
                        utoks.push( Token::$tok );
                    )+
                }
            )*
            return utoks.contains( tok );
        }
        fn prec_amount() -> usize {
            let mut amount = 0;
            $(
                amount += 1;
                format!("{:?}", OpType::$optype);
            )*
            amount
        }
        fn prec_type(mut prec: usize) -> OpType {
            $(
                if prec == 0 {
                    return OpType::$optype;
                }
                prec -= 1;
                format!("{}", prec);
            )*
            unreachable!()
        }
    };
}


operators!(
    // RightAssoc  <==  [ Assign ],
    // RightAssoc  <==  [ PlusEq MinusEq MultEq DivEq ModEq PowEq EuclModEq ],
    // LeftAssoc   <==  [ And Or ],
    // LeftAssoc   <==  [ Pipe ],
    // Unary       <==  [ ExclMark ],
    // LeftAssoc   <==  [ Eq NotEq Greater GreaterEq Lesser LesserEq ],
    // LeftAssoc   <==  [ DoubleDot ],
    // Unary       <==  [ DoubleDot ],
    // Unary       <==  [ TripleDot ],
    LeftAssoc   <==  [ Plus Minus ],
    Unary       <==  [ Minus ],
    LeftAssoc   <==  [ Mult Div Mod ],
    RightAssoc  <==  [ Pow ],
    // LeftAssoc   <==  [ As ],
);








fn parse_unit(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let start = span!(0);

    match tok!(0) {
        Token::Int(n) => Ok((ast_data.insert_expr(
            Expression::Literal(Literal::Int(*n)),
            span_ar!(0)
        ), pos + 1)),
        Token::Float(n) => Ok((ast_data.insert_expr(
            Expression::Literal(Literal::Float(*n)),
            span_ar!(0)
        ), pos + 1)),
        Token::True => Ok((ast_data.insert_expr(
            Expression::Literal(Literal::Bool(true)),
            span_ar!(0)
        ), pos + 1)),
        Token::False => Ok((ast_data.insert_expr(
            Expression::Literal(Literal::Bool(false)),
            span_ar!(0)
        ), pos + 1)),
        Token::String(s) => Ok((ast_data.insert_expr(
            Expression::Literal(Literal::String(s.into())),
            span_ar!(0)
        ), pos + 1)),
        Token::Ident(name) => Ok((ast_data.insert_expr(
            Expression::Var(name.into()),
            span_ar!(0),
        ), pos + 1)),

        Token::LParen => {
            pos += 1;
            if_tok!(== RParen: {
                Ok((ast_data.insert_expr(
                    Expression::Empty,
                    parse_data.source.to_area( (start.0, span!(-1).1) )
                ), pos + 1))
            } else {
                parse!(parse_expr => let value);
                check_tok!(RParen else ")");
                Ok((value, pos))
            })
        }

        Token::LSqBracket => {
            pos += 1;

            let mut elements = vec![];
            while_tok!(!= RSqBracket: {
                parse!(parse_expr => let elem);
                elements.push(elem);
                if !matches!(tok!(0), Token::RSqBracket | Token::Comma) {
                    expected_err!("] or ,", tok!(0), span!(0))
                }
                skip_tok!(Comma);
            });

            Ok((ast_data.insert_expr(
                Expression::Array(elements),
                parse_data.source.to_area( (start.0, span!(-1).1) )
            ), pos))
        }

        unary_op if is_unary(unary_op) => {
            pos += 1;
            let prec = unary_prec(unary_op);
            let mut next_prec = if prec + 1 < prec_amount() {prec + 1} else {1000000};
            while next_prec != 1000000 {
                if prec_type(next_prec) == OpType::Unary {
                    next_prec += 1
                } else {
                    break
                }
                if next_prec == prec_amount() { next_prec = 1000000 }
            }
            let value;
            if next_prec != 1000000 { parse!(parse_op(next_prec) => value); }
            else { parse!(parse_value => value); }

            Ok((ast_data.insert_expr(
                Expression::Unary(unary_op.clone(), value),
                parse_data.source.to_area( (start.0, span!(-1).1) )
            ), pos))
        }

        other => expected_err!("expression", other, span!(0)),
    }

    // match ast_data[ASTKey::default()].0.into_expr() {
    //     Expression::Literal(_) => todo!(),
    //     Expression::Op(_, _, _) => todo!(),
    //     Expression::Unary(_, _) => todo!(),
    // }

    // todo!()
}





fn parse_value(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);
    
    parse!(parse_unit => let mut value);
    let start = ast_data.area(value).span;
    
    while matches!(tok!(0),
        Token::LSqBracket
    ) {
        match tok!(0) {
            Token::LSqBracket => {
                pos += 1;
                parse!(parse_expr => let index);
                check_tok!(RSqBracket else "]");
                value = ast_data.insert_expr(
                    Expression::Index { base: value, index },
                    parse_data.source.to_area( (start.0, span!(-1).1) )
                );
            },
            _ => unreachable!(),
        }
    }

    Ok((value, pos))

}


fn parse_expr(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    pos: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    

    parse_op(parse_data, ast_data, pos, 0)
}



fn parse_op(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
    prec: usize,
) -> Result<(ExprKey, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let mut next_prec = if prec + 1 < prec_amount() {prec + 1} else {1000000};
    while next_prec != 1000000 {
        if prec_type(next_prec) == OpType::Unary {
            next_prec += 1
        } else {
            break
        }
        if next_prec == prec_amount() {next_prec = 1000000};
    }
    let mut left;
    if next_prec != 1000000 { parse!(parse_op(next_prec) => left); }
    else { parse!(parse_value => left); }

    while infix_prec(tok!(0)) == prec {
        let op = tok!(0).clone();
        pos += 1;
        let right;
        if prec_type(prec) == OpType::LeftAssoc {
            if next_prec != 1000000 { parse!(parse_op(next_prec) => right); }
            else { parse!(parse_value => right); }
        } else {
            parse!(parse_op(prec) => right);
        }
        let (left_span, right_span) = (ast_data.area(left).span, ast_data.area(right).span);
        left = ast_data.insert_expr(
            Expression::Op(left, op, right),
            parse_data.source.to_area( ( left_span.0, right_span.1 ) )
        );
    }
    Ok((left, pos))

}


fn parse_statement(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(StmtKey, usize), SyntaxError> {

    parse_util!(parse_data, ast_data, pos);
    let start = span!(0);

    macro_rules! expr_stmt {
        () => {
            {
                parse!(parse_expr => let value);
                Statement::Expr(value)
            }
        };
    }

    let stmt = match tok!(0) {
        Token::Let => {
            pos += 1;
            check_tok!(Ident(var_name) else "variable name");
            check_tok!(Assign else "=");
            parse!(parse_expr => let value);
            Statement::Declaration(var_name, value)
        }
        Token::If => {
            pos += 1;

            let mut branches = vec![];
            let mut else_branch = None;
            
            parse!(parse_expr => let cond);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            branches.push((cond, code));

            while let Token::Else = tok!(0) {
                pos += 1;
                if_tok!(== If: {
                    pos += 1;
                    parse!(parse_expr => let cond);
                    check_tok!(LBracket else "{");
                    parse!(parse_statements => let code);
                    check_tok!(RBracket else "}");
                    branches.push((cond, code));
                } else {
                    check_tok!(LBracket else "{");
                    parse!(parse_statements => let temp); else_branch = Some(temp);
                    check_tok!(RBracket else "}");
                    break;
                })
            }

            Statement::If {
                branches,
                else_branch,
            }

        }
        Token::While => {
            pos += 1;
            parse!(parse_expr => let cond);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            Statement::While {
                cond,
                code,
            }
        }
        Token::For => {
            pos += 1;
            check_tok!(Ident(var) else "variable name");
            check_tok!(In else "in");
            parse!(parse_expr => let iterator);
            check_tok!(LBracket else "{");
            parse!(parse_statements => let code);
            check_tok!(RBracket else "}");
            Statement::For {
                code,
                var,
                iterator,
            }
        }
        _ => expr_stmt!(),
    };

    if !matches!(tok!(-1), Token::RBracket) {
        check_tok!(Eol else ';');
    }
    skip_toks!(Eol);

    Ok((ast_data.insert_stmt(
        stmt,
        parse_data.source.to_area( (start.0, span!(-1).1) )
    ), pos))

}


fn parse_statements(
    parse_data: &ParseData,
    ast_data: &mut ASTData,
    mut pos: usize,
) -> Result<(Statements, usize), SyntaxError> {
    parse_util!(parse_data, ast_data, pos);

    let mut statements = vec![];

    while !matches!(tok!(0), Token::Eof | Token::RBracket) {
        parse!(parse_statement => let stmt);
        statements.push(stmt);
    };

    Ok((statements, pos))
}


pub fn parse(
    parse_data: &ParseData,
    ast_data: &mut ASTData
) -> Result<Statements, SyntaxError> {
    let mut pos = 0;
    parse_util!(parse_data, ast_data, pos);
    
    parse!(parse_statements => let stmts);
    // check_tok_static!(Eof else "end of file");
    Ok(stmts)
}
