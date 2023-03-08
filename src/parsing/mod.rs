pub mod ast;
pub mod attributes;
pub mod error;
pub mod parser;
pub mod utils;

#[cfg(test)]
#[allow(clippy::from_over_into)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::RwLock;

    use lasso::{Rodeo, Spur};
    use lazy_static::lazy_static;

    use super::ast::{
        Ast, ExprNode, Expression as Ex, Statement as St, StmtNode, StringContent, StringType,
    };
    use super::parser::Parser;
    use super::utils::operators::BinOp;
    use crate::gd::ids::IDClass;
    use crate::parsing::attributes::FileAttribute;
    use crate::parsing::parser::ParseResult;
    use crate::sources::{CodeSpan, SpwnSource};
    use crate::RandomState;

    #[derive(Debug)]
    struct Interner(Rc<RefCell<Rodeo<Spur, RandomState>>>);
    impl std::ops::Deref for Interner {
        type Target = Rc<RefCell<Rodeo<Spur, RandomState>>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    unsafe impl Send for Interner {}
    unsafe impl Sync for Interner {}

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

    lazy_static! {
        static ref INTERNER: RwLock<Interner> = RwLock::new(Interner(Rc::new(RefCell::new(
            Rodeo::with_hasher(RandomState::new())
        ))));
    }

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

    macro_rules! stmt {
        () => {};
    }

    macro_rules! string {
        ($str:literal $(, $is_bytes:literal)?) => {
            Ex::String(StringType {
                s: StringContent::Normal(
                    (&*INTERNER.write().unwrap())
                        .borrow_mut()
                        .get_or_intern($str),
                ),
                bytes: { false $(; $is_bytes)?},
            })
        };
    }

    fn parse(code: &'static str) -> ParseResult<Ast> {
        let mut parser = Parser::new(
            code,
            SpwnSource::File("<test>".into()),
            Rc::clone(&*INTERNER.write().unwrap()),
        );

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

    // and more
}
