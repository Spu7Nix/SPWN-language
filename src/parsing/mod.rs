pub mod ast;
pub mod attributes;
pub mod error;
pub mod parser;
pub mod utils;

/// these tests must be ran with only a single thread (they are not thread safe)
/// cargo test

#[cfg(test)]
#[allow(clippy::from_over_into)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    use lasso::{Rodeo, Spur};

    use super::ast::{
        Ast, ExprNode, Expression as Ex, Statement as St, StmtNode, StringContent, StringType,
    };
    use super::parser::Parser;
    use super::utils::operators::{BinOp, UnaryOp};
    use crate::gd::ids::IDClass;
    use crate::gd::object_keys::ObjectKey;
    use crate::parsing::ast::{
        ImportType, MacroArg, MacroCode, ModuleImport, ObjKeyType, ObjectType, Pattern,
        PatternNode, Spanned,
    };
    use crate::parsing::attributes::FileAttribute;
    use crate::parsing::parser::ParseResult;
    use crate::parsing::utils::operators::AssignOp;
    use crate::sources::{CodeSpan, SpwnSource};
    use crate::util::Interner;
    use crate::RandomState;

    impl Into<ExprNode> for Ex {
        fn into(self) -> ExprNode {
            ExprNode {
                expr: Box::new(self),
                attributes: vec![],
                span: CodeSpan::internal(),
            }
        }
    }
    impl Into<StmtNode> for St {
        fn into(self) -> StmtNode {
            StmtNode {
                stmt: Box::new(self),
                attributes: vec![],
                span: CodeSpan::internal(),
            }
        }
    }

    type Pt = Pattern<Spur, PatternNode, ExprNode>;
    impl Into<PatternNode> for Pt {
        fn into(self) -> PatternNode {
            PatternNode {
                pat: Box::new(self),
                span: CodeSpan::internal(),
            }
        }
    }

    static mut INTERNER: Option<Rc<RefCell<Interner>>> = None;

    macro_rules! expr_eq {
        ($ast:ident,[$($exprs:expr),*]) => {
            let _exprs = &[$($exprs),*];
            for (i, stmt) in $ast.statements.iter().enumerate() {
                match &*stmt.stmt {
                    St::Expr(_e) => {
                        assert_eq!(_exprs[i], *_e.expr);
                    },
                    _ => unreachable!(),
                }
            }
        };

        ($ast:ident, $expr:expr) => {
            expr_eq!($ast, [$expr])
        };
    }

    macro_rules! stmt_eq {
        ($ast:ident,[$($stmts:expr),*]) => {
            let _stmts = &[$($stmts),*];
            for (i, stmt) in $ast.statements.iter().enumerate() {
                assert_eq!(&_stmts[i], &*stmt.stmt);
            }
        };

        ($ast:ident, $expr:expr) => {
            stmt_eq!($ast, [$expr])
        };
    }

    macro_rules! span {
        ($expr:expr) => {{
            Spanned {
                value: $expr,
                span: CodeSpan::internal(),
            }
        }};
    }

    macro_rules! spur {
        ($str:literal) => {{
            let v = unsafe { INTERNER.as_ref().unwrap() };
            let mut i = v.borrow_mut();
            i.get_or_intern($str)
        }};
    }

    macro_rules! string {
        ($str:literal $(, $is_bytes:literal)?) => {{
            Ex::String(StringType {
                s: StringContent::Normal( spur!($str) ),
                bytes: { false $(; $is_bytes)?},
            })
        }};
    }

    // will error if we add / remove expressions / statements
    // if this errors please add / remove the respective tests
    #[rustfmt::skip]
    const _: () = {
        match Ex::Empty {
            Ex::Int(_) => (),
            Ex::Float(_) => (),
            Ex::String(_) => (),
            Ex::Bool(_) => (),
            Ex::Id(_, _) => (),
            Ex::Op(_, _, _) => (),
            Ex::Unary(_, _) => (),
            Ex::Var(_) => (),
            Ex::Type(_) => (),
            Ex::Array(_) => (),
            Ex::Dict(_) => (),
            Ex::Maybe(_) => (),
            Ex::Is(_, _) => (),
            Ex::Index { .. } => (),
            Ex::Member { .. } => (),
            Ex::TypeMember { .. } => (),
            Ex::Associated { .. } => (),
            Ex::Call { .. } => (),
            Ex::Macro { .. } => (),
            Ex::TriggerFunc { .. } => (),
            Ex::TriggerFuncCall(_) => (),
            Ex::Ternary { .. } => (),
            Ex::Typeof(_) => (),
            Ex::Builtins => (),
            Ex::Empty => (),
            Ex::Epsilon => (),
            Ex::Import(_) => (),
            Ex::Instance { .. } => (),
            Ex::Obj(_, _) => (),
        }
        match St::Break {
            St::Expr(_) => (),
            St::Let(_, _) => (),
            St::AssignOp(_, _, _) => (),
            St::If { .. } => (),
            St::While { .. } => (),
            St::For { .. } => (),
            St::TryCatch { .. } => (),
            St::Arrow(_) => (),
            St::Return(_) => (),
            St::Break => (),
            St::Continue => (),
            St::TypeDef { .. } => (),
            St::ExtractImport(_) => (),
            St::Impl { .. } => (),
            St::Overload { .. } => (),
            St::Dbg(_) => (),
            St::Throw(_) => (),
        }
    };

    fn parse(code: &'static str) -> ParseResult<Ast> {
        unsafe {
            INTERNER = Some(Rc::new(RefCell::new(
                Rodeo::with_hasher(RandomState::new()),
            )))
        };

        let mut parser = Parser::new(code, SpwnSource::File("<test>".into()), unsafe {
            Rc::clone(INTERNER.as_ref().unwrap())
        });

        parser.parse()
    }

    #[test]
    fn test_file_attrs() -> ParseResult<()> {
        let t = parse("#![no_std, cache_output]")?;

        assert_eq!(
            &t.file_attributes,
            &[FileAttribute::NoStd, FileAttribute::CacheOutput]
        );

        Ok(())
    }

    #[test]
    fn test_int() -> ParseResult<()> {
        let t = parse("1")?;
        expr_eq!(t, Ex::Int(1));

        let t = parse("0")?;
        expr_eq!(t, Ex::Int(0));

        let t = parse("1337")?;
        expr_eq!(t, Ex::Int(1337));

        let t = parse("10_000_000")?;
        expr_eq!(t, Ex::Int(10000000));

        let t = parse("1_0_00")?;
        expr_eq!(t, Ex::Int(1000));

        let t = parse("0b0101011")?;
        expr_eq!(t, Ex::Int(43));

        let t = parse("0xDEAD_BEEF")?;
        expr_eq!(t, Ex::Int(3735928559));

        let t = parse("0o20")?;
        expr_eq!(t, Ex::Int(16));

        Ok(())
    }

    #[test]
    fn test_float() -> ParseResult<()> {
        let t = parse("1.0")?;
        expr_eq!(t, Ex::Float(1.0));

        let t = parse("0.034534")?;
        expr_eq!(t, Ex::Float(0.034534));

        let t = parse("13.3_7")?;
        expr_eq!(t, Ex::Float(13.37));

        Ok(())
    }

    #[test]
    fn test_string() -> ParseResult<()> {
        let t = parse(r#""test123""#)?;
        expr_eq!(t, string!("test123"));

        let t = parse("'test123'")?;
        expr_eq!(t, string!("test123"));

        let t = parse(r#""newline\n""#)?;
        expr_eq!(t, string!("newline\n"));

        let t = parse(r#""\"""#)?;
        expr_eq!(t, string!("\""));

        let t = parse(r#""\\";"#)?;
        expr_eq!(t, string!("\\"));

        let t = parse(r#""\u{09DE}";"#)?;
        expr_eq!(t, string!("\u{09DE}"));

        let t = parse(r#""🐠""#)?;
        expr_eq!(t, string!("🐠"));

        let t = parse(r#"r"abc""#)?;
        expr_eq!(t, string!("abc"));

        let t = parse(r##"r#"abc"#"##)?;
        expr_eq!(t, string!("abc"));

        let t = parse(r###"r##"abc"##"###)?;
        expr_eq!(t, string!("abc"));

        let t = parse(r#"b64"skrunkly""#)?;
        expr_eq!(t, string!("c2tydW5rbHk="));

        let t = parse(r#"u"\n\taa\n\t\tbb""#)?;
        expr_eq!(t, string!("aa\n\tbb"));

        let t = parse(r#"b"rawww""#)?;
        expr_eq!(t, string!("rawww", true));

        let t = parse(r##"b64_r#" "something" "#"##)?;
        expr_eq!(t, string!("ICJzb21ldGhpbmciIA=="));

        let t = parse(r#"u_b64"\n\taa\n\t\tbb""#)?;
        expr_eq!(t, string!("YWEKCWJi"));

        Ok(())
    }

    #[test]
    fn test_bool() -> ParseResult<()> {
        let t = parse("true")?;
        expr_eq!(t, Ex::Bool(true));

        let t = parse("false")?;
        expr_eq!(t, Ex::Bool(false));

        Ok(())
    }

    #[test]
    fn test_id() -> ParseResult<()> {
        let t = parse("19g")?;
        expr_eq!(t, Ex::Id(IDClass::Group, Some(19)));

        let t = parse("?b")?;
        expr_eq!(t, Ex::Id(IDClass::Block, None));

        let t = parse("114i")?;
        expr_eq!(t, Ex::Id(IDClass::Item, Some(114)));

        let t = parse("?c")?;
        expr_eq!(t, Ex::Id(IDClass::Color, None));

        Ok(())
    }

    #[test]
    fn test_ops() -> ParseResult<()> {
        // useless match so this errors if we ever add/remove ops
        // if this errors please add / remove the respective tests
        match BinOp::And {
            BinOp::Range => (),
            BinOp::In => (),
            BinOp::BinOr => (),
            BinOp::Or => (),
            BinOp::BinAnd => (),
            BinOp::And => (),
            BinOp::Eq => (),
            BinOp::Neq => (),
            BinOp::Gt => (),
            BinOp::Gte => (),
            BinOp::Lt => (),
            BinOp::Lte => (),
            BinOp::ShiftLeft => (),
            BinOp::ShiftRight => (),
            BinOp::Plus => (),
            BinOp::Minus => (),
            BinOp::Mult => (),
            BinOp::Div => (),
            BinOp::Mod => (),
            BinOp::Pow => (),
            BinOp::As => (),
        };

        let t = parse("5..100")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(5).into(), BinOp::Range, Ex::Int(100).into())
        );
        let t = parse("2..4..20")?;
        expr_eq!(
            t,
            Ex::Op(
                Ex::Op(Ex::Int(2).into(), BinOp::Range, Ex::Int(4).into()).into(),
                BinOp::Range,
                Ex::Int(20).into()
            )
        );

        let t = parse("1.2 in 2.2")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Float(1.2).into(), BinOp::In, Ex::Float(2.2).into())
        );

        let t = parse("0b01 | 0b10")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::BinOr, Ex::Int(2).into())
        );
        let t = parse("true || false")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Bool(true).into(), BinOp::Or, Ex::Bool(false).into())
        );

        let t = parse("0b10 & 0b01")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(2).into(), BinOp::BinAnd, Ex::Int(1).into())
        );
        let t = parse("false && false")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Bool(false).into(), BinOp::And, Ex::Bool(false).into())
        );

        let t = parse("0x9DE == true")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(2526).into(), BinOp::Eq, Ex::Bool(true).into())
        );
        let t = parse(r#""a" != 2"#)?;
        expr_eq!(
            t,
            Ex::Op(string!("a").into(), BinOp::Neq, Ex::Int(2).into())
        );
        let t = parse("5.3_2 > ?g")?;
        expr_eq!(
            t,
            Ex::Op(
                Ex::Float(5.32).into(),
                BinOp::Gt,
                Ex::Id(IDClass::Group, None).into()
            )
        );
        let t = parse("10 >= 5")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(10).into(), BinOp::Gte, Ex::Int(5).into(),)
        );
        let t = parse("10 < 5")?;
        expr_eq!(t, Ex::Op(Ex::Int(10).into(), BinOp::Lt, Ex::Int(5).into(),));
        let t = parse("10 <= 5")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(10).into(), BinOp::Lte, Ex::Int(5).into(),)
        );

        let t = parse("0b01 << 2")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::ShiftLeft, Ex::Int(2).into(),)
        );
        let t = parse("0b01 >> 2")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::ShiftRight, Ex::Int(2).into(),)
        );

        let t = parse("1 + 1")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::Plus, Ex::Int(1).into(),)
        );
        let t = parse("1 - 1")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::Minus, Ex::Int(1).into(),)
        );
        let t = parse("1 * 1")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(1).into(), BinOp::Mult, Ex::Int(1).into(),)
        );
        let t = parse("1 / 1")?;
        expr_eq!(t, Ex::Op(Ex::Int(1).into(), BinOp::Div, Ex::Int(1).into(),));
        let t = parse("1 % 1")?;
        expr_eq!(t, Ex::Op(Ex::Int(1).into(), BinOp::Mod, Ex::Int(1).into(),));
        let t = parse("1 ^ 1")?;
        expr_eq!(t, Ex::Op(Ex::Int(1).into(), BinOp::Pow, Ex::Int(1).into(),));

        let t = parse("2 as false")?;
        expr_eq!(
            t,
            Ex::Op(Ex::Int(2).into(), BinOp::As, Ex::Bool(false).into(),)
        );

        Ok(())
    }

    #[test]
    fn test_unary_op() -> ParseResult<()> {
        // useless match so this errors if we ever add/remove ops
        // if this errors please add / remove the respective tests
        match UnaryOp::Minus {
            UnaryOp::ExclMark => (),
            UnaryOp::Minus => (),
        }

        let t = parse("!0")?;
        expr_eq!(t, Ex::Unary(UnaryOp::ExclMark, Ex::Int(0).into()));

        let t = parse("-10.2")?;
        expr_eq!(t, Ex::Unary(UnaryOp::Minus, Ex::Float(10.2).into()));

        Ok(())
    }

    #[test]
    fn test_var() -> ParseResult<()> {
        let t = parse("a")?;
        expr_eq!(t, Ex::Var(spur!("a")));

        let t = parse("p_b")?;
        expr_eq!(t, Ex::Var(spur!("p_b")));

        let t = parse("a123")?;
        expr_eq!(t, Ex::Var(spur!("a123")));

        Ok(())
    }

    #[test]
    fn test_type() -> ParseResult<()> {
        let t = parse("@a")?;
        expr_eq!(t, Ex::Type(spur!("a")));

        let t = parse("@p_b")?;
        expr_eq!(t, Ex::Type(spur!("p_b")));

        let t = parse("@a123")?;
        expr_eq!(t, Ex::Type(spur!("a123")));

        Ok(())
    }

    #[test]
    fn test_array() -> ParseResult<()> {
        let t = parse(r#"[10,]"#)?;
        expr_eq!(t, Ex::Array(vec![Ex::Int(10).into(),]));

        let t = parse(r#"[10, a, true, "aa", 1.2, @a, [1, 2], a in b, !false]"#)?;
        expr_eq!(
            t,
            Ex::Array(vec![
                Ex::Int(10).into(),
                Ex::Var(spur!("a")).into(),
                Ex::Bool(true).into(),
                string!("aa").into(),
                Ex::Float(1.2).into(),
                Ex::Type(spur!("a")).into(),
                Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into(),]).into(),
                Ex::Op(
                    Ex::Var(spur!("a")).into(),
                    BinOp::In,
                    Ex::Var(spur!("b")).into()
                )
                .into(),
                Ex::Unary(UnaryOp::ExclMark, Ex::Bool(false).into()).into(),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_dict() -> ParseResult<()> {
        let t = parse(
            r#"{
            a,
            b: 10,
            "c": "a",
        }"#,
        )?;
        expr_eq!(
            t,
            Ex::Dict(vec![
                (span!(spur!("a")), None, false),
                (span!(spur!("b")), Some(Ex::Int(10).into()), false),
                (span!(spur!("c")), Some(string!("a").into()), false),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_maybe() -> ParseResult<()> {
        let t = parse("?")?;
        expr_eq!(t, Ex::Maybe(None));

        let t = parse("10?")?;
        expr_eq!(t, Ex::Maybe(Some(Ex::Int(10).into())));

        Ok(())
    }

    #[test]
    fn test_is_pattern() -> ParseResult<()> {
        // useless match so this errors if we ever add/remove patterns
        // if this errors please add / remove the respective tests
        match Pt::Any {
            Pattern::Any => (),
            Pattern::Type(_) => (),
            Pattern::Either(..) => (),
            Pattern::Both(..) => (),
            Pattern::Eq(_) => (),
            Pattern::Neq(_) => (),
            Pattern::Lt(_) => (),
            Pattern::Lte(_) => (),
            Pattern::Gt(_) => (),
            Pattern::Gte(_) => (),
            Pattern::MacroPattern { .. } => (),
        };

        let e: ExprNode = Ex::Int(1).into();

        let t = parse("1 is _")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Any.into()));

        let t = parse("1 is @test")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Type(spur!("test")).into()));

        let t = parse("1 is @int|@float")?;
        expr_eq!(
            t,
            Ex::Is(
                e.clone(),
                Pt::Either(
                    Pt::Type(spur!("int")).into(),
                    Pt::Type(spur!("float")).into(),
                )
                .into()
            )
        );

        let t = parse("1 is ==1")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Eq(Ex::Int(1).into()).into()));
        let t = parse("1 is !=1")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Neq(Ex::Int(1).into()).into()));
        let t = parse("1 is <1")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Lt(Ex::Int(1).into()).into()));
        let t = parse("1 is <=1")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Lte(Ex::Int(1).into()).into()));
        let t = parse("1 is >1")?;
        expr_eq!(t, Ex::Is(e.clone(), Pt::Gt(Ex::Int(1).into()).into()));
        let t = parse("1 is >=1")?;
        expr_eq!(t, Ex::Is(e, Pt::Gte(Ex::Int(1).into()).into()));

        Ok(())
    }

    #[test]
    fn test_index() -> ParseResult<()> {
        let t = parse("a[200]")?;
        expr_eq!(
            t,
            Ex::Index {
                base: Ex::Var(spur!("a")).into(),
                index: Ex::Int(200).into()
            }
        );

        Ok(())
    }

    #[test]
    fn test_member() -> ParseResult<()> {
        let t = parse("foo.bar")?;
        expr_eq!(
            t,
            Ex::Member {
                base: Ex::Var(spur!("foo")).into(),
                name: span!(spur!("bar")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_type_member() -> ParseResult<()> {
        let t = parse("foo.@bar")?;
        expr_eq!(
            t,
            Ex::TypeMember {
                base: Ex::Var(spur!("foo")).into(),
                name: span!(spur!("bar")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_associated_member() -> ParseResult<()> {
        let t = parse("@foo::bar")?;
        expr_eq!(
            t,
            Ex::Associated {
                base: Ex::Type(spur!("foo")).into(),
                name: span!(spur!("bar")),
            }
        );

        Ok(())
    }

    #[test]
    fn test_call() -> ParseResult<()> {
        let t = parse("a()")?;
        expr_eq!(
            t,
            Ex::Call {
                base: Ex::Var(spur!("a")).into(),
                params: vec![],
                named_params: vec![],
            }
        );

        let t = parse("a(10)")?;
        expr_eq!(
            t,
            Ex::Call {
                base: Ex::Var(spur!("a")).into(),
                params: vec![Ex::Int(10).into()],
                named_params: vec![],
            }
        );

        let t = parse("a(foo = 20)")?;
        expr_eq!(
            t,
            Ex::Call {
                base: Ex::Var(spur!("a")).into(),
                params: vec![],
                named_params: vec![(span!(spur!("foo")), Ex::Int(20).into())],
            }
        );

        let t = parse("a(10, 20, foo = 30)")?;
        expr_eq!(
            t,
            Ex::Call {
                base: Ex::Var(spur!("a")).into(),
                params: vec![Ex::Int(10).into(), Ex::Int(20).into()],
                named_params: vec![(span!(spur!("foo")), Ex::Int(30).into())],
            }
        );

        Ok(())
    }

    #[test]
    fn test_macro() -> ParseResult<()> {
        let t = parse("(a, b: @int, c = 20, d: @bool = true) {}")?;
        expr_eq!(
            t,
            Ex::Macro {
                args: vec![
                    MacroArg::Single {
                        name: span!(spur!("a")),
                        pattern: None,
                        default: None,
                        is_ref: false,
                    },
                    MacroArg::Single {
                        name: span!(spur!("b")),
                        pattern: Some(PatternNode {
                            pat: Box::new(Pt::Type(spur!("int"))),
                            span: CodeSpan::internal(),
                        }),
                        default: None,
                        is_ref: false,
                    },
                    MacroArg::Single {
                        name: span!(spur!("c")),
                        pattern: None,
                        default: Some(ExprNode {
                            expr: Box::new(Ex::Int(20)),
                            attributes: vec![],
                            span: CodeSpan::internal(),
                        }),
                        is_ref: false,
                    },
                    MacroArg::Single {
                        name: span!(spur!("d")),
                        pattern: Some(PatternNode {
                            pat: Box::new(Pt::Type(spur!("bool"))),
                            span: CodeSpan::internal(),
                        }),
                        default: Some(ExprNode {
                            expr: Box::new(Ex::Bool(true)),
                            attributes: vec![],
                            span: CodeSpan::internal(),
                        }),
                        is_ref: false,
                    }
                ],
                ret_type: None,
                code: MacroCode::Normal(vec![])
            }
        );

        let t = parse("(&a) {}")?;
        expr_eq!(
            t,
            Ex::Macro {
                args: vec![MacroArg::Single {
                    name: span!(spur!("a")),
                    pattern: None,
                    default: None,
                    is_ref: true,
                },],
                ret_type: None,
                code: MacroCode::Normal(vec![])
            }
        );

        let t = parse("(...a, ...b: @int) {}")?;
        expr_eq!(
            t,
            Ex::Macro {
                args: vec![
                    MacroArg::Spread {
                        name: span!(spur!("a")),
                        pattern: None,
                    },
                    MacroArg::Spread {
                        name: span!(spur!("b")),
                        pattern: Some(PatternNode {
                            pat: Box::new(Pt::Type(spur!("int"))),
                            span: CodeSpan::internal(),
                        }),
                    }
                ],
                ret_type: None,
                code: MacroCode::Normal(vec![])
            }
        );

        let t = parse("() -> @string {}")?;
        expr_eq!(
            t,
            Ex::Macro {
                args: vec![],
                ret_type: Some(Ex::Type(spur!("string")).into()),
                code: MacroCode::Normal(vec![])
            }
        );

        Ok(())
    }

    #[test]
    fn test_trigger_func() -> ParseResult<()> {
        let t = parse(
            "!{
            a = 10
        }",
        )?;
        expr_eq!(
            t,
            Ex::TriggerFunc {
                attributes: vec![],
                code: vec![St::AssignOp(
                    Ex::Var(spur!("a")).into(),
                    AssignOp::Assign,
                    Ex::Int(10).into()
                )
                .into()]
            }
        );

        let t = parse("a!")?;
        expr_eq!(t, Ex::TriggerFuncCall(Ex::Var(spur!("a")).into()));

        Ok(())
    }

    #[test]
    fn test_ternary() -> ParseResult<()> {
        let t = parse("1 if true else 2")?;
        expr_eq!(
            t,
            Ex::Ternary {
                cond: Ex::Bool(true).into(),
                if_true: Ex::Int(1).into(),
                if_false: Ex::Int(2).into(),
            }
        );

        Ok(())
    }

    #[test]
    fn test_typeof() -> ParseResult<()> {
        let t = parse("2.type")?;
        expr_eq!(t, Ex::Typeof(Ex::Int(2).into()));

        Ok(())
    }

    #[test]
    fn test_import() -> ParseResult<()> {
        let t = parse(r#"import foobar"#)?;
        expr_eq!(t, Ex::Import(ImportType::Library("foobar".into())));

        let t = parse(r#"import "foobar.spwn""#)?;
        expr_eq!(
            t,
            Ex::Import(ImportType::Module(
                "foobar.spwn".into(),
                ModuleImport::Regular
            ))
        );

        Ok(())
    }

    #[test]
    fn test_instance() -> ParseResult<()> {
        let t = parse("@foo::{a: 10, b}")?;
        expr_eq!(
            t,
            Ex::Instance {
                base: Ex::Type(spur!("foo")).into(),
                items: vec![
                    (span!(spur!("a")), Some(Ex::Int(10).into()), false,),
                    (span!(spur!("b")), None, false),
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_obj() -> ParseResult<()> {
        let t = parse(
            "obj {
            OBJ_ID: 10,
            GROUPS: [20g],
            5: false
        }",
        )?;
        expr_eq!(
            t,
            Ex::Obj(
                ObjectType::Object,
                vec![
                    (
                        span!(ObjKeyType::Name(ObjectKey::ObjId)),
                        Ex::Int(10).into()
                    ),
                    (
                        span!(ObjKeyType::Name(ObjectKey::Groups)),
                        Ex::Array(vec![Ex::Id(IDClass::Group, Some(20)).into()]).into()
                    ),
                    (span!(ObjKeyType::Num(5)), Ex::Bool(false).into())
                ]
            )
        );

        let t = parse(
            "trigger {
            OBJ_ID: 10,
        }",
        )?;
        expr_eq!(
            t,
            Ex::Obj(
                ObjectType::Trigger,
                vec![(
                    span!(ObjKeyType::Name(ObjectKey::ObjId)),
                    Ex::Int(10).into()
                )],
            )
        );

        Ok(())
    }

    #[test]
    fn test_misc() -> ParseResult<()> {
        let t = parse("$")?;
        expr_eq!(t, Ex::Builtins);

        let t = parse("()")?;
        expr_eq!(t, Ex::Empty);

        let t = parse("ε")?;
        expr_eq!(t, Ex::Epsilon);

        Ok(())
    }

    #[test]
    fn test_let() -> ParseResult<()> {
        let t = parse("let a = 10")?;
        stmt_eq!(t, St::Let(Ex::Var(spur!("a")).into(), Ex::Int(10).into()));

        Ok(())
    }

    #[test]
    fn test_assign_op() -> ParseResult<()> {
        // useless match so this errors if we ever add/remove ops
        // if this errors please add / remove the respective tests
        match AssignOp::Assign {
            AssignOp::Assign => (),
            AssignOp::PlusEq => (),
            AssignOp::MinusEq => (),
            AssignOp::MultEq => (),
            AssignOp::DivEq => (),
            AssignOp::PowEq => (),
            AssignOp::ModEq => (),
            AssignOp::BinAndEq => (),
            AssignOp::BinOrEq => (),
            AssignOp::ShiftLeftEq => (),
            AssignOp::ShiftRightEq => (),
        }

        let t = parse("a = 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::Assign,
                Ex::Int(1).into()
            )
        );

        let t = parse("a += 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::PlusEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a -= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::MinusEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a *= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::MultEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a /= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::DivEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a ^= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::PowEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a %= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::ModEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a &= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::BinAndEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a |= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::BinOrEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a <<= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::ShiftLeftEq,
                Ex::Int(1).into()
            )
        );

        let t = parse("a >>= 1")?;
        stmt_eq!(
            t,
            St::AssignOp(
                Ex::Var(spur!("a")).into(),
                AssignOp::ShiftRightEq,
                Ex::Int(1).into()
            )
        );

        Ok(())
    }

    #[test]
    fn test_if() -> ParseResult<()> {
        let t = parse("if false {} else if true {} else {}")?;
        stmt_eq!(
            t,
            St::If {
                branches: vec![
                    (Ex::Bool(false).into(), vec![]),
                    (Ex::Bool(true).into(), vec![])
                ],
                else_branch: Some(vec![])
            }
        );

        let t = parse("if false {}")?;
        stmt_eq!(
            t,
            St::If {
                branches: vec![(Ex::Bool(false).into(), vec![]),],
                else_branch: None
            }
        );

        Ok(())
    }

    #[test]
    fn test_while() -> ParseResult<()> {
        let t = parse("while true {}")?;
        stmt_eq!(
            t,
            St::While {
                cond: Ex::Bool(true).into(),
                code: vec![]
            }
        );

        Ok(())
    }

    #[test]
    fn test_for() -> ParseResult<()> {
        let t = parse("for a in [1, 2] {}")?;
        stmt_eq!(
            t,
            St::For {
                iter_var: Ex::Var(spur!("a")).into(),
                iterator: Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into()]).into(),
                code: vec![]
            }
        );

        Ok(())
    }

    #[test]
    fn test_try_catch() -> ParseResult<()> {
        let t = parse("try {} catch e {} catch {}")?;
        stmt_eq!(
            t,
            St::TryCatch {
                try_code: vec![],
                branches: vec![(Ex::Var(spur!("e")).into(), vec![])],
                catch_all: Some(vec![]),
            }
        );

        let t = parse("try {} catch e {}")?;
        stmt_eq!(
            t,
            St::TryCatch {
                try_code: vec![],
                branches: vec![(Ex::Var(spur!("e")).into(), vec![])],
                catch_all: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_arrow() -> ParseResult<()> {
        let t = parse("-> let a = 10")?;
        stmt_eq!(
            t,
            St::Arrow(Box::new(
                St::Let(Ex::Var(spur!("a")).into(), Ex::Int(10).into()).into()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_control_flow() -> ParseResult<()> {
        let t = parse("return 20")?;
        stmt_eq!(t, St::Return(Some(Ex::Int(20).into())));

        let t = parse("return")?;
        stmt_eq!(t, St::Return(None));

        let t = parse("break")?;
        stmt_eq!(t, St::Break);

        let t = parse("continue")?;
        stmt_eq!(t, St::Continue);

        Ok(())
    }

    #[test]
    fn test_typedef() -> ParseResult<()> {
        let t = parse("type @A")?;
        stmt_eq!(
            t,
            St::TypeDef {
                name: spur!("A"),
                private: false
            }
        );

        let t = parse("private type @A")?;
        stmt_eq!(
            t,
            St::TypeDef {
                name: spur!("A"),
                private: true
            }
        );

        Ok(())
    }

    #[test]
    fn test_extract_import() -> ParseResult<()> {
        let t = parse("extract import a")?;
        stmt_eq!(
            t,
            St::ExtractImport(ImportType::Library(PathBuf::from("a"),))
        );

        let t = parse(r#"extract import "test.spwn""#)?;
        stmt_eq!(
            t,
            St::ExtractImport(ImportType::Module(
                PathBuf::from("test.spwn"),
                ModuleImport::Regular
            ))
        );

        Ok(())
    }

    #[test]
    fn test_impl() -> ParseResult<()> {
        let t = parse(
            r#"impl @A {
                a: 10,
                private b: false
            }"#,
        )?;
        stmt_eq!(
            t,
            St::Impl {
                base: Ex::Type(spur!("A")).into(),
                items: vec![
                    (span!(spur!("a")), Some(Ex::Int(10).into()), false),
                    (span!(spur!("b")), Some(Ex::Bool(false).into()), true),
                ],
            }
        );

        Ok(())
    }

    #[test]
    fn test_overload() -> ParseResult<()> {
        // let t = parse(
        //     r#"overload + {

        // }"#,
        // )?;
        // stmt_eq!(
        //     t,

        // );

        Ok(())
    }

    #[test]
    fn test_throw() -> ParseResult<()> {
        let t = parse(r#"throw "a""#)?;
        stmt_eq!(t, St::Throw(spur!("a")));

        Ok(())
    }
}
