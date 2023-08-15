use serde::{Deserialize, Serialize};

use crate::sources::CodeSpan;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DeprecatedFeatures {
    pub let_uses: Vec<CodeSpan>,
    pub empty_types: Vec<CodeSpan>,
}
