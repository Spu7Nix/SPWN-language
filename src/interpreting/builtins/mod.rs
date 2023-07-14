pub mod core;

#[rustfmt::skip]
#[macro_export]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:expr])*
        // temporary until 1.0
        $(#[raw($impl_raw:literal)])?
        impl $value_typ:ident {
            Constants:
            $(
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                $(#[doc = $const_doc:expr])+
                const $const:ident = $c_name:ident
                                        $( ( $( $c_val:expr ),* $(,)? ) )?
                                        $( { $( $c_n:ident: $c_val_s:expr ),* $(,)? } )?;
            )*

            Functions:
            $(
                // temporary until 1.0
                $(#[raw($fn_raw:literal)])?
                $(#[doc = $fn_doc:expr])+
                fn $func_name:ident($($args:tt)*)
                
                $(-> $ret_type:ident)? {
                    $($code:tt)*
                }
            )*
        }
    ) => {
        impl $crate::interpreting::value::type_aliases::$value_typ {
            pub fn get_override_fn(self, name: &str) -> Option<$crate::interpreting::value::BuiltinFn> {
                trait IntoValueShortcut {
                    fn into_value(self) -> $crate::interpreting::value::Value;
                }
                impl IntoValueShortcut for () {
                    fn into_value(self) -> $crate::interpreting::value::Value { $crate::interpreting::value::Value::Empty }
                }
                impl IntoValueShortcut for $crate::interpreting::value::Value {
                    fn into_value(self) -> $crate::interpreting::value::Value { self }
                }
                
                $(
                    #[allow(unused)]
                    fn $func_name(
                        mut args: Vec<$crate::interpreting::vm::ValueRef>,
                        vm: &mut $crate::Vm,
                        area: $crate::sources::CodeArea,
                    ) -> $crate::interpreting::vm::RuntimeResult<$crate::interpreting::value::Value> {
                        use $crate::interpreting::value::value_structs::*;
                        
                        impl_type! { @ArgsCheckCloneA[0](args, vm) $($args)* }
                        impl_type! { @ArgsA[0](args, vm, area) $($args)* }

                        Ok({ $($code)* }.into_value())
                    }
                )*

                match name {
                    $(
                        stringify!($func_name) => Some($crate::interpreting::value::BuiltinFn($func_name)),
                    )*
                    _ => None
                }
            }
        }

        paste::paste! {
            #[cfg(test)]
            mod [<$value_typ:snake _core_gen>] {
                #[test]
                pub fn [<$value_typ:snake _core_gen>]() {
                    let path = std::path::PathBuf::from(format!("{}{}.spwn", $crate::CORE_PATH, stringify!( [<$value_typ:snake>] )));

                    let consts: &[String] = &[
                        $(
                            indoc::formatdoc!("\t{const_raw}
                                \t#[doc({const_doc:?})]
                                \t{const_name}: {const_val},",
                                const_raw = stringify!($($const_raw)*),
                                const_doc = <[String]>::join(&[$($const_doc.to_string())*], "\n"),
                                const_name = stringify!($const),
                                const_val = $crate::compiling::bytecode::Constant::
                                    $c_name
                                        $( ( $( $c_val ),* ) )?
                                        $( { $( $c_n : $c_val_s ,)* } )?,
                            ),
                        )*
                    ];

                    let macros: &[String] = &[
                        $(
                            indoc::formatdoc!("\t{macro_raw}
                                \t#[doc({macro_doc:?})]
                                \t{macro_name}: #[builtin] ({macro_args}\n\t){macro_ret}{{}},",
                                macro_raw = { "" $(; $fn_raw)? },
                                macro_doc = {
                                    let doc = <[&'static str]>::join(&["\n\t", $($fn_doc),*], "\n");
                                    assert!(doc != "", "ERROR: empty doc for builtin fn");
                                    unindent::unindent(&doc)
                                },
                                macro_name = stringify!($func_name),
                                macro_args = {
                                    let mut out = String::new();
                                    $crate::interpreting::builtins::impl_type! { @SpwnArgsGenA(out) $($args)* }
                                    out
                                },
                                macro_ret = {
                                    " " $(; format!(" -> @{} ", stringify!([<$ret_type:snake>])))?
                                }
                            )
                        ),*
                    ];

                    let out = indoc::formatdoc!(r#"
                            /*
                             * This file is automatically generated!
                             * Do not modify or your changes will be overwritten!
                            */
                            {impl_raw}
                            impl @{typ} {{
                                {consts}
                                {macros}
                            }}
                        "#,
                        impl_raw = { "" $(; $impl_raw)? },
                        //impl_doc = <[String]>::join(&[$($impl_doc .to_string()),*], "\n"),
                        typ = stringify!( [<$value_typ:snake>] ),
                        consts = consts.join("\n"),
                        macros = macros.join("\n"),
                    );

                    std::fs::write(path, &out).unwrap();
                }
            }

        }
    };

    // no more args
    (@ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident)) => {};

    // any kind of argument
    (@ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident) &mut $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsB[$ __ $idx, true]($args, $vm, $area) $ident $($t)* }
    };
    (@ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident) mut $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsB[$ __ $idx, true]($args, $vm, $area) $ident $($t)* }
    };
    (@ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident) & $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsB[$ __ $idx, false]($args, $vm, $area) $ident $($t)* }
    };
    (@ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident) $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsB[$ __ $idx, false]($args, $vm, $area) $ident $($t)* }
    };

    // spread arguments single type
    (
        @ArgsA[$idx:expr]($args:ident, $vm:ident, $area:ident)

        ...$var:ident : $typ:ident $(if $pattern:literal)? $(as $spwn_name:literal)?

        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            randsym::randsym! {
                let /?@v/ = $args[$idx].borrow();
                let $var = match &/?@v/.value {
                    $crate::interpreting::value::Value::Array(v) => itertools::Itertools::collect_vec(v.iter().map(|v| [< $typ Getter >]::<'_, false>::make_from(v))),
                    _ => panic!("scock"),
                };
            }
        }
        
        impl_type! { @ArgsA[$idx + 1]($args, $vm, $area) $($($t)*)? }
    };

    // tuple variant destructure argument
    (
        @ArgsB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident, $area:ident)
        
        $variant:ident ($field:ident) $(if $pattern:literal)? as $spwn_name:literal $( = $default:literal )?
        
        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            let $field = [< $variant Getter >]::<'_, $mut>::make_from(& $args[$idx]).0;
        }

        impl_type! { @ArgsA[$idx + 1]($args, $vm, $area) $($($t)*)? }
    };
    // struct variant destructure argument
    (
        @ArgsB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident, $area:ident)

        $variant:ident{ $( $field:ident $(: $bind:ident)? ),* $(,)? } $(if $pattern:literal)? as $spwn_name:literal $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            let getter = [< $variant Getter >]::<'_, $mut>::make_from(& $args[$idx]);
            
            $(
                let impl_type! {@FieldBind $field $($bind)?} = getter.$field;
            )*
        }

        impl_type! { @ArgsA[$idx + 1]($args, $vm, $area) $($($t)*)? }
    };
    // single type argument
    (
        @ArgsB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident, $area:ident)

        $var:ident $(: $typ:ident)? $(if $pattern:literal)? $(as $spwn_name:literal)? $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            let $var = {
                & $args[$idx]
                $(;
                    [< $typ Getter >]::<'_, $mut>::make_from(& $args[$idx])
                )?
            };
        }
        impl_type! { @ArgsA[$idx + 1]($args, $vm, $area) $($($t)*)? }
    };
    // multiple type argument
    (
        @ArgsB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident, $area:ident)
        
        $var:ident : $($typ:ident $(if $pattern:literal)?)|+ $(as $spwn_name:literal)? $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {
        randsym::randsym! {
            paste::paste! {
                enum Union<'a> {
                    $(
                        $typ( [< $typ Getter >]<'a> ),
                    )+
                }
                #[allow(unused)]
                // random symbol
                let /?@v/ = match &$args[$idx].borrow().value {
                    $(
                        $crate::interpreting::value::Value::$typ{..} => Union::$typ([< $typ Getter >]::<'_, $mut>::make_from(& $args[$idx])),
                    )+
                    _ => panic!("fgdf 4534 kXKKLKLDK")
                };
                macro_rules! $var {
                    (
                        $_(
                            $destr_variant:ident
                                $_( ( $destr_tuple:ident ) )?
                                $_( { $_( $destr_field:ident $_(: $destr_bind:ident)? ),* $_(,)? } )?
                            => $code:expr
                        ),*
        
                        $_(,)?
                    ) => {
                        match /?@v/ {
                            $_(
                                Union::$destr_variant(getter) => {
                                    $_(
                                        let $destr_tuple = getter.0;
                                    )?
                                    $_(
                                        $_(
                                            let impl_type! {@FieldBind $destr_field $_($destr_bind)?} = getter.$destr_field;
                                        )*
                                    )?
                                    $code
                                },
                            )*
                        }
                    };
                }
            }
        }

        impl_type! { @ArgsA[$idx + 1]($args, $vm, $area) $($($t)*)? }

    };

    (@FieldBind $field:ident) => {
        $field
    };
    (@FieldBind $field:ident $bind:ident) => {
        $bind
    };
    
    (@SpwnArgsGenA($out:ident)) => {};

    (@SpwnArgsGenA($out:ident) &mut $ident:ident $($t:tt)*) => {
        $out += "\n\t\t&mut ";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenB($out) $ident $($t)* }
    };
    (@SpwnArgsGenA($out:ident) mut $ident:ident $($t:tt)*) => {
        $out += "\n\t\tmut ";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenB($out) $ident $($t)* }
    };
    (@SpwnArgsGenA($out:ident) & $ident:ident $($t:tt)*) => {
        $out += "\n\t\t&";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenB($out) $ident $($t)* }
    };
    (@SpwnArgsGenA($out:ident) $ident:ident $($t:tt)*) => {
        $out += "\n\t\t";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenB($out) $ident $($t)* }
    };
    
    (
        @SpwnArgsGenA($out:ident)

        ...$var:ident : $typ:ident $(if $pattern:literal)? $(as $spwn_name:literal)?

        // rest
        $(, $($t:tt)*)?
    ) => {
        $out += "\n\t\t";
        paste::paste! {
            let arg_name = {
                stringify!($var)
                $(;
                    $spwn_name
                )?
            };

            $out += &format!("...{}: (@{}", arg_name, stringify!([< $typ:snake >]))
        }

        $(
            $out += &format!(" & ({})", $pattern);
        )?
        $out += ")[],";
        
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenA($out) $($($t)*)? }
    };

    (
        @SpwnArgsGenB($out:ident)
        
        $variant:ident $( ( $($t1:tt)* ) )? $( { $($t2:tt)* } )? $(if $pattern:literal)? as $spwn_name:literal $( = $default:literal )?
        
        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            $out += &format!("{}: @{}", $spwn_name, stringify!([< $variant:snake >]))

        }

        $(
            $out += &format!(" & ({})", $pattern);
        )?

        $(
            $out += &format!(" = {}", $default);
        )?
        $out += ",";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenA($out) $($($t)*)? }
    };
    (
        @SpwnArgsGenB($out:ident)

        $var:ident $(: $typ:ident)? $(if $pattern:literal)? $(as $spwn_name:literal)? $( = $default:literal )?
        
        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {
            let arg_name = {
                stringify!($var)
                $(;
                    $spwn_name
                )?
            };

            $out += arg_name;


            $out += &{
                ": _".to_string()
                $(;
                    format!(": @{}", stringify!([< $typ:snake >]))
                )?
            };
        }

        $(
            $out += &format!(" & ({})", $pattern);
        )?
        
        $(
            $out += &format!(" = {}", $default);
        )?
        $out += ",";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenA($out) $($($t)*)? }
    };
    (
        @SpwnArgsGenB($out:ident)

        $var:ident : $($typ:ident $(if $pattern:literal)?)|+ $(as $spwn_name:literal)? $( = $default:literal )?
        
        // rest
        $(, $($t:tt)*)?
    ) => {
        paste::paste! {

            let arg_name = {
                stringify!($var)
                $(;
                    $spwn_name
                )?
            };

            $out += arg_name;

            $out += ": ";
            {
                use itertools::Itertools;
                $out += &[$( format!("(@{}{})", stringify!([< $typ:snake >]), {
                    ""
                    $(;
                        format!(" & ({})", $pattern)
                    )?
                }) ),+].iter().join(" | ")
            }
        }
        $(
            $out += &format!(" = {}", $default);
        )?
        $out += ",";
        $crate::interpreting::builtins::impl_type! { @SpwnArgsGenA($out) $($($t)*)? }
    };

    

    // no more args
    (@ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident)) => {};

    // any kind of argument
    (@ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident) &mut $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsCheckCloneB[$ __ $idx, true]($args, $vm) $ident $($t)* }
    };
    (@ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident) mut $ident:ident $($t:tt)*) => {
        {
            let new = $crate::interpreting::vm::DeepClone::deep_clone_ref($vm, &$args[$idx]);
            $args[$idx] = new;
        }
        impl_type! { @ArgsCheckCloneB[$ __ $idx, true]($args, $vm) $ident $($t)* }
    };
    (@ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident) & $ident:ident $($t:tt)*) => {
        impl_type! { @ArgsCheckCloneB[$ __ $idx, false]($args, $vm) $ident $($t)* }
    };
    (@ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident) $ident:ident $($t:tt)*) => {
        {
            let new = $crate::interpreting::vm::DeepClone::deep_clone_ref($vm, &$args[$idx]);
            $args[$idx] = new;
        }
        impl_type! { @ArgsCheckCloneB[$ __ $idx, false]($args, $vm) $ident $($t)* }
    };

    // spread arguments single type
    (
        @ArgsCheckCloneA[$idx:expr]($args:ident, $vm:ident)

        ...$var:ident : $typ:ident $(if $pattern:literal)? $(as $spwn_name:literal)?

        // rest
        $(, $($t:tt)*)?
    ) => {
        impl_type! { @ArgsCheckCloneA[$idx + 1]($args, $vm) $($($t)*)? }
    };

    // tuple variant destructure argument
    (
        @ArgsCheckCloneB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident)
        
        $variant:ident ($field:ident) $(if $pattern:literal)? as $spwn_name:literal $( = $default:literal )?
        
        // rest
        $(, $($t:tt)*)?
    ) => {

        impl_type! { @ArgsCheckCloneA[$idx + 1]($args, $vm) $($($t)*)? }
    };
    // struct variant destructure argument
    (
        @ArgsCheckCloneB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident)

        $variant:ident{ $( $field:ident $(: $bind:ident)? ),* $(,)? } $(if $pattern:literal)? as $spwn_name:literal $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {

        impl_type! { @ArgsCheckCloneA[$idx + 1]($args, $vm) $($($t)*)? }
    };
    // single type argument
    (
        @ArgsCheckCloneB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident)

        $var:ident $(: $typ:ident)? $(if $pattern:literal)? $(as $spwn_name:literal)? $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {
        impl_type! { @ArgsCheckCloneA[$idx + 1]($args, $vm) $($($t)*)? }
    };
    // multiple type argument
    (
        @ArgsCheckCloneB[$_:tt __ $idx:expr, $mut:expr]($args:ident, $vm:ident)
        
        $var:ident : $($typ:ident $(if $pattern:literal)?)|+ $(as $spwn_name:literal)? $( = $default:literal )?

        // rest
        $(, $($t:tt)*)?
    ) => {
        impl_type! { @ArgsCheckCloneA[$idx + 1]($args, $vm) $($($t)*)? }

    };
}

pub use impl_type;

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
