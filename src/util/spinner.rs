use colored::Colorize;
use spinoff::spinners::SpinnerFrames;
use spinoff::{Spinner as SSpinner, *};

pub(crate) struct Spinner {
    frames: SpinnerFrames,
    disabled: bool,
    spinner: Option<(SSpinner, String)>,
}

impl Spinner {
    pub fn new(disabled: bool) -> Self {
        Self {
            frames: spinner!(["◜ ", "◠ ", "◝ ", "◞ ", "◡ ", "◟ "], 50),
            spinner: None,
            disabled,
        }
    }

    pub fn start(&mut self, msg: String) {
        if self.disabled {
            println!("{msg}");
        } else {
            self.spinner = Some((SSpinner::new(self.frames.clone(), msg.clone(), None), msg));
        }
    }

    pub fn fail(&mut self, msg: Option<String>) {
        if let Some((spinner, curr_msg)) = self.spinner.take() {
            spinner.stop_with_message(&format!("{curr_msg} ❌"));
        } else {
            println!("\n{}", "══════════════════════════════════".dimmed().bold());
        }
        if let Some(m) = msg {
            eprintln!("\n{m}");
        }
    }

    pub fn complete(&mut self, msg: Option<String>) {
        if let Some((spinner, curr_msg)) = self.spinner.take() {
            if let Some(m) = msg {
                spinner.stop_with_message(&format!("{curr_msg} ✅",));
                println!("{m}");
            } else {
                spinner.clear();
                println!("{curr_msg} ✅")
            }
            return;
        }
        if let Some(m) = msg {
            println!("{m}");
        }
    }
}
