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
use crate::parsing::operators::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::parsing::parser::ParseResult;
use crate::sources::{CodeSpan, Spanned, SpwnSource};
use crate::util::interner::Interner;

type Pt = Pattern<Spur, PatternNode, ExprNode, Spur>;

trait IntoNode<T>
where
    Self: Sized,
{
    fn node(self) -> T {
        self.node_attrs(vec![])
    }

    fn node_attrs(self, attrs: Vec<Attribute>) -> T;
}

impl IntoNode<ExprNode> for Ex {
    fn node_attrs(self, attributes: Vec<Attribute>) -> ExprNode {
        ExprNode {
            expr: Box::new(self),
            attributes,
            span: CodeSpan::internal(),
        }
    }
}

impl IntoNode<StmtNode> for St {
    fn node_attrs(self, attributes: Vec<Attribute>) -> StmtNode {
        StmtNode {
            stmt: Box::new(self),
            attributes,
            span: CodeSpan::internal(),
        }
    }
}

impl IntoNode<PatternNode> for Pt {
    fn node_attrs(self, _: Vec<Attribute>) -> PatternNode {
        PatternNode {
            pat: Box::new(self),
            span: CodeSpan::internal(),
        }
    }
}

static mut INTERNER: Option<Interner> = None;

macro_rules! expr_eq {
    ($ast:ident,[$($exprs:expr),*] $(, attrs:$attrs:expr)?) => {
        let _exprs = &[$($exprs),*];
        for (i, stmt) in $ast.statements.iter().enumerate() {
            match &*stmt.stmt {
                St::Expr(_e) => {
                    assert_eq!(_exprs[i], *_e.expr);
                    $(assert_eq!($attrs, _e.attributes);)?
                },
                _ => unreachable!(),
            }
        }
    };

    ($ast:ident, $expr:expr $(, attrs:$attrs:expr)?) => {
        expr_eq!($ast, [$expr] $(, attrs:$attrs)?)
    };
}

macro_rules! stmt_eq {
    ($ast:ident,[$($stmts:expr),*] $(, attrs:$attrs:expr)?) => {
        let _stmts = &[$($stmts),*];
        for (i, stmt) in $ast.statements.iter().enumerate() {
            assert_eq!(&_stmts[i], &*stmt.stmt);
            $(assert_eq!($attrs, stmt.attributes);)?
        }
    };

    ($ast:ident, $expr:expr $(, attrs:$attrs:expr)?) => {
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
        let mut v = unsafe { INTERNER.as_ref().unwrap() };
        v.get_or_intern($str)
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
        Ex::Int(..) => (),
        Ex::Float(..) => (),
        Ex::String(..) => (),
        Ex::Bool(..) => (),
        Ex::Id(..) => (),
        Ex::Op(..) => (),
        Ex::Unary(..) => (),
        Ex::Var(..) => (),
        Ex::Type(..) => (),
        Ex::Array(..) => (),
        Ex::Dict(..) => (),
        Ex::Maybe(..) => (),
        Ex::Is(..) => (),
        Ex::Index { .. } => (),
        Ex::Member { .. } => (),
        Ex::TypeMember { .. } => (),
        Ex::Associated { .. } => (),
        Ex::Call { .. } => (),
        Ex::Macro { .. } => (),
        Ex::TriggerFunc { .. } => (),
        Ex::TriggerFuncCall(..) => (),
        Ex::Ternary { .. } => (),
        Ex::Typeof(..) => (),
        Ex::Builtins => (),
        Ex::Empty => (),
        Ex::Epsilon => (),
        Ex::Import(..) => (),
        Ex::Instance { .. } => (),
        Ex::Obj(..) => (),
        Ex::Dbg(..) => (),
        Ex::Match { .. } => (),
    }
    match St::Break {
        St::Expr(..) => (),
        St::AssignOp(..) => (),
        St::Assign(..) => (),
        St::If { .. } => (),
        St::While { .. } => (),
        St::For { .. } => (),
        St::TryCatch { .. } => (),
        St::Arrow(..) => (),
        St::Return(..) => (),
        St::Break => (),
        St::Continue => (),
        St::TypeDef { .. } => (),
        St::ExtractImport(..) => (),
        St::Impl { .. } => (),
        St::Overload { .. } => (),
        St::Throw(..) => (),
    }
};

fn parse(code: &'static str) -> ParseResult<Ast> {
    unsafe { INTERNER = Some(Interner::new()) };

    let mut parser = Parser::new(code, Rc::new(SpwnSource::File("<test>".into())), unsafe {
        INTERNER.as_ref().unwrap().clone()
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
fn test_attrs() -> ParseResult<()> {
    let t = parse(r#"#[doc(u"")] type @int"#)?;
    stmt_eq!(
        t,
        St::TypeDef(Vis::Public(spur!("int"))),
        attrs: vec![Attribute {
            style: AttrStyle::Outer,
            item: AttrItem {
                namespace: None,
                name: span!(spur!("doc")),
                args: AttrArgs::Eq(string!("", flags: { unindent: true }).node())
            },
            span: CodeSpan::internal()
        }]
    );
    let t = parse(r##"#[doc = r#""#] type @int"##)?;
    stmt_eq!(
        t,
        St::TypeDef(Vis::Public(spur!("int"))),
        attrs: vec![Attribute {
            style: AttrStyle::Outer,
            item: AttrItem {
                namespace: None,
                name: span!(spur!("doc")),
                args: AttrArgs::Eq(string!("", flags: { unindent: true }).node())
            },
            span: CodeSpan::internal()
        }]
    );

    let t = parse("#[debug_bytecode] () {}")?;
    expr_eq!(
        t,
        Ex::Macro { args: vec![], ret_pat: None, code: MacroCode::Normal(vec![]) },
        attrs: vec![Attribute {
            style: AttrStyle::Outer,
            item: AttrItem {
                namespace: None,
                name: span!(spur!("debug_bytecode")),
                args: AttrArgs::Empty
            },
            span: CodeSpan::internal()
        }]
    );

    let t = parse(
        r#"impl @int {
            #[builtin] a: () {}
        }"#,
    )?;
    stmt_eq!(
        t,
        St::Impl {
            name: span!(spur!("A")),
            items: vec![Vis::Public(DictItem {
                name: span!(spur!("a")),
                value: Some(
                    Ex::Macro {
                        args: vec![],
                        ret_pat: None,
                        code: MacroCode::Normal(vec![])
                    }
                    .node()
                ),
                attributes: vec![Attribute {
                    style: AttrStyle::Outer,
                    item: AttrItem {
                        namespace: None,
                        name: span!(spur!("builtin")),
                        args: AttrArgs::Empty
                    },
                    span: CodeSpan::internal()
                }],
            }),],
        }
    );

    let t = parse(
        r#"impl @int {
            #[alias = foo] a: () {}
        }"#,
    )?;
    stmt_eq!(
        t,
        St::Impl {
            name: span!(spur!("A")),
            items: vec![Vis::Public(DictItem {
                name: span!(spur!("a")),
                value: Some(
                    Ex::Macro {
                        args: vec![],
                        ret_pat: None,
                        code: MacroCode::Normal(vec![])
                    }
                    .node()
                ),
                attributes: vec![Attribute {
                    style: AttrStyle::Outer,
                    item: AttrItem {
                        namespace: None,
                        name: span!(spur!("alias")),
                        args: AttrArgs::Eq(Ex::Var(spur!("foo")).node())
                    },
                    span: CodeSpan::internal()
                }],
            }),],
        }
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
            s: StringType::FString(vec![
                Either::Left(spur!("")),
                Either::Right(Ex::Op(Ex::Int(1).node(), BinOp::Plus, Ex::Int(1).node()).node())
            ]),
            flags: StringFlags {
                ..Default::default()
            }
        })
    );

    let t = parse(r#"uBf"{1 + 1}""#)?;
    expr_eq!(
        t,
        Ex::String(StringContent {
            s: StringType::FString(vec![
                Either::Left(spur!("")),
                Either::Right(Ex::Op(Ex::Int(1).node(), BinOp::Plus, Ex::Int(1).node()).node())
            ]),
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
        Ex::Op(Ex::Int(5).node(), BinOp::Range, Ex::Int(100).node())
    );
    let t = parse("2..4..20")?;
    expr_eq!(
        t,
        Ex::Op(
            Ex::Op(Ex::Int(2).node(), BinOp::Range, Ex::Int(4).node()).node(),
            BinOp::Range,
            Ex::Int(20).node()
        )
    );

    let t = parse("1.2 in 2.2")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Float(1.2).node(), BinOp::In, Ex::Float(2.2).node())
    );

    let t = parse("0b01 | 0b10")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::BinOr, Ex::Int(2).node())
    );
    let t = parse("true || false")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Bool(true).node(), BinOp::Or, Ex::Bool(false).node())
    );

    let t = parse("0b10 & 0b01")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(2).node(), BinOp::BinAnd, Ex::Int(1).node())
    );
    let t = parse("false && false")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Bool(false).node(), BinOp::And, Ex::Bool(false).node())
    );

    let t = parse("0x9DE == true")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(2526).node(), BinOp::Eq, Ex::Bool(true).node())
    );
    let t = parse(r#""a" != 2"#)?;
    expr_eq!(
        t,
        Ex::Op(string!("a").node(), BinOp::Neq, Ex::Int(2).node())
    );
    let t = parse("5.3_2 > ?g")?;
    expr_eq!(
        t,
        Ex::Op(
            Ex::Float(5.32).node(),
            BinOp::Gt,
            Ex::Id(IDClass::Group, None).node()
        )
    );
    let t = parse("10 >= 5")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(10).node(), BinOp::Gte, Ex::Int(5).node(),)
    );
    let t = parse("10 < 5")?;
    expr_eq!(t, Ex::Op(Ex::Int(10).node(), BinOp::Lt, Ex::Int(5).node(),));
    let t = parse("10 <= 5")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(10).node(), BinOp::Lte, Ex::Int(5).node(),)
    );

    let t = parse("0b01 << 2")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::ShiftLeft, Ex::Int(2).node(),)
    );
    let t = parse("0b01 >> 2")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::ShiftRight, Ex::Int(2).node(),)
    );

    let t = parse("1 + 1")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::Plus, Ex::Int(1).node(),)
    );
    let t = parse("1 - 1")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::Minus, Ex::Int(1).node(),)
    );
    let t = parse("1 * 1")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(1).node(), BinOp::Mult, Ex::Int(1).node(),)
    );
    let t = parse("1 / 1")?;
    expr_eq!(t, Ex::Op(Ex::Int(1).node(), BinOp::Div, Ex::Int(1).node(),));
    let t = parse("1 % 1")?;
    expr_eq!(t, Ex::Op(Ex::Int(1).node(), BinOp::Mod, Ex::Int(1).node(),));
    let t = parse("1 ^ 1")?;
    expr_eq!(t, Ex::Op(Ex::Int(1).node(), BinOp::Pow, Ex::Int(1).node(),));

    let t = parse("2 as false")?;
    expr_eq!(
        t,
        Ex::Op(Ex::Int(2).node(), BinOp::As, Ex::Bool(false).node(),)
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
    expr_eq!(t, Ex::Unary(UnaryOp::ExclMark, Ex::Int(0).node()));

    let t = parse("-10.2")?;
    expr_eq!(t, Ex::Unary(UnaryOp::Minus, Ex::Float(10.2).node()));

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
    expr_eq!(t, Ex::Array(vec![Ex::Int(10).node(),]));

    let t = parse(r#"[10, a, true, "aa", 1.2, @a, [1, 2], a in b, !false]"#)?;
    expr_eq!(
        t,
        Ex::Array(vec![
            Ex::Int(10).node(),
            Ex::Var(spur!("a")).node(),
            Ex::Bool(true).node(),
            string!("aa").node(),
            Ex::Float(1.2).node(),
            Ex::Type(spur!("a")).node(),
            Ex::Array(vec![Ex::Int(1).node(), Ex::Int(2).node(),]).node(),
            Ex::Op(
                Ex::Var(spur!("a")).node(),
                BinOp::In,
                Ex::Var(spur!("b")).node()
            )
            .node(),
            Ex::Unary(UnaryOp::ExclMark, Ex::Bool(false).node()).node(),
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
                value: Some(Ex::Int(10).node()),
            }),
            Vis::Public(DictItem {
                name: span!(spur!("c")),
                attributes: vec![],
                value: Some(string!("a").node()),
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
    expr_eq!(t, Ex::Maybe(Some(Ex::Int(10).node())));

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

    let e: ExprNode = Ex::Int(1).node();

    let t = parse("1 is _")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Any.node()));

    let t = parse("1 is @test")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Type(spur!("test")).node()));

    let t = parse("1 is @int | @float")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::Either(
                Pt::Type(spur!("int")).node(),
                Pt::Type(spur!("float")).node(),
            )
            .node()
        )
    );

    let t = parse("1 is ==1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Eq(Ex::Int(1).node()).node()));
    let t = parse("1 is !=1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Neq(Ex::Int(1).node()).node()));
    let t = parse("1 is <1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Lt(Ex::Int(1).node()).node()));
    let t = parse("1 is <=1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Lte(Ex::Int(1).node()).node()));
    let t = parse("1 is >1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Gt(Ex::Int(1).node()).node()));
    let t = parse("1 is >=1")?;
    expr_eq!(t, Ex::Is(e.clone(), Pt::Gte(Ex::Int(1).node()).node()));

    let t = parse("1 is () -> ()")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::MacroPattern {
                args: vec![],
                ret: Pt::Empty.node(),
            }
            .node()
        )
    );
    let t = parse("1 is () -> _")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::MacroPattern {
                args: vec![],
                ret: Pt::Any.node(),
            }
            .node()
        )
    );
    let t = parse("1 is (@foo | @bar) -> @int")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::MacroPattern {
                args: vec![Pt::Either(
                    Pt::Type(spur!("foo")).node(),
                    Pt::Type(spur!("bar")).node()
                )
                .node()],
                ret: Pt::Type(spur!("int")).node(),
            }
            .node()
        )
    );

    let t = parse("1 is in [1, 2]")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::In(Ex::Array(vec![Ex::Int(1).node(), Ex::Int(2).node()]).node()).node()
        )
    );

    let t = parse("1 is @string[==2]")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::ArrayPattern(
                Pt::Type(spur!("string")).node(),
                Pt::Eq(Ex::Int(2).node()).node()
            )
            .node()
        )
    );

    let t = parse("1 is @int{:}")?;
    expr_eq!(
        t,
        Ex::Is(
            e.clone(),
            Pt::DictPattern(Pt::Type(spur!("int")).node()).node()
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
                .node(),
                Pt::Path {
                    var: spur!("y"),
                    path: vec![],
                    is_ref: true
                }
                .node()
            ])
            .node(),
            Ex::Array(vec![Ex::Int(1).node(), Ex::Int(2).node()]).node()
        )
    );

    let t = parse("{ x: @float } = ()")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::DictDestructure(AHashMap::from([(
                span!(spur!("x")),
                Pt::Type(spur!("float")).node()
            )]))
            .node(),
            Ex::Empty.node()
        )
    );
    let t = parse("@a::{ x } = ()")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::InstanceDestructure(
                spur!("a"),
                AHashMap::from([(
                    span!(spur!("x")),
                    Pt::Path {
                        var: spur!("x"),
                        path: vec![],
                        is_ref: false
                    }
                    .node()
                )])
            )
            .node(),
            Ex::Empty.node()
        )
    );

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
                .node()
            ))
            .node(),
            Ex::Maybe(None).node()
        )
    );
    let t = parse("? = ?")?;
    stmt_eq!(
        t,
        St::Assign(Pt::MaybeDestructure(None).node(), Ex::Maybe(None).node())
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
            .node(),
            Ex::Int(1).node()
        )
    );
    let t = parse("x[0].y::z = 1")?;
    stmt_eq!(
        t,
        St::Assign(
            Pt::Path {
                var: spur!("x"),
                path: vec![
                    AssignPath::Index(Ex::Int(0).node()),
                    AssignPath::Member(spur!("y")),
                    AssignPath::Associated(spur!("z"))
                ],
                is_ref: false
            }
            .node(),
            Ex::Int(1).node()
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
            .node(),
            Ex::Int(1).node()
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
            .node(),
            Ex::Int(1).node()
        )
    );

    let t = parse("1 is (@int | @float) if x > 2")?;
    expr_eq!(
        t,
        Ex::Is(
            e,
            Pt::IfGuard {
                pat: Pt::Either(
                    Pt::Type(spur!("int")).node(),
                    Pt::Type(spur!("float")).node()
                )
                .node(),
                cond: Ex::Op(Ex::Var(spur!("x")).node(), BinOp::Gt, Ex::Int(2).node()).node()
            }
            .node()
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
            base: Ex::Var(spur!("a")).node(),
            index: Ex::Int(200).node()
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
            base: Ex::Var(spur!("foo")).node(),
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
            base: Ex::Var(spur!("foo")).node(),
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
            base: Ex::Type(spur!("foo")).node(),
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
            base: Ex::Var(spur!("a")).node(),
            params: vec![],
            named_params: vec![],
        }
    );

    let t = parse("a(10)")?;
    expr_eq!(
        t,
        Ex::Call {
            base: Ex::Var(spur!("a")).node(),
            params: vec![Ex::Int(10).node()],
            named_params: vec![],
        }
    );

    let t = parse("a(foo = 20)")?;
    expr_eq!(
        t,
        Ex::Call {
            base: Ex::Var(spur!("a")).node(),
            params: vec![],
            named_params: vec![(span!(spur!("foo")), Ex::Int(20).node())],
        }
    );

    let t = parse("a(10, 20, foo = 30)")?;
    expr_eq!(
        t,
        Ex::Call {
            base: Ex::Var(spur!("a")).node(),
            params: vec![Ex::Int(10).node(), Ex::Int(20).node()],
            named_params: vec![(span!(spur!("foo")), Ex::Int(30).node())],
        }
    );

    Ok(())
}

#[test]
fn test_macro() -> ParseResult<()> {
    let t = parse("(a, &b: @int, c = 20, d: @bool = true) {}")?;
    expr_eq!(
        t,
        Ex::Macro {
            args: vec![
                MacroArg::Single {
                    pattern: Pt::Path {
                        var: spur!("a"),
                        path: vec![],
                        is_ref: false
                    }
                    .node(),
                    default: None,
                },
                MacroArg::Single {
                    pattern: Pt::Both(
                        Pt::Path {
                            var: spur!("b"),
                            path: vec![],
                            is_ref: true
                        }
                        .node(),
                        Pt::Type(spur!("int")).node()
                    )
                    .node(),
                    default: None,
                },
                MacroArg::Single {
                    pattern: Pt::Path {
                        var: spur!("c"),
                        path: vec![],
                        is_ref: false
                    }
                    .node(),
                    default: Some(Ex::Int(20).node()),
                },
                MacroArg::Single {
                    pattern: Pt::Both(
                        Pt::Path {
                            var: spur!("d"),
                            path: vec![],
                            is_ref: false
                        }
                        .node(),
                        Pt::Type(spur!("bool")).node()
                    )
                    .node(),
                    default: Some(Ex::Bool(true).node()),
                }
            ],
            ret_pat: None,
            code: MacroCode::Normal(vec![])
        }
    );

    let t = parse("(...&b: @int) {}")?;
    expr_eq!(
        t,
        Ex::Macro {
            args: vec![MacroArg::Spread {
                pattern: Pt::Both(
                    Pt::Path {
                        var: spur!("b"),
                        path: vec![],
                        is_ref: true
                    }
                    .node(),
                    Pt::Type(spur!("int")).node()
                )
                .node(),
            }],
            ret_pat: None,
            code: MacroCode::Normal(vec![])
        }
    );

    let t = parse("() -> @string {}")?;
    expr_eq!(
        t,
        Ex::Macro {
            args: vec![],
            ret_pat: Some(Pt::Type(spur!("string")).node()),
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
            code: vec![St::Assign(
                Pt::Path {
                    var: spur!("a"),
                    path: vec![],
                    is_ref: false
                }
                .node(),
                Ex::Int(10).node()
            )
            .node()]
        }
    );

    let t = parse("a!")?;
    expr_eq!(t, Ex::TriggerFuncCall(Ex::Var(spur!("a")).node()));

    Ok(())
}

#[test]
fn test_ternary() -> ParseResult<()> {
    let t = parse("1 if true else 2")?;
    expr_eq!(
        t,
        Ex::Ternary {
            cond: Ex::Bool(true).node(),
            if_true: Ex::Int(1).node(),
            if_false: Ex::Int(2).node(),
        }
    );

    Ok(())
}

#[test]
fn test_typeof() -> ParseResult<()> {
    let t = parse("2.type")?;
    expr_eq!(t, Ex::Typeof(Ex::Int(2).node()));

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
            base: Ex::Type(spur!("foo")).node(),
            items: vec![
                Vis::Public(DictItem {
                    name: span!(spur!("a")),
                    value: Some(Ex::Int(10).node()),
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
                    Ex::Int(10).node()
                ),
                (
                    span!(ObjKeyType::Name(ObjectKey::Groups)),
                    Ex::Array(vec![Ex::Id(IDClass::Group, Some(20)).node()]).node()
                ),
                (span!(ObjKeyType::Num(5)), Ex::Bool(false).node())
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
                Ex::Int(10).node()
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
            .node(),
            Ex::Int(10).node()
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
            .node(),
            AssignOp::PlusEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::MinusEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::MultEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::DivEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::PowEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::ModEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::BinAndEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::BinOrEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::ShiftLeftEq,
            Ex::Int(1).node()
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
            .node(),
            AssignOp::ShiftRightEq,
            Ex::Int(1).node()
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
                (Ex::Bool(false).node(), vec![]),
                (Ex::Bool(true).node(), vec![])
            ],
            else_branch: Some(vec![])
        }
    );

    let t = parse("if false {}")?;
    stmt_eq!(
        t,
        St::If {
            branches: vec![(Ex::Bool(false).node(), vec![]),],
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
            cond: Ex::Bool(true).node(),
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
                .node(),
                Pt::Path {
                    var: spur!("y"),
                    path: vec![],
                    is_ref: false
                }
                .node()
            ])
            .node(),
            iterator: Ex::Array(vec![Ex::Int(1).node(), Ex::Int(2).node()]).node(),
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
                .node()
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
                .node(),
                Ex::Int(10).node()
            )
            .node()
        ))
    );

    Ok(())
}

#[test]
fn test_control_flow() -> ParseResult<()> {
    let t = parse("return 20")?;
    stmt_eq!(t, St::Return(Some(Ex::Int(20).node())));

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
                    value: Some(Ex::Int(10).node()),
                    attributes: vec![],
                }),
                Vis::Private(DictItem {
                    name: span!(spur!("b")),
                    value: Some(Ex::Bool(false).node()),
                    attributes: vec![],
                }),
            ],
        }
    );

    Ok(())
}

#[test]
fn test_overload() -> ParseResult<()> {
    let t = parse(
        r#"
        overload + {
            (left: @int, right: @string) {},
            private (left: @foo, right: @foo) {}
        }
    "#,
    )?;
    stmt_eq!(
        t,
        St::Overload {
            op: Operator::Bin(BinOp::Plus),
            macros: vec![
                Vis::Public(
                    Ex::Macro {
                        args: vec![
                            MacroArg::Single {
                                pattern: Pt::Both(
                                    Pt::Path {
                                        var: spur!("left"),
                                        path: vec![],
                                        is_ref: false
                                    }
                                    .node(),
                                    Pt::Type(spur!("int")).node()
                                )
                                .node(),
                                default: None
                            },
                            MacroArg::Single {
                                pattern: Pt::Both(
                                    Pt::Path {
                                        var: spur!("right"),
                                        path: vec![],
                                        is_ref: false
                                    }
                                    .node(),
                                    Pt::Type(spur!("string")).node()
                                )
                                .node(),
                                default: None
                            },
                        ],
                        ret_pat: None,
                        code: MacroCode::Normal(vec![])
                    }
                    .node()
                ),
                Vis::Private(
                    Ex::Macro {
                        args: vec![
                            MacroArg::Single {
                                pattern: Pt::Both(
                                    Pt::Path {
                                        var: spur!("left"),
                                        path: vec![],
                                        is_ref: false
                                    }
                                    .node(),
                                    Pt::Type(spur!("foo")).node()
                                )
                                .node(),
                                default: None
                            },
                            MacroArg::Single {
                                pattern: Pt::Both(
                                    Pt::Path {
                                        var: spur!("right"),
                                        path: vec![],
                                        is_ref: false
                                    }
                                    .node(),
                                    Pt::Type(spur!("foo")).node()
                                )
                                .node(),
                                default: None
                            },
                        ],
                        ret_pat: None,
                        code: MacroCode::Normal(vec![])
                    }
                    .node()
                )
            ]
        }
    );

    let t = parse(
        r#"
        overload unary - {
            (value: @bool) {}
        }
    "#,
    )?;
    stmt_eq!(
        t,
        St::Overload {
            op: Operator::Unary(UnaryOp::Minus),
            macros: vec![Vis::Public(
                Ex::Macro {
                    args: vec![MacroArg::Single {
                        pattern: Pt::Both(
                            Pt::Path {
                                var: spur!("value"),
                                path: vec![],
                                is_ref: false
                            }
                            .node(),
                            Pt::Type(spur!("bool")).node()
                        )
                        .node(),
                        default: None
                    },],
                    ret_pat: None,
                    code: MacroCode::Normal(vec![])
                }
                .node()
            )]
        }
    );

    Ok(())
}

#[test]
fn test_throw() -> ParseResult<()> {
    let t = parse(r#"throw [1, 2]"#)?;
    stmt_eq!(
        t,
        St::Throw(Ex::Array(vec![Ex::Int(1).node(), Ex::Int(2).node()]).node())
    );

    Ok(())
}
