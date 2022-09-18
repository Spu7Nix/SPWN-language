pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod parser_util;

use lexer::Token;

impl From<Token> for &str {
    fn from(tok: Token) -> Self {
        use Token::*;
        match tok {
            Int => "int literal",
            Float => "float literal",
            Id => "ID literal",
            String => "string literal",
            TypeIndicator => "type indicator",
            Let => "let",
            Mut => "mut",
            Ident => "identifier",
            Error => "invalid",
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
            MuLte => "*=",
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
            PatAnd => "&",
            PatOr => "|",
            And => "&&",
            Or => "||",
        }
    }
}
#[cfg(test)]
mod test_util;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::RwLock;

    use super::ast::{Expression as Ex, Statement as St, *};
    use super::error::SyntaxError;
    use super::lexer::Token;
    use super::parser::Parser;
    use crate::leveldata::object_data::ObjectMode;
    use crate::parsing::test_util::DeepEq;
    use crate::sources::SpwnSource;
    use crate::*;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref SOURCE: SpwnSource = SpwnSource::File(PathBuf::from("<tests>"));
        static ref DATA: RwLock<ASTData> = RwLock::new(ASTData::new(SOURCE.clone()));
    }

    type Result<T> = std::result::Result<T, SyntaxError>;

    fn parse(code: &str) -> Result<Statements> {
        let mut parser = Parser::new(code, SOURCE.clone());

        let mut data = DATA.write().unwrap();

        parser.parse(&mut *data)
    }

    #[test]
    fn test_int() -> Result<()> {
        let t = parse("1;")?;
        expr_eq!(t, expr!(1));

        let t = parse("0;")?;
        expr_eq!(t, expr!(0));

        let t = parse("1337;")?;
        expr_eq!(t, expr!(1337));

        let t = parse("10_000_000;")?;
        expr_eq!(t, expr!(10000000));

        let t = parse("1_0_00;")?;
        expr_eq!(t, expr!(1000));

        let t = parse("0b0101011;")?;
        expr_eq!(t, expr!(43));

        Ok(())
    }

    #[test]
    fn test_float() -> Result<()> {
        let t = parse("0.123;")?;
        expr_eq!(t, expr!(0.123));

        let t = parse("1.234;")?;
        expr_eq!(t, expr!(1.234));

        let t = parse("0.000;")?;
        expr_eq!(t, expr!(0.000));

        let t = parse("1_000.30_04;")?;
        expr_eq!(t, expr!(1000.3004));

        Ok(())
    }

    #[test]
    fn test_bool() -> Result<()> {
        let t = parse("true;")?;
        expr_eq!(t, expr!(true));

        let t = parse("false;")?;
        expr_eq!(t, expr!(false));

        Ok(())
    }

    #[test]
    fn test_id() -> Result<()> {
        let t = parse("19g;")?;
        expr_eq!(
            t,
            expr!(Ex::Id {
                class: IdClass::Group,
                value: Some(19)
            })
        );

        let t = parse("?b;")?;
        expr_eq!(
            t,
            expr!(Ex::Id {
                class: IdClass::Block,
                value: None
            })
        );

        let t = parse("114i;")?;
        expr_eq!(
            t,
            expr!(Ex::Id {
                class: IdClass::Item,
                value: Some(114)
            })
        );

        let t = parse("?c;")?;
        expr_eq!(
            t,
            expr!(Ex::Id {
                class: IdClass::Color,
                value: None
            })
        );

        Ok(())
    }

    #[test]
    fn test_string() -> Result<()> {
        let t = parse("\"test123\";")?;
        expr_eq!(t, expr!("test123"));

        let t = parse("'test123';")?;
        expr_eq!(t, expr!("test123"));

        let t = parse("\"newline\n\";")?;
        expr_eq!(t, expr!("newline\n"));

        let t = parse("\"\\\"\";")?;
        expr_eq!(t, expr!("\""));

        let t = parse("\"\\\\\";")?;
        expr_eq!(t, expr!("\\"));

        let t = parse("\"\\u{09DE}\";")?;
        expr_eq!(t, expr!("\u{09DE}"));

        let t = parse("\"🐠\";")?;
        expr_eq!(t, expr!("🐠"));

        //todo!("string flags");

        Ok(())
    }

    #[test]
    fn test_ops() -> Result<()> {
        let t = parse("10 + 20;")?;
        expr_eq!(t, expr_op!(expr!(10), Token::Plus, expr!(20)));

        let t = parse("13.37 ^ 12.06;")?;
        expr_eq!(t, expr_op!(expr!(13.37), Token::Pow, expr!(12.06)));

        let t = parse("1 == 2;")?;
        expr_eq!(t, expr_op!(expr!(1), Token::Eq, expr!(2)));

        let t = parse("10.7 <= 4;")?;
        expr_eq!(t, expr_op!(expr!(10.7), Token::Lte, expr!(4)));

        // let t = parse("1..10;")?;
        // expr_eq!(t, expr_op!(expr!(1), Token::DotDot, expr!(10)));

        // let t = parse("1.0002 %= 2;")?;
        // expr_eq!(t, expr_op!(expr!(1.0002), Token::ModEq, expr!(2)));

        let t = parse("-10;")?;
        expr_eq!(t, expr_op!(Token::Minus, expr!(10)));

        //todo!("opeq + bin ops + ++ -- ..");

        Ok(())
    }

    #[test]
    fn test_sym() -> Result<()> {
        let t = parse("i;")?;
        expr_eq!(t, expr_sym!("i"));

        let t = parse("_i;")?;
        expr_eq!(t, expr_sym!("_i"));

        let t = parse("_i1;")?;
        expr_eq!(t, expr_sym!("_i1"));

        let t = parse("@test;")?;
        expr_eq!(t, expr_sym!(@"test"));

        let t = parse("@__;")?;
        expr_eq!(t, expr_sym!(@"__"));

        let t = parse("@_1;")?;
        expr_eq!(t, expr_sym!(@"_1"));

        Ok(())
    }

    #[test]
    fn test_array() -> Result<()> {
        let t = parse("[10, 'a', true, test];")?;
        expr_eq!(
            t,
            expr_arr!(expr!(10), expr!("a"), expr!(true), expr_sym!("test"))
        );

        let t = parse("[{a, b: 13.37}, @a];")?;
        expr_eq!(
            t,
            expr_arr!(expr_dict!("a", "b": expr!(13.37)), expr_sym!(@"a"))
        );

        let t = parse("[1, [2, [3]]];")?;
        expr_eq!(
            t,
            expr_arr!(expr!(1), expr_arr!(expr!(2), expr_arr!(expr!(3))))
        );

        Ok(())
    }

    #[test]
    fn test_dictlike() -> Result<()> {
        let t = parse("{a: 'b', b: 30.0_5};")?;
        expr_eq!(t, expr_dict!("a": expr!("b"), "b": expr!(30.05)));

        let t = parse("{a: {b: {c: 10}}};")?;
        expr_eq!(
            t,
            expr_dict!("a": expr_dict!("b": expr_dict!("c": expr!(10))))
        );

        let t = parse("{a, b};")?;
        expr_eq!(t, expr_dict!("a", "b"));

        let t = parse("@a::{i: 10, x: '20', a};")?;
        expr_eq!(
            t,
            expr!(Ex::Instance(
                expr_key!(expr_sym!(@"a")),
                expr_key!(expr_dict!("i": expr!(10), "x": expr!("20"), "a"))
            ))
        );

        let t = parse("obj { OBJ_ID: 2, HVS: '0a0a0a', X: 20.4 };")?;
        expr_eq!(
            t,
            expr!(Ex::Obj(
                ObjectMode::Object,
                spanned_vec![
                    (expr_key!(expr_sym!("OBJ_ID")), expr_key!(expr!(2))),
                    (expr_key!(expr_sym!("HVS")), expr_key!(expr!("0a0a0a"))),
                    (expr_key!(expr_sym!("X")), expr_key!(expr!(20.4)))
                ]
            ))
        );

        let t = parse("trigger { Y: 5000, GROUPS: 1g };")?;
        expr_eq!(
            t,
            expr!(Ex::Obj(
                ObjectMode::Trigger,
                spanned_vec![
                    (expr_key!(expr_sym!("Y")), expr_key!(expr!(5000))),
                    (
                        expr_key!(expr_sym!("GROUPS")),
                        expr_key!(expr!(Ex::Id {
                            class: IdClass::Group,
                            value: Some(1)
                        }))
                    )
                ]
            ))
        );

        //todo!("string keys");

        Ok(())
    }

    #[test]
    fn test_paths() -> Result<()> {
        let t = parse("a.x;")?;
        expr_eq!(
            t,
            expr!(Ex::Member {
                base: expr_key!(expr_sym!("a")),
                name: s!("x"),
            })
        );

        let t = parse("a.type;")?;
        expr_eq!(
            t,
            expr!(Ex::TypeOf {
                base: expr_key!(expr_sym!("a")),
            })
        );

        // let t = parse("@a::x;")?;
        // expr_eq!(
        //     t,
        //     expr!(Ex::Member {
        //         base: expr_key!(expr_sym!(@"a")),
        //         name: s!("x"),
        //     })
        // );

        let t = parse("a[0];")?;
        expr_eq!(
            t,
            expr!(Ex::Index {
                base: expr_key!(expr_sym!("a")),
                index: expr_key!(expr!(0)),
            })
        );

        // let t = parse("a[0:3];")?;
        // expr_eq!(
        //     t,
        //     expr!(Ex::Index {
        //         base: expr_key!(expr_sym!("a")),
        //         index: expr_key!(expr!(0)),
        //     })
        // );

        let t = parse("a();")?;
        expr_eq!(
            t,
            expr!(Ex::Call {
                base: expr_key!(expr_sym!("a")),
                params: vec![],
                named_params: vec![],
            })
        );

        let t = parse("a(10, true, 'test');")?;
        expr_eq!(
            t,
            expr!(Ex::Call {
                base: expr_key!(expr_sym!("a")),
                params: vec![
                    expr_key!(expr!(10)),
                    expr_key!(expr!(true)),
                    expr_key!(expr!("test"))
                ],
                named_params: vec![],
            })
        );

        let t = parse("a(10.0_4, x = true, something_long = test);")?;
        expr_eq!(
            t,
            expr!(Ex::Call {
                base: expr_key!(expr_sym!("a")),
                params: vec![expr_key!(expr!(10.04))],
                named_params: spanned_vec![
                    (s!("x"), expr_key!(expr!(true))),
                    (s!("something_long"), expr_key!(expr_sym!("test")))
                ],
            })
        );

        //todo!("associated members + index range + typeof?");

        Ok(())
    }

    #[test]
    fn test_macros() -> Result<()> {
        let t = parse("() {};")?;
        expr_eq!(
            t,
            expr!(Ex::Macro {
                args: vec![],
                ret_type: None,
                // `code` is ignored when comparing the expression
                code: MacroCode::Normal(vec![])
            })
        );

        let t = parse("() => {};")?;
        expr_eq!(
            t,
            expr!(Ex::Macro {
                args: vec![],
                ret_type: None,
                code: MacroCode::Normal(vec![])
            })
        );

        let t = parse("() -> @a => {};")?;
        expr_eq!(
            t,
            expr!(Ex::Macro {
                args: vec![],
                ret_type: Some(expr_key!(expr_sym!(@"a"))),
                code: MacroCode::Normal(vec![])
            })
        );

        let t = parse("(a, b: @a, c: @b = 10) -> @c {};")?;
        expr_eq!(
            t,
            expr!(Ex::Macro {
                args: spanned_vec![
                    (s!("a"), None, None),
                    (s!("b"), Some(expr_key!(expr_sym!(@"a"))), None),
                    (
                        s!("c"),
                        Some(expr_key!(expr_sym!(@"b"))),
                        Some(expr_key!(expr!(10)))
                    )
                ],
                ret_type: Some(expr_key!(expr_sym!(@"c"))),
                code: MacroCode::Normal(vec![])
            })
        );

        let t = parse("() -> _;")?;
        expr_eq!(
            t,
            expr!(Ex::MacroPattern {
                args: vec![],
                ret_type: expr_key!(expr_sym!("_"))
            })
        );

        let t = parse("(@a) -> @b;")?;
        expr_eq!(
            t,
            expr!(Ex::MacroPattern {
                args: vec![expr_key!(expr_sym!(@"a"))],
                ret_type: expr_key!(expr_sym!(@"b"))
            })
        );

        Ok(())
    }

    #[test]
    fn test_misc() -> Result<()> {
        let t = parse("a?;")?;
        expr_eq!(t, expr!(Ex::Maybe(Some(expr_key!(expr_sym!("a"))))));

        let t = parse("!{};")?;
        expr_eq!(t, expr!(Ex::TriggerFunc(vec![])));

        let t = parse("t!;")?;
        expr_eq!(t, expr!(Ex::TriggerFuncCall(expr_key!(expr_sym!("t")))));

        let t = parse("a if true else b;")?;
        expr_eq!(
            t,
            expr!(Ex::Ternary {
                cond: expr_key!(expr!(true)),
                if_true: expr_key!(expr_sym!("a")),
                if_false: expr_key!(expr_sym!("b"))
            })
        );

        //todo!("attributes")

        Ok(())
    }

    #[test]
    fn test_statements() -> Result<()> {
        let t = parse("let a = 10;")?;
        stmt_eq!(t, St::Let(s!("a"), expr_key!(expr!(10))));

        let t = parse("a = true;")?;
        stmt_eq!(t, St::Assign(s!("a"), expr_key!(expr!(true))));

        let t = parse("if a == b {} else {};")?;
        stmt_eq!(
            t,
            St::If {
                branches: vec![(
                    expr_key!(expr_op!(expr_sym!("a"), Token::Eq, expr_sym!("b"))),
                    vec![]
                )],
                else_branch: None
            }
        );

        let t = parse("try {} catch e {};")?;
        stmt_eq!(
            t,
            St::TryCatch {
                try_branch: vec![],
                catch: vec![],
                catch_var: s!("e")
            }
        );

        let t = parse("while true {};")?;
        stmt_eq!(
            t,
            St::While {
                cond: expr_key!(expr!(true)),
                code: vec![]
            }
        );

        let t = parse("for e in [1, 2, 3] {};")?;
        stmt_eq!(
            t,
            St::For {
                var: s!("e"),
                iterator: expr_key!(expr_arr!(expr!(1), expr!(2), expr!(3))),
                code: vec![]
            }
        );

        // let t = parse("for (k, v) in {a: 10, b: 20, c: 30} {};")?;
        // stmt_eq!(
        // );

        let t = parse("return;")?;
        stmt_eq!(t, St::Return(None));

        let t = parse("return true;")?;
        stmt_eq!(t, St::Return(Some(expr_key!(expr!(true)))));

        let t = parse("break;")?;
        stmt_eq!(t, St::Break);

        let t = parse("continue;")?;
        stmt_eq!(t, St::Continue);

        let t = parse("type @a;")?;
        stmt_eq!(t, St::TypeDef(s!("a")));

        let t = parse("impl @a { a: 10, b: false };")?;
        stmt_eq!(
            t,
            St::Impl(
                expr_key!(expr_sym!(@"a")),
                expr_key!(expr_dict!("a": expr!(10), "b": expr!(false)))
            )
        );

        //todo!("for loop var expansion");

        Ok(())
    }
}
