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
use super::opcodes::{Opcode, OptOpcode};
use crate::interpreting::value::ValueType;
use crate::new_id_wrapper;
use crate::parsing::ast::{Vis, VisTrait};
use crate::sources::{CodeSpan, Spanned, SpwnSource};
use crate::util::{remove_quotes, Digest, ImmutStr, ImmutVec, SlabMap};

#[derive(Clone, Debug, From, EnumDisplay, Serialize, Deserialize)]
pub enum Constant {
    #[delve(display = |i: &i64| format!("{i}"))]
    Int(i64),
    #[delve(display = |i: &f64| format!("{i}"))]
    Float(f64),
    #[delve(display = |i: &bool| format!("{i}"))]
    Bool(bool),
    #[delve(display = |i: &ImmutVec<char>| format!("{:?}", String::from_iter(i.iter())))]
    String(ImmutVec<char>),
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
}

// pub enum DestructurePattern<R: Copy + std::fmt::Display> {
//     Read(R),
//     Write(R),
//     Array(ImmutVec<Self>),
//     Dict(AHashMap<ImmutStr, Self>),
//     Instance {
//         typ: CustomTypeID,
//         items: AHashMap<ImmutStr, Self>,
//     },
// }

impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Constant::Int(v) => v.hash(state),
            Constant::Float(v) => v.to_bits().hash(state),
            Constant::Bool(v) => v.hash(state),
            Constant::String(v) => v.hash(state),
            Constant::Type(v) => v.hash(state),
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
pub struct Register<T: Copy + Display>(pub T);

pub type UnoptRegister = Register<usize>;
pub type OptRegister = Register<u8>;

// impl<T> Index<OptRegister> for T where T: Index<usize> {}

impl TryFrom<UnoptRegister> for OptRegister {
    type Error = ();

    fn try_from(value: UnoptRegister) -> Result<Self, Self::Error> {
        Ok(Register(value.0.try_into().map_err(|_| ())?))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub regs_used: u8,
    pub opcodes: ImmutVec<Spanned<OptOpcode>>,
    pub span: CodeSpan,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bytecode {
    pub source_hash: Digest,
    pub version: Version,

    pub constants: ImmutVec<Constant>,

    pub functions: ImmutVec<Function>,

    pub custom_types: AHashMap<CustomTypeID, Vis<Spanned<ImmutStr>>>,

    pub export_names: ImmutVec<ImmutStr>,
    pub import_paths: ImmutVec<SpwnSource>,
}

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

    impl Bytecode {
        pub fn debug_str(&self, src: &Rc<SpwnSource>) {
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
            let opcode_pos_regex = Regex::new(r"OpcodePos\((\d+)\)").unwrap();
            let import_regex = Regex::new(r"ImportID\((\d+)\)").unwrap();
            let reg_regex = Regex::new(r"(R\d+|R:mem)").unwrap();
            let mem_arrow_regex = Regex::new(r"~>").unwrap();

            for (func_id, func) in self.functions.iter().enumerate() {
                let mut max = TableRowMax::default();
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
                            let c =
                                mem_arrow_regex.replace_all(&c, "~>".bright_green().to_string());
                            let c = reg_regex.replace_all(&c, "$1".bright_red().to_string());
                            let c =
                                opcode_pos_regex.replace_all(&c, "$1".bright_blue().to_string());
                            let c = const_regex.replace_all(&c, |c: &Captures| {
                                let id = c.get(1).unwrap().as_str().parse::<usize>().unwrap();
                                format!("{}", self.constants[id]).bright_green().to_string()
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
                            if s.len() >= 15 {
                                s = format!(
                                    "{}{}{}",
                                    {
                                        let s = format!("{:?}", &s[..7]);
                                        remove_quotes(&s).to_string()
                                    },
                                    "...".dimmed(),
                                    {
                                        let s = format!("{:?}", &s[s.len() - 7..]);
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

                let extra = &[("regs used", func.regs_used.to_string())];

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
