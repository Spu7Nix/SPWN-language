use std::env::current_dir;
use std::fs::{create_dir, File};
use std::io::Write;
use std::path::PathBuf;

use crate::pckp::util;
use crate::util::string::unindent;

pub fn new(target_directory: PathBuf) {
    let directory_name = target_directory.file_name().unwrap().to_str().unwrap();
    let mut path = current_dir().unwrap();
    path.push(&target_directory);
    match create_dir(&path) {
        Ok(_) => {},
        Err(e) => { util::pckp_error(format!("Failed to create folder: {}", e).as_str()) }
    };
    path.push("pckp.yaml");
    let mut pckp_yaml = File::create(&path).unwrap();
    pckp_yaml.write_all(
        unindent(
            format!(
                "
                    name: {}
                    version: 0.0.1
                    folders:
                    - src
                    dependencies: []
                ", directory_name
            ),
            true,
            true,
        )
            .as_bytes()
    ).unwrap();
    path.pop();
    path.push("src");
    create_dir(&path).unwrap();
    path.push("main.spwn");
    let mut main_spwn = File::create(&path).unwrap();
    main_spwn.write_all(
        unindent(
            format!("$.print(\"Hello world!\\nfrom: {}\");", directory_name),
            true,
            true,
        )
        .as_bytes()
    ).unwrap()
}
