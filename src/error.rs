use ariadne::{Label, Report, ReportKind};

use thiserror::Error;

use crate::sources::{CodeArea, SpwnCache};

const ERROR_S: f64 = 0.4;
const ERROR_V: f64 = 1.0;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Expected `{expected}`, found {typ} `{found}`")]
    Expected {
        expected: String,
        found: String,
        typ: String,
        area: CodeArea,
    },
}

impl SyntaxError {
    const MESSAGE: &'static str = "Syntax Error";

    pub fn raise(self, cache: SpwnCache) {
        let mut colors = RainbowColorGenerator::new(120.0, ERROR_S, ERROR_V, 20.0);

        let (area, labels, note): (_, _, Option<String>) = match self {
            SyntaxError::Expected { ref area, .. } => {
                let labels = vec![(area, self.to_string())];

                (area, labels, None)
            }
        };

        let mut report = Report::build(ReportKind::Error, area.source.clone(), area.span.0)
            .with_message(Self::MESSAGE.to_string() + "\n");

        for (c, s) in labels {
            report = report.with_label(
                Label::new(c.to_owned())
                    .with_message(s)
                    .with_color(colors.next()),
            )
        }

        if let Some(m) = &note {
            report = report.with_note(m)
        }

        report.finish().eprint(cache).unwrap();
    }
}

#[derive(Debug)]
pub struct RainbowColorGenerator {
    h: f64,
    s: f64,
    v: f64,
    shift: f64,
}

impl RainbowColorGenerator {
    pub fn new(h: f64, s: f64, v: f64, shift: f64) -> Self {
        Self { h, s, v, shift }
    }
    pub fn next(&mut self) -> ariadne::Color {
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

// Custom wrapper `Result` type as all errors will be syntax errors.
pub type Result<T> = std::result::Result<T, SyntaxError>;
