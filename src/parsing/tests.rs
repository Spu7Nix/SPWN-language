use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use itertools::Either;
use lasso::{Rodeo, Spur};

use super::ast::{
    Ast, ExprNode, Expression as Ex, Statement as St, StmtNode, StringContent, StringType,
};
use super::parser::Parser;
use crate::gd::ids::IDClass;
use crate::gd::object_keys::ObjectKey;
use crate::parsing::ast::{
    AssignPath, AttrArgs, AttrItem, AttrStyle, Attribute, DictItem, Import, ImportType, MacroArg,
    MacroCode, ObjKeyType, ObjectType, Pattern, PatternNode, StringFlags, Vis,
};
use crate::parsing::operators::operators::{AssignOp, BinOp, UnaryOp};
use crate::parsing::parser::ParseResult;
use crate::sources::{CodeSpan, Spanned, SpwnSource};
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

type Pt = Pattern<Spur, PatternNode, ExprNode, Spur>;
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
    ($str:literal $(, flags: {
        $($flag:ident : $b:literal),*
    })?) => {{
        Ex::String(StringContent {
            s: StringType::Normal(spur!($str)),
            flags: StringFlags {
                $($($flag : $b,)*)?
                ..Default::default()
            }
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
        Ex::Dbg(_, _) => todo!(),
        Ex::Match { value, branches } => todo!(),
    }
    match St::Break {
        St::Expr(_) => (),
        St::AssignOp(_, _, _) => (),
        St::Assign(_, _) => todo!(),
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
        St::Throw(_) => (),
    }
};

fn parse(code: &'static str) -> ParseResult<Ast> {
    unsafe {
        INTERNER = Some(Rc::new(RefCell::new(
            Rodeo::with_hasher(RandomState::new()),
        )))
    };

    let mut parser = Parser::new(code, Rc::new(SpwnSource::File("<test>".into())), unsafe {
        Rc::clone(INTERNER.as_ref().unwrap())
    });

    parser.parse()
}

#[test]
fn test_file_attrs() -> ParseResult<()> {
    let t = parse("#![no_std]")?;

    assert_eq!(
        &t.file_attributes,
        &[Attribute {
            style: AttrStyle::Inner,
            item: AttrItem {
                namespace: None,
                name: span!(spur!("no_std")),
                args: AttrArgs::Empty
            },
            span: CodeSpan::internal()
        }]
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

    // cant check the strings cause the string flag is only applied when calling `get_compile_time`
    // and not at parsing so the spurs will not match
    parse(r#"B"skrunkly""#)?;
    parse(r#"u"\n\taa\n\t\tbb""#)?;
    parse(r#"b"rawww""#)?;
    parse(r##"Br#" "something" "#"##)?;
    parse(r#"uB"\n\taa\n\t\tbb""#)?;

    let t = parse(r#"f"{1 + 1}""#)?;
    expr_eq!(
        t,
        Ex::String(StringContent {
            s: StringType::FString(vec![Either::Right(
                Ex::Op(Ex::Int(1).into(), BinOp::Plus, Ex::Int(1).into()).into()
            )]),
            flags: StringFlags {
                ..Default::default()
            }
        })
    );

    let t = parse(r#"uBf"{1 + 1}""#)?;
    expr_eq!(
        t,
        Ex::String(StringContent {
            s: StringType::FString(vec![Either::Right(
                Ex::Op(Ex::Int(1).into(), BinOp::Plus, Ex::Int(1).into()).into()
            )]),
            flags: StringFlags {
                base64: true,
                unindent: true,
                ..Default::default()
            }
        })
    );

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
    expr_eq!(t, Ex::Id(IDClass::Channel, None));

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
            Vis::Public(DictItem {
                name: span!(spur!("a")),
                attributes: vec![],
                value: None,
            }),
            Vis::Public(DictItem {
                name: span!(spur!("b")),
                attributes: vec![],
                value: Some(Ex::Int(10).into()),
            }),
            Vis::Public(DictItem {
                name: span!(spur!("c")),
                attributes: vec![],
                value: Some(string!("a").into()),
            })
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
        Pattern::Type(..) => (),
        Pattern::Either(..) => (),
        Pattern::Both(..) => (),
        Pattern::Eq(..) => (),
        Pattern::Neq(..) => (),
        Pattern::Lt(..) => (),
        Pattern::Lte(..) => (),
        Pattern::Gt(..) => (),
        Pattern::Gte(..) => (),

        Pattern::MacroPattern { .. } => (),
        Pattern::In(..) => (),
        Pattern::ArrayPattern(..) => (),
        Pattern::DictPattern(..) => (),
        Pattern::ArrayDestructure(..) => (),
        Pattern::DictDestructure(..) => (),
        Pattern::MaybeDestructure(..) => (),
        Pattern::InstanceDestructure(..) => (),
        Pattern::Path { .. } => (),
        Pattern::Mut { .. } => (),
        Pattern::IfGuard { .. } => (),
        Pattern::Empty => (),
    };

    let e: ExprNode = Ex::Int(1).into();

    let t = parse("1 is _")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Any.into()));

    let t = parse("1 is @test")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Type(spur!("test")).into()));

    let t = parse("1 is @int | @float")?;
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
    expr_eq!(t, Ex::Is(e.clone(), Pt::Gte(Ex::Int(1).into()).into()));

    // let t = parse("() -> ()")?;
    // expr_eq!(t, Pt::MacroPattern {
    //     args: vec![],
    //     ret: Pt::
    // }.into());
    let t = parse("1 is () -> _")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::MacroPattern {
                args: vec![],
                ret: Pt::Any.into(),
            }
            .into()
        )
    );
    let t = parse("1 is (@foo | @bar) -> @int")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::MacroPattern {
                args: vec![Pt::Either(
                    Pt::Type(spur!("foo")).into(),
                    Pt::Type(spur!("bar")).into()
                )
                .into()],
                ret: Pt::Type(spur!("int")).into(),
            }
            .into()
        )
    );

    let t = parse("1 is in [1, 2]")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::In(Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into()]).into()).into()
        )
    );

    let t = parse("1 is @string[==2]")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::ArrayPattern(
                Pt::Type(spur!("string")).into(),
                Pt::Eq(Ex::Int(2).into()).into()
            )
            .into()
        )
    );

    let t = parse("1 is @int{:}")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::DictPattern(Pt::Type(spur!("int")).into()).into()
        )
    );

    let t = parse("[x, &y] = [1, 2]")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::ArrayDestructure(vec![
                Pt::Path {
                    var: spur!("x"),
                    path: vec![],
                    is_ref: false
                }
                .into(),
                Pt::Path {
                    var: spur!("y"),
                    path: vec![],
                    is_ref: true
                }
                .into()
            ])
            .into(),
            Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into()]).into()
        )
    );

    let t = parse("{ x: @float } = ()")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::DictDestructure(AHashMap::from([(
                span!(spur!("x")),
                Pt::Type(spur!("float")).into()
            )]))
            .into(),
            Ex::Empty.into()
        )
    );
    // let t = parse("@a::{ x } = ()")?;
    // stmt_eq!(
    //     t,
    //     St::Assign(
    //         Pt::InstanceDestructure(spur!("a"), AHashMap::from([(spur!("x"), None)])).into(),
    //         Ex::Empty.into()
    //     )
    // );

    let t = parse("x? = ?")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::MaybeDestructure(Some(
                Pt::Path {
                    var: spur!("x"),
                    path: vec![],
                    is_ref: false
                }
                .into()
            ))
            .into(),
            Ex::Maybe(None).into()
        )
    );
    let t = parse("? = ?")?;
    stmt_eq!(
        t,
        St::Assign(Pt::MaybeDestructure(None).into(), Ex::Maybe(None).into())
    );

    let t = parse("&x = 1")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Path {
                var: spur!("x"),
                path: vec![],
                is_ref: true
            }
            .into(),
            Ex::Int(1).into()
        )
    );
    let t = parse("x[0].y::z = 1")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Path {
                var: spur!("x"),
                path: vec![
                    AssignPath::Index(Ex::Int(0).into()),
                    AssignPath::Member(spur!("y")),
                    AssignPath::Associated(spur!("z"))
                ],
                is_ref: false
            }
            .into(),
            Ex::Int(1).into()
        )
    );

    let t = parse("&mut x = 1")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Mut {
                name: spur!("x"),
                is_ref: true
            }
            .into(),
            Ex::Int(1).into()
        )
    );
    let t = parse("mut x = 1")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Mut {
                name: spur!("x"),
                is_ref: false
            }
            .into(),
            Ex::Int(1).into()
        )
    );

    let t = parse("1 is (@int | @float) if x > 2")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::IfGuard {
                pat: Pt::Either(
                    Pt::Type(spur!("int")).into(),
                    Pt::Type(spur!("float")).into()
                )
                .into(),
                cond: Ex::Op(Ex::Var(spur!("x")).into(), BinOp::Gt, Ex::Int(2).into()).into()
            }
            .into()
        )
    );

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
    //c = 20, d: @bool = true
    // let t = parse("(a, &b: @int, c = 20, d: @bool = true) {}")?;
    // expr_eq!(
    //     t,
    //     Ex::Macro {
    //         args: vec![
    //             MacroArg::Single {
    //                 pattern: Pt::Path {
    //                     var: spur!("a"),
    //                     path: vec![],
    //                     is_ref: false
    //                 }
    //                 .into(),
    //                 default: None,
    //             },
    //             MacroArg::Single {
    //                 pattern: Pt::Both(
    //                     Pt::Path {
    //                         var: spur!("b"),
    //                         path: vec![],
    //                         is_ref: true
    //                     }
    //                     .into(),
    //                     Pt::Type(spur!("int")).into()
    //                 )
    //                 .into(),
    //                 default: None,
    //             },
    //             MacroArg::Single {
    //                 pattern: Pt::Path {
    //                     var: spur!("c"),
    //                     path: vec![],
    //                     is_ref: false
    //                 }
    //                 .into(),
    //                 default: Some(Ex::Int(20).into()),
    //             },
    //             // MacroArg::Single {
    //             //     pattern: Pt::Both(
    //             //         Pt::Path {
    //             //             var: spur!("d"),
    //             //             path: vec![],
    //             //             is_ref: false
    //             //         }
    //             //         .into(),
    //             //         Pt::Type(spur!("bool")).into()
    //             //     )
    //             //     .into(),
    //             //     default: Some(Ex::Bool(true).into()),
    //             // }
    //         ],
    //         ret_pat: None,
    //         code: MacroCode::Normal(vec![])
    //     }
    // );

    // let t = parse("(...&b: @int) {}")?;
    // expr_eq!(
    //     t,
    //     Ex::Macro {
    //         args: vec![
    //             MacroArg::Spread {
    //                 pattern: Pt::Both(
    //                     Pt::Path {
    //                         var: spur!("b"),
    //                         path: vec![],
    //                         is_ref: true
    //                     }
    //                     .into(),
    //                     Pt::Type(spur!("int")).into()
    //                 )
    //                 .into(),
    //             }
    //         ],
    //         ret_pat: None,
    //         code: MacroCode::Normal(vec![])
    //     }
    // );

    // let t = parse("() -> @string {}")?;
    // expr_eq!(
    //     t,
    //     Ex::Macro {
    //         args: vec![],
    //         ret_pat: Some(Pt::Type(spur!("string")).into()),
    //         code: MacroCode::Normal(vec![])
    //     }
    // );

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
            code: vec![St::Assign(
                Pt::Path {
                    var: spur!("a"),
                    path: vec![],
                    is_ref: false
                }
                .into(),
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
    expr_eq!(
        t,
        Ex::Import(Import {
            path: "foobar".into(),
            typ: ImportType::Library,
        })
    );

    let t = parse(r#"import "foobar.spwn""#)?;
    expr_eq!(
        t,
        Ex::Import(Import {
            path: "foobar.spwn".into(),
            typ: ImportType::File
        })
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
                Vis::Public(DictItem {
                    name: span!(spur!("a")),
                    value: Some(Ex::Int(10).into()),
                    attributes: vec![],
                }),
                Vis::Public(DictItem {
                    name: span!(spur!("b")),
                    value: None,
                    attributes: vec![],
                }),
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
fn test_assign() -> ParseResult<()> {
    let t = parse("&a = 10")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: true
            }
            .into(),
            Ex::Int(10).into()
        )
    );

    Ok(())
}

#[test]
fn test_assign_op() -> ParseResult<()> {
    // useless match so this errors if we ever add/remove ops
    // if this errors please add / remove the respective tests
    match AssignOp::PlusEq {
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

    let t = parse("a += 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::PlusEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a -= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::MinusEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a *= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::MultEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a /= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::DivEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a ^= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::PowEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a %= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::ModEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a &= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::BinAndEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a |= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::BinOrEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a <<= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
            AssignOp::ShiftLeftEq,
            Ex::Int(1).into()
        )
    );

    let t = parse("a >>= 1")?;
    stmt_eq!(
        t,
        St::AssignOp(
            Pt::Path {
                var: spur!("a"),
                path: vec![],
                is_ref: false
            }
            .into(),
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
    let t = parse("for [x, y] in [1, 2] {}")?;
    stmt_eq!(
        t,
        St::For {
            iter: Pt::ArrayDestructure(vec![
                Pt::Path {
                    var: spur!("x"),
                    path: vec![],
                    is_ref: false
                }
                .into(),
                Pt::Path {
                    var: spur!("y"),
                    path: vec![],
                    is_ref: false
                }
                .into()
            ])
            .into(),
            iterator: Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into()]).into(),
            code: vec![]
        }
    );

    Ok(())
}

#[test]
fn test_try_catch() -> ParseResult<()> {
    let t = parse("try {} catch {}")?;
    stmt_eq!(
        t,
        St::TryCatch {
            try_code: vec![],
            catch_pat: None,
            catch_code: vec![],
        }
    );

    let t = parse("try {} catch e {}")?;
    stmt_eq!(
        t,
        St::TryCatch {
            try_code: vec![],
            catch_pat: Some(
                Pt::Path {
                    var: spur!("e"),
                    path: vec![],
                    is_ref: false
                }
                .into()
            ),
            catch_code: vec![],
        }
    );

    Ok(())
}

#[test]
fn test_arrow() -> ParseResult<()> {
    let t = parse("-> a = 10")?;
    stmt_eq!(
        t,
        St::Arrow(Box::new(
            St::Assign(
                Pt::Path {
                    var: spur!("a"),
                    path: vec![],
                    is_ref: false
                }
                .into(),
                Ex::Int(10).into()
            )
            .into()
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
    stmt_eq!(t, St::TypeDef(Vis::Public(spur!("A"))));

    let t = parse("private type @A")?;
    stmt_eq!(t, St::TypeDef(Vis::Private(spur!("A"))));

    Ok(())
}

#[test]
fn test_extract_import() -> ParseResult<()> {
    let t = parse("extract import lib")?;
    stmt_eq!(
        t,
        St::ExtractImport(Import {
            path: "lib".into(),
            typ: ImportType::Library
        })
    );

    let t = parse(r#"extract import "test.spwn""#)?;
    stmt_eq!(
        t,
        St::ExtractImport(Import {
            path: "test.spwn".into(),
            typ: ImportType::File
        })
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
            name: span!(spur!("A")),
            items: vec![
                Vis::Public(DictItem {
                    name: span!(spur!("a")),
                    value: Some(Ex::Int(10).into()),
                    attributes: vec![],
                }),
                Vis::Private(DictItem {
                    name: span!(spur!("b")),
                    value: Some(Ex::Bool(false).into()),
                    attributes: vec![],
                }),
            ],
        }
    );

    Ok(())
}

#[test]
fn test_overload() -> ParseResult<()> {
    // todo!("overloading tests");
    Ok(())
}

#[test]
fn test_throw() -> ParseResult<()> {
    let t = parse(r#"throw [1, 2]"#)?;
    stmt_eq!(
        t,
        St::Throw(Ex::Array(vec![Ex::Int(1).into(), Ex::Int(2).into()]).into())
    );

    Ok(())
}
