use ahash::AHashSet;
use ariadne::ReportKind;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::compiler::attributes::Deprecated;
use crate::error::ErrorReport;
use crate::sources::{CodeArea, CodeSpan, Spanned};

// any features deprecated from <0.9 / any deprecated attributes
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DeprecatedFeatures {
    // use of `type @a` without explicit members `type @a { ... }`
    pub empty_type_def: AHashSet<CodeSpan>,
    // use of `let` instead of `mut`
    pub let_not_mut: AHashSet<CodeSpan>,

    pub user_deprecated: AHashSet<Spanned<Deprecated>>,
}

impl DeprecatedFeatures {
    // used in the parser to merge after cloning
    pub fn extend(&mut self, other: DeprecatedFeatures) {
        self.empty_type_def.extend(other.empty_type_def);
        self.let_not_mut.extend(other.let_not_mut);
    }
}

pub struct DeprecatedWarning {
    pub message: String,

    pub area: CodeArea,
    pub area_message: String,

    pub note: Option<String>,
}

impl DeprecatedWarning {
    pub fn to_report(&self) -> ErrorReport {
        ErrorReport {
            title: "Deprecation Warning".to_string(),
            message: self.message.truecolor(255, 223, 79).to_string(),
            labels: vec![(self.area.clone(), self.area_message.clone())],
            note: self.note.clone(),
            report_kind: ReportKind::Warning,
        }
    }
}
