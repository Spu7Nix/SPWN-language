use std::collections::BTreeMap;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Index;
use std::rc::Rc;

use ahash::AHashMap;
use colored::Colorize;
use delve::{EnumDisplay, VariantNames};
use derive_more::{Deref, DerefMut, Display, From};
use itertools::Itertools;
use lasso::Spur;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::compiler::{CustomTypeID, LocalTypeID};
use super::opcodes::{FuncID, Opcode, OptOpcode};
use crate::gd::ids::IDClass;
use crate::interpreting::value::ValueType;
use crate::new_id_wrapper;
use crate::parsing::ast::{Vis, VisTrait};
use crate::parsing::operators::operators::Operator;
use crate::sources::{CodeSpan, Spanned, SpwnSource};
use crate::util::{remove_quotes, Digest, ImmutStr, ImmutStr32, ImmutVec, SlabMap};

#[derive(Clone, Debug, From, EnumDisplay, Serialize, Deserialize)]
pub enum Constant {
    #[delve(display = |i| format!("{i}"))]
    Int(i64),
    #[delve(display = |i| format!("{i}"))]
    Float(f64),
    #[delve(display = |i| format!("{i}"))]
    Bool(bool),
    #[delve(display = |i| format!("{:?}", i))]
    String(ImmutStr32),

    #[delve(display = |t: &ValueType| {
        format!(
            "@{}",
            match t {
                ValueType::Custom(i) => format!("<{}:{}>", *i.local, i.source_hash),
                _ => <ValueType as Into<&str>>::into(*t).into(),
            }
        )
    })]
    Type(ValueType),

    #[delve(display = |class, id| format!("{id}{class}"))]
    Id(IDClass, u16),

    #[delve(display = |a: &Vec<Constant>| format!("[{}]", a.iter().map(|a| format!("{a}")).join(", ")))]
    Array(Vec<Constant>),

    #[delve(display = |d: &AHashMap<ImmutStr32, Constant>| format!("{{{}}}", d.iter().map(|(k, v)| format!("{k}: {v}")).join(", ")))]
    Dict(AHashMap<ImmutStr32, Constant>),

    #[delve(display = |m: &Option<Box<Constant>>| format!("{}?", m.as_ref().map(|v| format!("{v}")).unwrap_or("".into())))]
    Maybe(Option<Box<Constant>>),

    #[delve(display = "$")]
    Builtins,
    #[delve(display = "()")]
    Empty,

    #[delve(display = |_, i: &AHashMap<ImmutStr32, Box<Constant>>| format!("@<type>::{{{}}}", i.iter().map(|(k, v)| format!("{k}: {v}")).join(", ")))]
    Instance {
        typ: CustomTypeID,
        items: AHashMap<ImmutStr32, Box<Constant>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CallExpr<Arg, R, S> {
    // pub base: R,
    pub dest: Option<R>,
    pub positional: ImmutVec<(Arg, Mutability)>,
    pub named: ImmutVec<(S, Arg, Mutability)>,
}

impl<T: RegNum, U> CallExpr<Register<T>, Register<T>, U> {
    pub fn get_regs(&self) -> impl Iterator<Item = &Register<T>> {
        self.dest
            .iter()
            .chain(self.positional.iter().map(|(r, _)| r))
            .chain(self.named.iter().map(|(_, r, _)| r))
    }

    pub fn get_regs_mut(&mut self) -> impl Iterator<Item = &mut Register<T>> {
        self.dest
            .iter_mut()
            .chain(self.positional.iter_mut().map(|(r, _)| r))
            .chain(self.named.iter_mut().map(|(_, r, _)| r))
    }
}

impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Constant::Int(v) => v.hash(state),
            Constant::Float(v) => v.to_bits().hash(state),
            Constant::Bool(v) => v.hash(state),
            Constant::String(v) => v.hash(state),
            Constant::Type(v) => v.hash(state),
            Constant::Id(class, id) => {
                class.hash(state);
                id.hash(state);
            },
            Constant::Array(v) => {
                for i in v {
                    v.hash(state)
                }
            },
            Constant::Dict(map) => {
                let a: BTreeMap<_, _> = map.iter().collect();
                for (k, v) in a {
                    v.hash(state);
                    k.hash(state);
                }
            },
            Constant::Maybe(m) => m.hash(state),
            Constant::Builtins => "$".hash(state),
            Constant::Empty => "()".hash(state),
            Constant::Instance { typ, items } => {
                let a: BTreeMap<_, _> = items.iter().collect();
                for (k, v) in a {
                    v.hash(state);
                    k.hash(state);
                }
                typ.hash(state);
            },
        }
    }
}
impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l), Self::Int(r)) => l == r,
            (Self::Float(l), Self::Float(r)) => l.to_bits() == r.to_bits(),
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            _ => false,
        }
    }
}
impl Eq for Constant {}

pub trait RegNum: Copy + Display + From<u8> {}
impl RegNum for usize {}
impl RegNum for u8 {}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    DerefMut,
    Display,
    Serialize,
    Deserialize,
)]
#[display(fmt = "R{_0}")]
pub struct Register<T: RegNum>(pub T);

// trait RegTrait {
//     type InnerType;
// }

// impl<T> Register<T> {
//     pub type InnerType = T;
// }

pub type UnoptRegister = Register<usize>;
pub type OptRegister = Register<u8>;

// impl<T> Index<OptRegister> for T where T: Index<usize> {}

impl TryFrom<UnoptRegister> for OptRegister {
    type Error = ();

    fn try_from(value: UnoptRegister) -> Result<Self, Self::Error> {
        Ok(Register(value.0.try_into().map_err(|_| ())?))
    }
}

pub type Mutability = bool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function<T: RegNum> {
    pub regs_used: T,
    pub opcodes: ImmutVec<Spanned<Opcode<Register<T>>>>,

    pub span: CodeSpan,

    pub args: ImmutVec<Spanned<(Option<ImmutStr>, Mutability)>>,
    pub spread_arg: Option<u8>,
    pub captured_regs: Vec<(Register<T>, Register<T>)>,

    pub child_funcs: ImmutVec<FuncID>,
}
pub type OptFunction = Function<u8>;
pub type UnoptFunction = Function<usize>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bytecode<T: RegNum> {
    pub source_hash: Digest,
    pub version: Version,

    pub constants: Vec<Constant>,

    pub functions: Vec<Function<T>>,

    pub custom_types: AHashMap<CustomTypeID, Vis<Spanned<ImmutStr>>>,

    pub export_names: Vec<ImmutStr>,
    pub import_paths: Vec<SpwnSource>,

    pub debug_funcs: Vec<FuncID>,

    pub call_exprs: Vec<CallExpr<Register<T>, Register<T>, ImmutStr>>,
}
pub type OptBytecode = Bytecode<u8>;
pub type UnoptBytecode = Bytecode<usize>;

mod debug_bytecode {
    use std::borrow::Cow;

    use regex::{Captures, Regex};

    use super::*;
    use crate::util::clear_ansi;

    fn clear_len(v: &str) -> usize {
        clear_ansi(v).len()
    }

    #[derive(Debug)]
    struct TableRow {
        idx: String,
        opcode_name: String,
        opcode_str: String,
        span: String,
        snippet: String,
    }

    // holds the max length of each string
    #[derive(Default, Debug)]
    struct TableRowMax {
        idx: usize,
        opcode_name: usize,
        opcode_str: usize,
        span: usize,
        snippet: usize,
    }

    impl TableRowMax {
        fn update(&mut self, row: &TableRow) {
            self.idx = self.idx.max(clear_len(&row.idx));
            self.opcode_name = self.opcode_name.max(clear_len(&row.opcode_name));
            self.opcode_str = self.opcode_str.max(clear_len(&row.opcode_str));
            self.span = self.span.max(clear_len(&row.span));
            self.snippet = self.snippet.max(clear_len(&row.snippet));
        }
    }

    impl<T: RegNum> Bytecode<T> {
        pub fn debug_str(&self, src: &Rc<SpwnSource>, debug_funcs: Option<&[FuncID]>) {
            if matches!(&**src, SpwnSource::Core(..) | SpwnSource::Std(..)) {
                return;
            }

            println!(
                "{}\n",
                format!(
                    "================== {} ==================",
                    format!("{:?}", src).bright_yellow()
                )
                .bright_white()
            );
            println!(
                "- Constants: [{}]",
                self.constants
                    .iter()
                    .map(|c| format!("{c}").bright_green())
                    .join(", "),
            );
            println!(
                "- Export names: [{}]",
                self.export_names
                    .iter()
                    .map(|c| format!("{c}").bright_blue())
                    .join(", "),
            );
            println!(
                "- Import paths: [{}]",
                self.import_paths
                    .iter()
                    .map(|c| format!("{c:?}").bright_magenta())
                    .join(", "),
            );
            println!("- Custom types:");
            for (id, s) in &self.custom_types {
                let t = format!(
                    "    {}@{}",
                    if s.is_priv() { "priv " } else { "" }.bright_red(),
                    s.value().value
                )
                .bright_magenta();
                let id = format!("<{}:{}>", *id.local, id.source_hash,).dimmed();
                println!("{} {}", t, id);
            }

            let code = src.read().unwrap();

            let const_regex = Regex::new(r"ConstID\((\d+)\)").unwrap();
            let func_regex = Regex::new(r"FuncID\((\d+)\)").unwrap();
            let call_expr_regex = Regex::new(r"CallExprID\((\d+)\)").unwrap();
            let opcode_pos_regex = Regex::new(r"OpcodePos\((\d+)\)").unwrap();
            let import_regex = Regex::new(r"ImportID\((\d+)\)").unwrap();
            let reg_regex = Regex::new(r"(R\d+|R:mem)").unwrap();
            let mem_arrow_regex = Regex::new(r"~>").unwrap();

            for (func_id, func) in self.functions.iter().enumerate() {
                if let Some(v) = debug_funcs {
                    if !v.contains(&func_id.into()) {
                        continue;
                    }
                }
                let mut max = TableRowMax {
                    idx: 2,
                    opcode_name: 5,
                    opcode_str: 5,
                    span: 0,
                    snippet: 0,
                };
                let mut rows = vec![];
                for (
                    i,
                    Spanned {
                        value: opcode,
                        span,
                    },
                ) in func.opcodes.iter().enumerate()
                {
                    let row = TableRow {
                        idx: i.to_string().bright_blue().to_string(),
                        opcode_name: Into::<&str>::into(opcode).bright_white().to_string(),
                        opcode_str: {
                            let c: Cow<'_, str> = format!("{opcode}").into();
                            let c = mem_arrow_regex.replace_all(&c, "~>".yellow().to_string());
                            let c = reg_regex.replace_all(&c, "$1".bright_red().to_string());
                            let c =
                                opcode_pos_regex.replace_all(&c, "$1".bright_blue().to_string());
                            let c = func_regex
                                .replace_all(&c, "F$1".bright_magenta().bold().to_string());
                            let c = const_regex.replace_all(&c, |c: &Captures| {
                                let id = c.get(1).unwrap().as_str().parse::<usize>().unwrap();
                                format!("{}", self.constants[id]).bright_green().to_string()
                            });
                            let c = call_expr_regex.replace_all(&c, |c: &Captures| {
                                let id = c.get(1).unwrap().as_str().parse::<usize>().unwrap();

                                let call_expr = &self.call_exprs[id];

                                format!(
                                    "({}) -> {}",
                                    call_expr
                                        .positional
                                        .iter()
                                        .map(|(r, _)| r.to_string().bright_red().to_string())
                                        .chain(call_expr.named.iter().map(|(name, r, _)| format!(
                                            "{} = {}",
                                            name,
                                            r.to_string().bright_red()
                                        )))
                                        .join(", "),
                                    call_expr
                                        .dest
                                        .map(|r| r.to_string())
                                        .unwrap_or("?".into())
                                        .bright_red()
                                )
                            });
                            let c = import_regex.replace_all(&c, |c: &Captures| {
                                let id = c.get(1).unwrap().as_str().parse::<usize>().unwrap();
                                format!("{:?}", self.import_paths[id])
                                    .bright_magenta()
                                    .to_string()
                            });

                            c.bright_white().to_string()
                        },
                        span: format!("{}..{}", span.start, span.end)
                            .bright_white()
                            .dimmed()
                            .to_string(),
                        snippet: {
                            let mut s = code[span.start..span.end].to_string();
                            let chars = s.chars().collect_vec();
                            if chars.len() >= 15 {
                                s = format!(
                                    "{}{}{}",
                                    {
                                        let s =
                                            format!("{:?}", chars[..7].iter().collect::<String>());
                                        remove_quotes(&s).to_string()
                                    },
                                    "...".dimmed(),
                                    {
                                        let s = format!(
                                            "{:?}",
                                            chars[chars.len() - 7..].iter().collect::<String>()
                                        );
                                        remove_quotes(&s).to_string()
                                    }
                                )
                            } else {
                                s = format!("{:?}", s);
                                s = remove_quotes(&s).to_string();
                            }
                            s.bright_cyan().to_string()
                        },
                    };
                    max.update(&row);
                    rows.push(row);
                }

                let top = format!(
                    "╭─{}────{}──{}─┬─{}─{}─╮",
                    "─".repeat(max.idx),
                    "─".repeat(max.opcode_name),
                    "─".repeat(max.opcode_str),
                    "─".repeat(max.span),
                    "─".repeat(max.snippet),
                );
                let fn_title = format!(" Function {} ", func_id);

                let top = top.chars().take(5).collect::<String>()
                    + &fn_title
                    + &top.chars().skip(5 + fn_title.len()).collect::<String>();

                println!("{}", top.bright_yellow());

                for row in rows {
                    macro_rules! calc {
                        ($name:ident) => {
                            max.$name - clear_ansi(&row.$name).len() + row.$name.len()
                        };
                    }

                    let s = format!(
                        "│ {:>idx$}    {:>opcode_name$}  {:opcode_str$} │ {:>span$} {:snippet$} │",
                        row.idx,
                        row.opcode_name,
                        row.opcode_str,
                        row.span,
                        row.snippet,
                        idx = calc!(idx),
                        opcode_name = calc!(opcode_name),
                        opcode_str = calc!(opcode_str),
                        span = calc!(span),
                        snippet = calc!(snippet),
                    );
                    println!("{}", s.bright_yellow());
                }
                println!(
                    "{}",
                    format!(
                        "├─{}────{}──{}─┴─{}─{}─╯",
                        "─".repeat(max.idx),
                        "─".repeat(max.opcode_name),
                        "─".repeat(max.opcode_str),
                        "─".repeat(max.span),
                        "─".repeat(max.snippet),
                    )
                    .bright_yellow()
                );

                let extra = &[
                    ("regs used", func.regs_used.to_string()),
                    ("args", {
                        format!(
                            "{}, ({})",
                            func.args.len(),
                            func.args
                                .iter()
                                .enumerate()
                                .map(|(i, f)| {
                                    let s = f
                                        .value
                                        .0
                                        .as_ref()
                                        .map(|s| s.to_string())
                                        .unwrap_or("\\".bright_red().to_string());
                                    if func.spread_arg == Some(i as u8) {
                                        format!("...{}", s)
                                    } else {
                                        s
                                    }
                                })
                                .join(", ")
                        )
                    }),
                    (
                        "arg regs",
                        (0..func.args.len())
                            .map(|i| format!("-> {}", Register(i as u8).to_string().bright_red()))
                            .join(", "),
                    ),
                    (
                        "capture regs",
                        func.captured_regs
                            .iter()
                            .map(|(from, to)| {
                                format!(
                                    "{} -> {}",
                                    from.to_string().bright_red().to_string(),
                                    to.to_string().bright_red().to_string(),
                                )
                            })
                            .join(", "),
                    ),
                    (
                        "child funcs",
                        func.child_funcs
                            .iter()
                            .map(|v| format!("F{}", **v).bright_magenta().bold())
                            .join(", "),
                    ),
                ];

                for (k, v) in extra {
                    println!("{} {}", format!("│ {}:", k).bright_yellow(), v);
                }

                println!(
                    "{}",
                    "╰─────────────────────────────────────────────╼".bright_yellow()
                );

                println!();
                println!();
            }
        }
    }
}
