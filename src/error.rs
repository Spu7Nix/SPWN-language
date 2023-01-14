use std::{collections::HashMap, io::Write};

use ariadne::sources;
use colored::Colorize;

use crate::sources::{CodeArea, SpwnSource};

#[derive(Debug)]
pub struct ErrorReport {
    pub title: String,
    pub message: String,
    pub labels: Vec<(CodeArea, String)>,
    pub note: Option<String>,
}

#[macro_export]
macro_rules! error_maker {
    (
        Title: $title:literal
        Extra: {
            $(
                $extra_arg:ident: $extra_type:ty,
            )*
        }
        pub enum $enum:ident {
            $(
                #[
                    Message: $msg:expr, Note: $note:expr;
                    Labels: [
                        $(
                            $l_area:expr => $fmt:literal $(: $($e:expr),+)?;
                        )+
                        $(-> $spread:expr)?
                    ]
                ]
                $err_name:ident {
                    $(
                        $field:ident: $typ:ty,
                    )*
                },
            )*
        }
    ) => {
        #[derive(Debug, Clone)]
        pub enum $enum {
            $(
                $err_name {
                    $(
                        $field: $typ,
                    )*
                },
            )*
        }
        use $crate::error::ErrorReport;
        impl $enum {
            pub fn to_report(&self $(, $extra_arg: $extra_type)* ) -> ErrorReport {
                use colored::Colorize;
                match self {
                    $(
                        $enum::$err_name { $($field,)* } => ErrorReport {
                            title: $title.to_string(),
                            message: ($msg).to_string(),
                            labels: {
                                let v = vec![
                                    $(
                                        ($l_area.clone(), format!($fmt $(, $($e.to_string().truecolor(255, 234, 128).bold()),*)?)),
                                    )*
                                ];
                                $(
                                    for i in $spread {
                                        v.push(i)
                                    }
                                )?
                                v
                            },
                            note: $note,
                        },
                    )*
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct RainbowColorGenerator {
    h: f64,
    s: f64,
    v: f64,
}

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, v: f64) -> Self {
        Self { h, s, v }
    }
    pub fn next(&mut self) -> ariadne::Color {
        let c = self.v * self.s;
        let h0 = self.h / 60.0;

        let x = c * (1.0 - (h0.rem_euclid(2.0) - 1.0).abs());

        let (r, g, b) = if (0.0..1.0).contains(&h0) {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h0) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h0) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h0) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h0) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let m = self.v - c;
        let (r, g, b) = (r + m, g + m, b + m);

        self.h = (self.h + 45.0).rem_euclid(360.0);

        ariadne::Color::RGB((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

impl ErrorReport {
    pub fn display(&self) {
        use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};

        let mut colors = RainbowColorGenerator::new(345.0, 0.73, 1.0);

        let mut report = Report::build(ReportKind::Error, "", 0).with_message(&self.message);

        let mut source_vec = vec![];

        for (area, msg) in &self.labels {
            source_vec.push(area.src.clone());
            report = report.with_label(
                Label::new((area.src.name(), area.span.into()))
                    .with_message(msg)
                    .with_color(colors.next()),
            );
        }
        println!("\n");
        std::io::stdout().flush().unwrap();

        if let Some(n) = &self.note {
            report = report.with_note(n)
        }

        report
            .finish()
            .eprint(sources(
                source_vec
                    .iter()
                    .map(|src| (src.name(), src.read().unwrap())),
            ))
            .unwrap();
    }
}
