pub type RandomState = ahash::RandomState;
pub type Interner = lasso::Rodeo<lasso::Spur, RandomState>;

use colored::Colorize;

pub fn hyperlink<T: ToString, U: ToString>(url: T, text: Option<U>) -> String {
    let text = match text {
        Some(t) => t.to_string(),
        None => url.to_string(),
    };

    format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url.to_string(), text)
        .blue()
        .underline()
        .bold()
        .to_string()
}
