// pub mod builtin_funcs;
pub mod builtin_utils;
pub mod core;

#[test]
fn gen_all_core() {
    use std::process::Command;

    use regex::Regex;

    let path = std::path::PathBuf::from("./libraries/core/lib.spwn");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    Command::new("cargo")
        .args(["test", "--", "core_gen"])
        .spawn()
        .unwrap();

    let test_parser = Regex::new(r"(.*?)(?P<test_name>[a-zA-Z]*)_core_gen: test").unwrap();

    let output = Command::new("cargo")
        .args(["test", "--", "--list", "--format", "terse"])
        .output()
        .expect("failed to get tests")
        .stdout;

    let tests = String::from_utf8_lossy(&output);

    let mut lib_spwn = String::new();

    for test in test_parser.captures_iter(&tests) {
        lib_spwn.push_str(&format!(
            "import \"{}.spwn\"\n",
            test.name("test_name").unwrap().as_str(),
        ))
    }

    std::fs::write(path, &lib_spwn).unwrap();
}

// #\[doc\((.*?)\)\]
