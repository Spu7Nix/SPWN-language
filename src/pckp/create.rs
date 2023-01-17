use std::path::PathBuf;
use std::fs::{create_dir, write};

const PCKP_LIBRARIES_DIR: &str = "pckp_libraries";
const VRSN_FILE: &str = ".vrsn";

macro_rules! create_function {
    ($func_name:ident($pathbuf:expr, $create:expr $(=>($($opt:expr$(,)*)+))?, $iskind:ident, $typ:literal$(,)?)) => {
        pub fn $func_name() -> PathBuf {
            let output = $pathbuf;
            match output.exists() {
                true => {
                    if !output.$iskind() { panic!("🤨 `{:?}` isn't a {}", output, $typ) }
                }
                false => {
                    $create(&output $(,$($opt,)+)?).unwrap();
                }
            }
            output
        }
    };
}

create_function!(
    create_pckp_dir(
        PathBuf::from(PCKP_LIBRARIES_DIR),
        create_dir,
        is_dir,
        "folder",
    )
);

create_function!(
    create_vrsn_file(
        create_pckp_dir().join(VRSN_FILE),
        write => (""),
        is_file,
        "file",
    )
);
