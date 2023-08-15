use ahash::AHashSet;
use serde::{Deserialize, Serialize};

use crate::sources::CodeSpan;

// any features deprecated from <0.9
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DeprecatedFeatures {
    // use of `type @a` without explicit members `type @a { ... }`
    pub empty_type_def: AHashSet<CodeSpan>,
    // use of `let` instead of `mut`
    pub let_not_mut: AHashSet<CodeSpan>,
}

impl DeprecatedFeatures {
    // used in the parser to merge after cloning
    pub fn extend(&mut self, other: DeprecatedFeatures) {
        self.empty_type_def.extend(other.empty_type_def);
        self.let_not_mut.extend(other.let_not_mut);
    }
}
