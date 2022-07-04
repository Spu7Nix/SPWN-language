



use ariadne::{ReportBuilder, Report, Label, ReportKind, Fmt};

use crate::{CodeArea, SpwnCache, SpwnSource, lexer::Token};





#[derive(Debug, Clone)]
pub enum SyntaxError {
    Expected {
        expected: String,
        typ: String,
        found: String,
        area: CodeArea,
    },
}




pub trait ToReport {
    fn to_report(&self) -> ErrorReport;
}


#[derive(Debug)]
pub struct ErrorReport {
    source: CodeArea,
    message: String,
    labels: Vec<(CodeArea, String)>,
    note: Option<String>,
}

impl From<CodeArea> for SpwnSource {
    fn from(area: CodeArea) -> Self {
        area.source
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
        
        let (r, g, b) =
            if (0.0..1.0).contains(&h1) {
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
            ( (r + m) * 255.0) as u8,
            ( (g + m) * 255.0) as u8,
            ( (b + m) * 255.0) as u8,
        )
    }
}

const ERROR_S: f64 = 0.4;
const ERROR_V: f64 = 1.0;




impl ErrorReport {
    pub fn print_error(&self, cache: SpwnCache) {

        let mut colors = RainbowColorGenerator::new(0.0, ERROR_S, ERROR_V, 60.0);


        let mut report: ReportBuilder<CodeArea> = Report::build(ReportKind::Error, self.source.clone(), self.source.span.0)
        .with_message(self.message.clone() + "\n");


        for (c, s) in &self.labels {
            report = report.with_label( Label::new(c.clone()).with_message(s).with_color(colors.next()) )
        }

        if let Some(m) = &self.note {
            report = report.with_note(m)
        }
        report.finish().print(cache).unwrap();

    }
}


impl ToReport for SyntaxError {
    fn to_report(&self) -> ErrorReport {
        let mut colors = RainbowColorGenerator::new(120.0, ERROR_S, ERROR_V, 20.0);

        match self {
            SyntaxError::Expected {
                expected,
                typ,
                found,
                area,
            } => ErrorReport {
                source: area.clone(),
                message: "Syntax error".to_string(),
                labels: vec![
                    (area.clone(), format!("Expected {}, found {} {}", expected.fg(colors.next()), typ.fg(colors.next()), found.fg(colors.next())))
                ],
                note: None,
            },
        }
    }
}






