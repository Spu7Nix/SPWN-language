#![allow(unused_variables)]

use std::path::PathBuf;

use crate::run_spwn;

macro_rules! run_test {
    {$([$attr:ident])? NAME: $name:ident CODE: $code:literal $(OUTPUT: $output:literal)?} => {
        #[test]
        $(#[$attr])?
        fn $name() {
            let res = match run_spwn($code.to_string(), vec![PathBuf::from("./")], false) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("{}", e);
                    panic!("test {} failed, see message above", stringify!($name));
                }
            };
            $(assert_eq!(res[0].trim(), $output.trim());)?
        }
    };
}

// just basic parsing
run_test! {
    NAME: basic_parsing
    CODE: r"
#[no_std]
a = 1
b = 2

m = (a, let b, &c) => a + b + c
m(1, 2, 3)

() {
    m(1, 2, 3)
} ()

m2 = (a, b, c) -> @number {
    return a + b + c
}
    "
}

// mutability
run_test! {
    [should_panic]
    NAME: mutability1
    CODE: r"
#[no_std]
a = 1
a += 1
    "
}

run_test! {
    [should_panic]
    NAME: mutability2
    CODE: r"
#[no_std]
a = 1
m = (&b) {
    b += 1
}
m(a)
    "
}

// non-std things
run_test! {
    NAME: print_basic
    CODE: r"
#[no_std]
$.print('Hello')
$.print(r'Hello\nWorld')
    "
    OUTPUT: r"
Hello
Hello\nWorld
    "
}

run_test! {
    NAME: math
    CODE: r"#[no_std] $.print(10 * 2 + 2 * 3^2)"
    OUTPUT: r"38"
}

// std things

// strings
run_test! {
    NAME: string_fmt
    CODE: r"
$.print('Hello {}!'.fmt('world'))
    "
    OUTPUT: r"
Hello world!
    "
}

// arrays
run_test! {
    NAME: arr
    CODE: r"
arr = [1, 2, 3, 4]
$.print(arr.filter(>2))
$.print(arr.filter(>=2))
$.print(arr.filter(a => a > 2))
$.print(arr.all(a => a > 0))
$.print(arr.all(>0))
    "
    OUTPUT: r"
[3, 4]
[2, 3, 4]
[3, 4]
true
true
    "
}
