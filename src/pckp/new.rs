use std::{env::current_dir, fs::{create_dir, File}, io::Write};

use crate::pckp::util;

pub fn new(target_directory: String) {
    let mut path = current_dir().unwrap();
    path.push(&target_directory);
    match create_dir(&path) {
        Ok(_) => {},
        Err(e) => { util::pckp_error(format!("Failed to create folder: {}", e.to_string()).as_str()) }
    };
    path.push("pckp.yaml");
    let mut pckp_yaml = File::create(&path).unwrap();
    pckp_yaml.write_all(format!("name: {}
version: 0.0.1
folders:
- src
dependencies: []
", target_directory).as_bytes()).unwrap();
    path.pop();
    path.push("src");
    create_dir(&path).unwrap();
    path.push("main.spwn");
    let mut main_spwn = File::create(&path).unwrap();
    main_spwn.write_all(format!("
$.print(\"Hello world!\\nfrom: {}\");
", target_directory).as_bytes()).unwrap();

}
