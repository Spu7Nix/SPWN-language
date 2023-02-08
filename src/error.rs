use std::io::{BufWriter, Write};

use ariadne::{sources, Label, Report, ReportKind};

use crate::sources::CodeArea;
use crate::util::hsv_to_rgb;

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

        impl std::error::Error for $enum {}
        impl std::fmt::Display for $enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.to_report().to_string())
            }
        }

        impl $enum {
            pub fn to_report(&self $(, $extra_arg: $extra_type)* ) -> $crate::error::ErrorReport {
                use colored::Colorize;
                use $crate::error::RainbowColorGenerator;

                let mut info_colors = RainbowColorGenerator::new(166.0, 0.5, 0.95, 35.0);

                match self {
                    $(
                        $enum::$err_name { $($field,)* $($call_stack)? } => $crate::error::ErrorReport  {
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
        let h0 = self.h / 60.0;

        self.h = (self.h + self.hue_shift).rem_euclid(360.0);

        hsv_to_rgb(h0, self.s, self.v)
    }
}

impl ToString for ErrorReport {
    fn to_string(&self) -> String {
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

        if let Some(n) = &self.note {
            report = report.with_note(n)
        }

        let mut buf = BufWriter::new(Vec::new());

        report
            .finish()
            .write_for_stdout(
                sources(
                    source_vec
                        .iter()
                        .map(|src| (src.hyperlink(), src.read().unwrap())),
                ),
                &mut buf,
            )
            .unwrap();

        String::from_utf8(buf.into_inner().unwrap()).unwrap()
    }
}

impl ErrorReport {
    pub fn display(&self) {
        eprintln!("{}", self.to_string());
    }
}
