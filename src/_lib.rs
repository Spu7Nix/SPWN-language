//#![feature(arbitrary_enum_discriminant)]
use crate::optimize::optimize;
#[cfg_attr(target_os = "macos", path = "editorlive_mac.rs")]
#[cfg_attr(windows, path = "editorlive_win.rs")]
#[cfg_attr(
    not(any(target_os = "macos", windows)),
    path = "editorlive_unavailable.rs"
)]
//#[cfg_attr(target_os = "macos", path = "editorlive_mac.rs")]
//#[cfg_attr(windows, path = "editorlive_win.rs")]
//mod editorlive;
use termcolor::Color;

use crate::parser::*;

use std::path::PathBuf;

use std::io;
use tempfile::NamedTempFile;

//library has no console output
pub fn print_with_color(_text: &str, _color: Color) {}
pub fn eprint_with_color(_text: &str, _color: Color) {}

pub const STD_PATH: &str = "std";

#[derive(Clone)]
pub struct Compiler {
    opti_enabled: bool,
    included_paths: Vec<PathBuf>,
    unparsed_code: String,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            opti_enabled: true,
            included_paths: vec![
                std::env::current_dir().expect("Cannot access current directory"),
                std::env::current_exe()
                    .expect("Cannot access directory of executable")
                    .parent()
                    .expect("Executable must be in some directory")
                    .to_path_buf(),
            ],
            unparsed_code: String::new(),
        }
    }
    pub fn add_include(&mut self, path_str: String) -> io::Result<()> {
        self.included_paths.push({
            let path = PathBuf::from(path_str);
            path.read_dir()?;
            path
        });
        Ok(())
    }
    pub fn set_code(&mut self, code: String) {
        self.unparsed_code = code;
    }
    /*pub fn _run(code: String, included_paths: Vec<PathBuf>, opti_enabled: bool) -> Result<String, Box<dyn std::error::Error>> {
        let file = NamedTempFile::new()?;
        let mut pbuf = PathBuf::new();
        pbuf.push(file.path());
        let (statements, notes) = parse_spwn(code, pbuf.clone())?;

        let mut compiled = compiler::compile_spwn(
            statements,
            pbuf,
            included_paths,
            notes,
        )?;

        let has_stuff = compiled.func_ids.iter().any(|x| !x.obj_list.is_empty());
        if opti_enabled && has_stuff {
            compiled.func_ids = optimize(compiled.func_ids, compiled.closed_groups);
        }

        let mut objects = levelstring::apply_fn_ids(&compiled.func_ids);

        objects.extend(compiled.objects);

        let (new_ls, _) = levelstring::append_objects(objects, &String::new())?;

        Ok(new_ls)
    }*/
}
