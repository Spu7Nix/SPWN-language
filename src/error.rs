use std::io::{BufWriter, Write};

use ariadne::{sources, CharSet, Color, Config, Label, Report, ReportKind};
use colored::Colorize;

use crate::sources::CodeArea;
use crate::util::hsv_to_rgb;

#[derive(Debug)]
pub struct ErrorReport {
    pub title: String,
    pub message: String,
    pub labels: Vec<(CodeArea, String)>,
    pub note: Option<String>,
    pub report_kind: ReportKind<'static>, // we never use custom message
}

impl ErrorReport {
    pub fn fuck(&self) {
        println!("TITLE: {}", self.title);
        println!("MESSAGE: {}", self.message);
        println!("------------------------------");
        for (area, label) in &self.labels {
            let snippet = area.src.read().unwrap()[area.span.start..area.span.end].bright_red();
            println!("{} -> {}", snippet, label);
        }
        println!("------------------------------");
        if let Some(n) = &self.note {
            println!("NOTE: {}", n);
        }
    }
}

impl std::error::Error for ErrorReport {}
impl std::fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut label_colors = RainbowColorGenerator::new(308.0, 0.5, 0.95, 35.0);

        let charset = match std::env::var("USE_ASCII").ok() {
            Some(_) => CharSet::Ascii,
            None => CharSet::Unicode,
        };

        let color = match self.report_kind {
            ReportKind::Advice => Color::Blue,
            ReportKind::Warning => Color::Yellow,
            ReportKind::Error => Color::Red,
            ReportKind::Custom(_, c) => c,
        };

        let mut report = Report::build(ReportKind::Custom(&self.title, color), "", 0)
            .with_config(Config::default().with_char_set(charset))
            .with_message(&self.message);

        let mut source_vec = vec![];

        for (area, msg) in &self.labels {
            source_vec.push(area.src.clone());

            let col = label_colors.next();

            report = report.with_label(
                Label::new((area.src.hyperlink(), area.span.into()))
                    .with_message(msg)
                    .with_color(Color::RGB(col.0, col.1, col.2)),
            );
        }

        if let Some(n) = &self.note {
            report = report.with_note(n)
        }

        let mut out = BufWriter::new(Vec::new());

        report
            .finish()
            .write_for_stdout(
                sources(
                    source_vec
                        .iter()
                        .map(|src| (src.hyperlink(), src.read().unwrap())),
                ),
                &mut out,
            )
            .unwrap();

        write!(f, "{}", String::from_utf8(out.buffer().to_vec()).unwrap())
    }
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
        $(#[$($meta:tt)*])*
        pub enum $enum:ident $(<$lt:lifetime>)? {
            $(
                #[
                    Message: $msg:expr, Note: $note:expr;
                    Main Area: $main_area:expr;
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
        $(#[$($meta)*])*
        pub enum $enum $(<$lt>)?  {
            $(
                $err_name {
                    $(
                        $field: $typ,
                    )*
                    $($call_stack: Vec<CallInfo>)?
                },
            )*
        }

        impl$(<$lt>)? $enum$(<$lt>)? {
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
                            report_kind: ariadne::ReportKind::Error,
                            note: $note.map(|s: String| s.to_string()),
                        },
                    )*
                }
            }

            pub fn get_main_area(&self) -> &CodeArea {
                match self {
                    $(
                        $enum::$err_name { $($field,)* $($call_stack)? } => $main_area,
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
        let h0 = self.h / 360.0;

        self.h = (self.h + self.hue_shift).rem_euclid(360.0);

        hsv_to_rgb(h0, self.s, self.v)
    }
}
