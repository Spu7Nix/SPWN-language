use std::io::Write;

use ariadne::{sources, Label, Report, ReportKind};

use crate::sources::CodeArea;
use crate::vm::context::CallStackItem;

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
                            $l_area:expr => $fmt:literal $(: $( $($e:expr)? $(=>($e_no_col:expr))? ),+)?;
                        )+
                        $(-> $spread:expr)?
                    ]
                ]
                $err_name:ident {
                    $(
                        $field:ident: $typ:ty,
                    )*

                    $([$call_stack:ident])?
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
                    $($call_stack: Vec<CallStackItem>)?
                },
            )*
        }
        use $crate::error::ErrorReport;

        impl $enum {
            pub fn to_report(&self $(, $extra_arg: $extra_type)* ) -> ErrorReport {
                use colored::Colorize;
                use $crate::error::RainbowColorGenerator;

                let mut info_colors = RainbowColorGenerator::new(166.0, 0.5, 0.95, 35.0);

                match self {
                    $(
                        $enum::$err_name { $($field,)* $($call_stack)? } => ErrorReport {
                            title: $title.to_string(),
                            message: ($msg).to_string(),
                            labels: {
                                #[allow(unused_mut)]
                                let mut v = vec![
                                    $(
                                        ($l_area.clone(), format!($fmt $(, $(

                                            $(
                                                {
                                                    let col = info_colors.next();
                                                    $e.to_string().truecolor(col.0, col.1, col.2).bold()
                                                }
                                            )?

                                            $($e_no_col.to_string())?

                                        ),* )?)),
                                    )*
                                ];
                                $(
                                    for i in $spread {
                                        v.push(i)
                                    }
                                )?

                                $(
                                    for area in $call_stack.iter().filter_map(|i| i.call_area.clone()) {
                                        v.push((area, "Error comes from this macro call".to_string()));
                                    }
                                )?
                                v
                            },
                            note: $note.map(|s: String| s.to_string()),
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
    hue_shift: f64,
}

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, v: f64, hue_shift: f64) -> Self {
        Self { h, s, v, hue_shift }
    }

    pub fn next(&mut self) -> (u8, u8, u8) {
        let c = self.v * self.s;
        let h0 = self.h / 60.0;

        // "gfdgdf".bold().hidden()

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

        self.h = (self.h + self.hue_shift).rem_euclid(360.0);

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

impl ErrorReport {
    pub fn display(&self) {
        let mut label_colors = RainbowColorGenerator::new(308.0, 0.5, 0.95, 35.0);

        let mut report = Report::build(ReportKind::Error, "", 0).with_message(&self.message);

        let mut source_vec = vec![];

        for (area, msg) in &self.labels {
            source_vec.push(area.src.clone());

            let col = label_colors.next();

            report = report.with_label(
                Label::new((area.src.hyperlink(), area.span.into()))
                    .with_message(msg)
                    .with_color(ariadne::Color::RGB(col.0, col.1, col.2)),
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
                    .map(|src| (src.hyperlink(), src.read().unwrap())),
            ))
            .unwrap();
    }
}
