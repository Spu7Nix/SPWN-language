pub mod compiler_info;

use compiler_info::{CodeArea, CompilerInfo};

use ariadne::Fmt;

use internment::LocalIntern;
use shared::{BreakType, FileRange, SpwnSource};

pub enum RuntimeError {
    UndefinedErr {
        undefined: String,
        desc: String,
        info: CompilerInfo,
    },

    PackageSyntaxError {
        err: SyntaxError,
        info: CompilerInfo,
    },

    PackageError {
        err: Box<RuntimeError>,
        info: CompilerInfo,
    },

    TypeError {
        expected: String,
        found: String,
        val_def: CodeArea,
        info: CompilerInfo,
    },

    PatternMismatchError {
        pattern: String,
        val: String,
        pat_def: CodeArea,
        val_def: CodeArea,
        info: CompilerInfo,
    },

    CustomError(ErrorReport),

    BuiltinError {
        builtin: String,
        message: String,
        info: CompilerInfo,
    },

    MutabilityError {
        val_def: CodeArea,
        info: CompilerInfo,
    },
    ContextChangeMutateError {
        val_def: CodeArea,
        info: CompilerInfo,
        context_changes: Vec<CodeArea>,
    },
    ContextChangeError {
        message: String,
        info: CompilerInfo,
        context_changes: Vec<CodeArea>,
    },

    BreakNeverUsedError {
        breaktype: BreakType,
        info: CompilerInfo,
        broke: CodeArea,
        dropped: CodeArea,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RainbowColorGenerator {
    h: f64,
    s: f64,
    b: f64,
}

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, b: f64) -> Self {
        Self { h, s, b }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> ariadne::Color {
        self.h += 20.0;
        self.h %= 360.0;

        let hsl = *self;

        let c = (1.0 - (hsl.b * 2.0 - 1.0).abs()) * hsl.s;
        let h = hsl.h / 60.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = hsl.b - c * 0.5;

        let (red, green, blue) = if h >= 0.0 && h < 0.0 {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        ariadne::Color::RGB(
            ((red + m) * 255.0) as u8,
            ((green + m) * 255.0) as u8,
            ((blue + m) * 255.0) as u8,
        )
    }
}
pub fn create_report(rep: ErrorReport) -> ariadne::Report<CodeArea> {
    use ariadne::{Config, Label, Report, ReportKind};

    let info = rep.info;
    let message = rep.message;
    let labels = rep.labels;
    let note = rep.note;
    let position = info.position;

    let mut colors = RainbowColorGenerator::new(0.0, 1.5, 0.8);

    let mut report = Report::build(
        ReportKind::Error,
        position.file.as_ref().clone(),
        position.pos.0,
    )
    .with_config(Config::default().with_cross_gap(true))
    .with_message(message.clone());

    let mut i = 1;
    for area in info.call_stack {
        let color = colors.next();
        report = report.with_label(
            Label::new(area)
                .with_order(i)
                .with_message(&format!(
                    "{}: Error comes from this macro call",
                    i.to_string().fg(color)
                ))
                .with_color(color)
                .with_priority(1),
        );
        i += 1;
    }

    if labels.is_empty() || !labels.iter().any(|(a, _)| a == &position) {
        let color = colors.next();
        report = report.with_label(
            Label::new(position)
                .with_order(i)
                .with_color(color)
                .with_message(message)
                .with_priority(2),
        );
    }
    if i == 1 && labels.len() == 1 {
        let color = colors.next();
        report = report.with_label(
            Label::new(labels[0].0)
                .with_message(labels[0].1.clone())
                .with_order(i)
                .with_color(color)
                .with_priority(2),
        );
    } else if !labels.is_empty() {
        for (area, label) in labels {
            let color = colors.next();
            report = report.with_label(
                Label::new(area)
                    .with_message(&format!("{}: {}", i.to_string().fg(color), label))
                    .with_order(i)
                    .with_color(color)
                    .with_priority(2),
            );
            i += 1;
        }
    }
    if let Some(note) = note {
        report = report.with_note(note);
    }
    report.finish()
}

pub fn create_error(
    info: CompilerInfo,
    message: &str,
    labels: &[(CodeArea, &str)],
    note: Option<&str>,
) -> ErrorReport {
    ErrorReport {
        info: info.clone(),
        message: message.to_string(),
        labels: labels
            .iter()
            .map(|(a, s)| match a.file.as_ref() {
                SpwnSource::File(file) => {
                    if !file.exists() {
                        (
                            CodeArea {
                                pos: (0, 0),
                                ..info.position
                            },
                            s.to_string(),
                        )
                    } else {
                        (*a, s.to_string())
                    }
                }
                _ => (*a, s.to_string()),
            })
            .collect(),
        note: note.map(|s| s.to_string()),
    }
}

pub struct ErrorReport {
    pub info: CompilerInfo,
    pub message: String,
    pub labels: Vec<(CodeArea, String)>,
    pub note: Option<String>,
}

impl From<RuntimeError> for ErrorReport {
    fn from(err: RuntimeError) -> ErrorReport {
        let mut colors = RainbowColorGenerator::new(120.0, 1.5, 0.8);
        let a = colors.next();
        let b = colors.next();

        match err {
            RuntimeError::UndefinedErr {
                undefined,
                desc,
                info,
            } => create_error(
                info.clone(),
                &format!("Use of undefined {}", desc),
                &[(
                    info.position,
                    &format!("'{}' is undefined", undefined.fg(b)),
                )],
                None,
            ),
            RuntimeError::PackageSyntaxError { err, info } => {
                let syntax_error = ErrorReport::from(err);
                let mut labels = vec![(info.position, "Error when parsing this library/module")];
                labels.extend(syntax_error.labels.iter().map(|(a, b)| (*a, b.as_str())));
                create_error(info, &syntax_error.message, &labels, None)
            }

            RuntimeError::PackageError { err, info } => {
                let syntax_error = ErrorReport::from(*err);
                let mut labels = vec![(info.position, "Error when running this library/module")];
                labels.extend(syntax_error.labels.iter().map(|(a, b)| (*a, b.as_str())));
                create_error(info, &syntax_error.message, &labels, None)
            }

            RuntimeError::TypeError {
                expected,
                found,
                info,
                val_def,
            } => create_error(
                info.clone(),
                "Type mismatch",
                &[
                    (
                        val_def,
                        &format!("Value defined as {} here", found.clone().fg(b)),
                    ),
                    (
                        info.position,
                        &format!("Expected {}, found {}", expected.fg(a), found.fg(b)),
                    ),
                ],
                None,
            ),

            RuntimeError::PatternMismatchError {
                pattern,
                val,
                info,
                val_def,
                pat_def,
            } => create_error(
                info.clone(),
                "Pattern mismatch",
                &[
                    (
                        val_def,
                        &format!("Value defined as {} here", val.clone().fg(b)),
                    ),
                    (
                        pat_def,
                        &format!("Pattern defined as {} here", pattern.clone().fg(b)),
                    ),
                    (
                        info.position,
                        &format!("This {} is not {}", val.fg(a), pattern.fg(b)),
                    ),
                ],
                None,
            ),

            RuntimeError::CustomError(report) => report,

            RuntimeError::BuiltinError {
                message,
                info,
                builtin,
            } => create_error(
                info.clone(),
                &format!("Error when using built-in function: {}", builtin),
                &[(info.position, &message)],
                None,
            ),

            RuntimeError::MutabilityError { val_def, info } => create_error(
                info.clone(),
                "Attempted to change immutable variable",
                &[
                    (val_def, "Value was defined as immutable here"),
                    (info.position, "This tries to change the value"),
                ],
                None,
            ),

            RuntimeError::ContextChangeMutateError {
                val_def,
                info,
                context_changes,
            } => {
                let mut labels = vec![(val_def, "Value was defined here")];

                #[allow(clippy::comparison_chain)]
                if context_changes.len() == 1 {
                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                } else if context_changes.len() > 1 {
                    labels.push((*context_changes.last().unwrap(), "Context was changed here"));

                    for change in context_changes[1..(context_changes.len() - 1)].iter().rev() {
                        labels.push((*change, "This changes the context inside the macro"));
                    }

                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                }

                labels.push((info.position, "Attempted to change value here"));

                create_error(
                    info,
                    "Attempted to change a variable defined in a different trigger function context",
                    &labels,
                    Some("Consider using a counter"),
                )
            }

            RuntimeError::ContextChangeError {
                message,
                info,
                context_changes,
            } => {
                let mut labels = Vec::new();

                #[allow(clippy::comparison_chain)]
                if context_changes.len() == 1 {
                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                } else if context_changes.len() > 1 {
                    labels.push((*context_changes.last().unwrap(), "Context was changed here"));

                    for change in context_changes[1..(context_changes.len() - 1)].iter().rev() {
                        labels.push((*change, "This changes the context inside the macro"));
                    }

                    labels.push((
                        context_changes[0],
                        "New trigger function context was defined here",
                    ));
                }

                create_error(info, &message, &labels, None)
            }

            RuntimeError::BreakNeverUsedError {
                info,
                breaktype,
                broke,
                dropped,
                reason,
            } => create_error(
                info,
                &format!(
                    "{} statement never used",
                    match breaktype {
                        BreakType::ContinueLoop => "Continue",
                        BreakType::Loop => "Break",
                        BreakType::Macro(_, _) => "Return",
                        BreakType::Switch(_) => unreachable!("Switch break in the wild"),
                    }
                ),
                &[
                    (broke, "Declared here"),
                    (
                        dropped,
                        &format!("Can't reach past here because {}", reason),
                    ),
                ],
                None,
            ),
        }
    }
}

pub enum SyntaxError {
    ExpectedErr {
        expected: String,
        found: String,
        pos: FileRange,
        file: SpwnSource,
    },
    UnexpectedErr {
        found: String,
        pos: FileRange,
        file: SpwnSource,
    },
    SyntaxError {
        message: String,
        pos: FileRange,
        file: SpwnSource,
    },
    CustomError(ErrorReport),
}

impl From<SyntaxError> for ErrorReport {
    fn from(err: SyntaxError) -> ErrorReport {
        //write!(f, "SuperErrorSideKick is here!")
        let mut colors = RainbowColorGenerator::new(60.0, 1.0, 0.8);
        let a = colors.next();
        let b = colors.next();
        match err {
            SyntaxError::ExpectedErr {
                expected,
                found,
                pos,
                file,
            } => create_error(
                CompilerInfo::from_area(CodeArea {
                    pos,
                    file: LocalIntern::new(file.clone()),
                }),
                "Syntax error",
                &[(
                    CodeArea {
                        pos,
                        file: LocalIntern::new(file),
                    },
                    &format!(
                        "{} {}, {} {}",
                        "Expected".fg(b),
                        expected,
                        "found".fg(a),
                        found
                    ),
                )],
                None,
            ),

            SyntaxError::UnexpectedErr { found, pos, file } => create_error(
                CompilerInfo::from_area(CodeArea {
                    pos,
                    file: LocalIntern::new(file.clone()),
                }),
                "Syntax error",
                &[(
                    CodeArea {
                        pos,
                        file: LocalIntern::new(file),
                    },
                    &format!("Unexpected {}", found),
                )],
                None,
            ),

            SyntaxError::SyntaxError { message, pos, file } => create_error(
                CompilerInfo::from_area(CodeArea {
                    pos,
                    file: LocalIntern::new(file.clone()),
                }),
                "Syntax error",
                &[(
                    CodeArea {
                        pos,
                        file: LocalIntern::new(file),
                    },
                    &message,
                )],
                None,
            ),

            SyntaxError::CustomError(report) => report,
        }
    }
}
