// extra md syntax:
// ```spwn -> runnable example
// ``` -> non-runnable example
// !```spwn -> runnable example that errors
// !``` -> non-runnable example that errors
//
// [`<type name>`] -> auto link to <type name> docs
// [`path::to::type`] -> link to type
//
// > [!NOTE]
// > ....
//
// > [!WARNING]
// > ....
//
// > [!ERROR]
// > ....
//
// > [!IMPORTANT]
// > ....
//

mod error;

use std::path::PathBuf;
use std::rc::Rc;

use ahash::AHashMap;

use self::error::DocResult;
use crate::cli::DocSettings;
use crate::parsing::ast::Ast;
use crate::sources::SpwnSource;
use crate::util::interner::Interner;
use crate::util::ImmutStr;

pub struct DocCompiler<'a> {
    settings: &'a DocSettings,
    src: Rc<SpwnSource>,
    interner: Interner,
}

impl<'a> DocCompiler<'a> {
    pub fn new(settings: &'a DocSettings, src: Rc<SpwnSource>, interner: Interner) -> Self {
        Self {
            settings,
            src,
            interner,
        }
    }

    // pub fn make_area(&self, span: CodeSpan) -> CodeArea {
    //     CodeArea {
    //         span,
    //         src: Rc::clone(&self.src),
    //     }
    // }

    // fn intern(&self, s: &str) -> Spur {
    //     self.interner.borrow_mut().get_or_intern(s)
    // }

    // pub fn resolve(&self, s: &Spur) -> ImmutStr {
    //     self.Interner.resolve(s).into()
    // }

    // pub fn resolve_32(&self, s: &Spur) -> ImmutStr32 {
    //     String32::from_chars(self.Interner.resolve(s).chars().collect_vec()).into()
    // }

    pub fn compile(&self, ast: Ast) -> DocResult<()> {
        if let Some(t) = &self.settings.target_dir {
            std::env::set_current_dir(t).expect("todo thiserror");
        }

        Ok(())
    }
}
