use std::fmt::Display;
use std::io::Write;

// use crate::compiler::error::CompilerError;
use crate::interpreter::error::RuntimeError;
use crate::parser::error::SyntaxError;
use ariadne::Color;

pub const ERROR_S: f64 = 0.4;
pub const ERROR_V: f64 = 1.0;

#[derive(Debug)]
pub struct RainbowColorGenerator {
    h: f64,
    s: f64,
    v: f64,
    shift: f64,
}

#[derive(Debug)]
pub enum Error {
    Syntax(SyntaxError),
    Runtime(RuntimeError),
    // Compiler(CompilerError),
}

impl Display for self::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for self::Error {}

pub type Result<T> = std::result::Result<T, self::Error>;

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, v: f64, shift: f64) -> Self {
        Self { h, s, v, shift }
    }
    pub fn next(&mut self) -> Color {
        // thanks wikipedia

        self.h = self.h.rem_euclid(360.0);

        let c = self.v * self.s;
        let h1 = self.h / 60.0;

        let x = c * (1.0 - (h1.rem_euclid(2.0) - 1.0).abs());

        let (r, g, b) = if (0.0..1.0).contains(&h1) {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h1) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h1) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h1) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h1) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let m = self.v - c;

        self.h += self.shift;

        ariadne::Color::RGB(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
}

#[macro_export]
macro_rules! error_maker {
    (
        $(
            $( Globals: $globals:ident; )?
            pub enum $err_type:ident {
                $(
                    #[
                        Message = $msg:expr, Area = $area:expr, Note = $note:expr,
                        Labels = [
                            $(
                                $l_area:expr => $fmt:literal $(: $( $(@($c_e:expr))? $($e:expr)? ),* )?;
                            )+
                        ]
                    ]
                    $variant:ident {
                        $(
                            $field:ident: $typ:ty,
                        )+
                    },
                )*
            }
        )*

    ) => {
        use std::path::PathBuf;
        use $crate::error::*;
        use ariadne::{Report, ReportKind, Label, Source, Fmt};
        // #[allow(unused_imports)]
        // use $crate::Globals;

        $(
            #[derive(Debug)]
            pub enum $err_type {
                $(
                    $variant {
                        $(
                            $field: $typ,
                        )+
                    },
                )*
            }

            impl $err_type {
                pub fn raise(self, code: String, source: Option<PathBuf> /*$(, $globals: &Globals)?*/) -> String {
                    let mut label_colors = RainbowColorGenerator::new(120.0, ERROR_S, ERROR_V, 45.0);
                    let mut item_colors = RainbowColorGenerator::new(0.0, ERROR_S, ERROR_V, 15.0);


                    let (message, area, labels, note): (_, _, _, Option<String>) = match self {
                        $(
                            $err_type::$variant { $($field),+ } => {
                                let err_area = $area.clone();
                                let labels = vec![
                                    $(
                                        ( $l_area, format!($fmt  $( , $(   $($c_e.fg(item_colors.next()))? $($e)?       ,)* )? ) ),
                                    )+
                                ];

                                ($msg, err_area, labels, $note)
                            }
                        )*
                    };

                    // epic
                    let mut report = Report::build(ReportKind::Error, area.name(), area.span.0)
                        .with_message(message.to_string() + "\n");

                    for (c, s) in labels {
                        report = report.with_label(
                            Label::new(c.label())
                                .with_message(s)
                                .with_color(label_colors.next()),
                        )
                    }

                    if let Some(m) = &note {
                        report = report.with_note(m)
                    }

                    let ret = vec![];

                    report
                        .finish()
                        .write((source.name(), Source::from(code)), &mut ret)
                        .unwrap();

                    std::str::from_utf8(&ret).unwrap().to_string()
                }
            }
        )*

    };
}
