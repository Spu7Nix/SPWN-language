#[rustfmt::skip]
#[macro_export]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:expr])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl $impl_var:ident {
            Constants:
            $(
                $(#[doc = $const_doc:expr])*
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                const $const:ident = $c_name:ident
                                        $( ( $( $c_val:expr ),* $(,)? ) )?
                                        $( { $( $c_n:ident: $c_val_s:expr ),* $(,)? } )?;
            )*

            Functions($vm:ident, $call_area:ident):
            $(
                $(#[doc = $fn_doc:expr])*
                // temporary until 1.0
                $(#[raw($($fn_raw:tt)*)])?
                fn $fn_name:ident($(

                    $name:ident
                        $(
                            $(
                                ( $( $v_val:ident ),* $(,)? )
                            )?
                            $(
                                { $( $v_n:ident $(: $v_val_s:ident)? ),* $(,)? }
                            )?
                            as $binder:ident
                        )?
                        $(
                            ...:
                            $(
                                $($spread_deref_ty:ident)? $(&$spread_ref_ty:ident)?
                            )|+
                        )?
                        $(
                            :
                            $(
                                $($deref_ty:ident)? $(&$ref_ty:ident)?
                            )|+
                        )?

                    $(if ( $($pat:tt)* ) )?
                        
                    $(
                        = { $($default:tt)* }
                    )?
                
                    $(
                        where $($extra:ident($extra_bind:ident))+
                    )?

                ),* $(,)? ) $(-> $ret_type:ident)? {$($b:tt)*}
            )*
        }
    ) => {
        impl $crate::vm::value::type_aliases::$impl_var {
            pub fn get_override_fn(self, name: &str) -> Option<$crate::vm::value::BuiltinFn> {
                $(
                    #[allow(unused_assignments, unused_variables, unused_imports)]
                    $(#[doc = $fn_doc])*
                    fn $fn_name(
                        __args: Vec<$crate::vm::interpreter::ValueKey>,
                        $vm: &mut $crate::vm::interpreter::Vm,
                        $call_area: $crate::sources::CodeArea
                    ) -> $crate::vm::interpreter::RuntimeResult<$crate::vm::value::Value> {
                        use $crate::vm::value::value_structs::*;

                        let mut __arg_idx = 0usize;

                        $(
                            
                            $(
                                impl_type! { @union ($name, $vm, __args, __arg_idx) $( $($deref_ty)? $(&$ref_ty)? )|+ }
                            )?
                            $(
                                paste::paste! {
                                    let $crate::vm::value::Value::$name
                                        $(
                                            ( $( $v_val ),* )
                                        )?
                                        $(
                                            { $( $v_n $(: $v_val_s)? ,)* }
                                        )?
                                    = $vm.memory[__args[__arg_idx]].value.clone() else {
                                        unreachable!()
                                    };
                                }
                            )?
                            $(
                                impl_type! {@... ($name, $vm, __args, __arg_idx) $( $($spread_deref_ty)? $(&$spread_ref_ty)? )|+ }
                            )?
                            $(
                                $(
                                    impl_type! {@extra $extra ($extra_bind, $vm, __args, __arg_idx)}
                                )+
                            )?
                            __arg_idx += 1;
                        )*

                        Ok({ $($b)* })
                    }
                )*

                match name {
                    $(
                        stringify!($fn_name) => Some($crate::vm::value::BuiltinFn(&$fn_name)),
                    )*
                    _ => None
                }
            }
        }

        paste::paste! {
            #[cfg(test)]
            mod [<$impl_var:snake _core_gen>] {
                #[test]
                pub fn [<$impl_var:snake _core_gen>]() {
                    let path = std::path::PathBuf::from(format!("{}{}.spwn", $crate::CORE_PATH, stringify!( [<$impl_var:snake>] )));

                    //#[doc(u{const_doc:?})]
                    let consts: &[String] = &[
                        $(
                            indoc::formatdoc!("\t{const_raw}
                                \t
                                \t{const_name}: {const_val:?},",
                                const_raw = stringify!($($const_raw)*),
                                //const_doc = <[String]>::join(&[$($const_doc)*], "\n"),
                                const_name = stringify!($const),
                                const_val = $crate::compiling::bytecode::Constant::
                                    $c_name
                                        $( ( $( $c_val ),* ) )?
                                        $( { $( $c_n : $c_val_s ,)* } )?,
                            ),
                        )*
                    ];

                    //#[doc(u{macro_doc:?})]
                    let macros: &[String] = &[
                        $(
                            indoc::formatdoc!("\t{macro_raw}
                                \t{macro_name}:
                                \t\t
                                \t\t({macro_args}){macro_ret}{{
                                    \t\t/* compiler built-in */
                                \t\t}},",
                                macro_raw = stringify!($($fn_raw)*),
                                //macro_doc = <[&'static str]>::join(&[$($fn_doc),*], "\n"),
                                macro_name = stringify!($fn_name),
                                macro_args = <[String]>::join(&[


                                    $(
                                        format!("{}{}",
                                            format!("{}{}",
                                                {
                                                    #[allow(unused_variables)]
                                                    fn rename_self(s: &str) -> &str {
                                                        if s == "slf" {
                                                            "self"
                                                        } else {
                                                            s
                                                        }
                                                    }
                                                    $(
                                                        format!("{}: @{}", 
                                                            stringify!($binder),
                                                            rename_self(&stringify!([<$name:snake>])),
                                                        )
                                                    )?
                                                    $(
                                                        format!("...{}{}",
                                                            rename_self(&stringify!([<$name:snake>])),
                                                            {
                                                                let types: &[String] = &[
                                                                    $(
                                                                        {
                                                                            let name = $(stringify!([<$spread_deref_ty:snake>]))? $(stringify!([<$spread_ref_ty:snake>]))?;
                                                                            if ["value", "value_key"].contains(&name) {
                                                                                "".into()
                                                                            } else {
                                                                                format!("@{name}")
                                                                            }
                                                                        },
                                                                    )*
                                                                ];
                                                                let types = types.iter().filter(|n| n.len() != 0).cloned().collect::<Vec<_>>();
                                                                if types.is_empty() {
                                                                    "".into()
                                                                } else {
                                                                    format!(": {}", types.join(" | "))
                                                                }
                                                            }
                                                        )
                                                    )?
                                        
                                                    $(
                                                        format!("{}{}",
                                                            rename_self(&stringify!([<$name:snake>])),
                                                            {
                                                                let types: &[String] = &[
                                                                    $(
                                                                        {
                                                                            let name = $(stringify!([<$deref_ty:snake>]))? $(stringify!([<$ref_ty:snake>]))?;
                                                                            if ["value", "value_key"].contains(&name) {
                                                                                "".into()
                                                                            } else {
                                                                                format!("@{name}")
                                                                            }
                                                                        },
                                                                    )*
                                                                ];
                                                                let types = types.iter().filter(|n| n.len() != 0).cloned().collect::<Vec<_>>();
                                                                if types.is_empty() {
                                                                    "".into()
                                                                } else {
                                                                    format!(": {}", types.join(" | "))
                                                                }
                                                            }
                                                        )
                                                    )?
                                                },

                                                {
                                                    "" $(; format!(" & {}", <[&'static str]>::join(&[$( stringify!($pat) ),*], "")))?
                                                }
                                            ),
                                            {
                                                "" $( ; format!(" = {}", stringify!( $($default)* )) )?
                                            }
                                        ),
                                    )*
                                ], ", "), 
                                macro_ret = {
                                    " " $(; format!(" -> @{} ", stringify!([<$ret_type:snake>])))?
                                },
                            ),
                        )*
                    ];

                    //#[doc(u{impl_doc:?})]
                    let out = indoc::formatdoc!(r#"
                            /*
                             * This file is automatically generated!
                             * Do not modify or your changes will be overwritten!
                            */
                            {impl_raw}
                            
                            impl @{typ} {{{consts}
                            {macros}
                            }}
                        "#,
                        impl_raw = stringify!($($impl_raw),*),
                        //impl_doc = <[String]>::join(&[$($impl_doc .to_string()),*], "\n"),
                        typ = stringify!( [<$impl_var:snake>] ),
                        consts = consts.join(""),
                        macros = macros.join(""),
                    );

                    std::fs::write(path, &out).unwrap();
                }
            }

        }
    };

    (@union [type] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $($dederef_ty:ident)? $(&$deref_ty:ident)?) => {};
    (@union [type] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $( $($dederef_ty:ident)? $(&$deref_ty:ident)? )|+) => {
        paste::paste! {
            enum [<$name:camel Value>] {
                $(
                    $(
                        [<$dederef_ty>] ( [<$dederef_ty Deref>] ),
                    )?
                    $(
                        [<$deref_ty>] ( [<$deref_ty Getter>] ),
                    )?
                )+
            }
        }
    };


    (@union [let] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) Value) => {
        let $name = $vm.memory[$args[$arg_index]].value.clone();
    };
    (@union [let] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) ValueKey) => {
        let $name = $args[$arg_index];
    };
    (@union [let] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $($dederef_ty:ident)? $(&$deref_ty:ident)?) => {
        paste::paste! {
            $(
                let $name: [<$dederef_ty Deref>] = $vm.memory[$args[$arg_index]].value.clone().into();
            )?
            $(
                let $name = [<$deref_ty Getter>] ($args[$arg_index]);
            )?
        }
    };
    (@union [let] ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $( $($dederef_ty:ident)? $(&$deref_ty:ident)? )|+) => {
        paste::paste! {
            let $name = match &$vm.memory[$args[$arg_index]].value {
                $(
                    $(
                        v @ $crate::vm::value::Value::$dederef_ty {..} => [<$name:camel Value>]::$dederef_ty(v.clone().into()),
                    )?
                    $(
                        $crate::vm::value::Value::$deref_ty {..} => [<$name:camel>]::$deref_ty([<$deref_ty Getter>] ($args[$arg_index])),
                    )?
                )+
                _ => unreachable!(),
            };
        }
    };
    

    
    (@union ($name:ident, $vm:ident, $args:ident, $arg_index:ident) Value) => {
        impl_type! {@union [let] ($name, $vm, $args, $arg_index) Value }
    };
    (@union ($name:ident, $vm:ident, $args:ident, $arg_index:ident) ValueKey) => {
        impl_type! {@union [let] ($name, $vm, $args, $arg_index) ValueKey }
    };
    (@union ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $($dederef_ty:ident)? $(&$deref_ty:ident)?) => {
        impl_type! {@union [let] ($name, $vm, $args, $arg_index) $($dederef_ty)? $(&$deref_ty)? }
    };
    
    (@union ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $( $($dederef_ty:ident)? $(&$deref_ty:ident)? )|+) => {
        impl_type! { @union [type] ($name, $vm, $args, $arg_index) $( $($dederef_ty)? $(&$deref_ty)? )|+ }
        impl_type! { @union [let] ($name, $vm, $args, $arg_index) $( $($dederef_ty)? $(&$deref_ty)? )|+ }
    };
    
    (@... ($name:ident, $vm:ident, $args:ident, $arg_index:ident) Value) => {
        let $name = match &$vm.memory[$args[$arg_index]].value {
            $crate::vm::value::Value::Array(v) => {
                v.iter().map(|k| $vm.memory[*k].value.clone()).collect::<Vec<_>>()
            }
            _ => unreachable!(),
        };
    };
    (@... ($name:ident, $vm:ident, $args:ident, $arg_index:ident) ValueKey) => {
        let $name = match &$vm.memory[$args[$arg_index]].value {
            $crate::vm::value::Value::Array(v) => {
                v.clone()
            }
            _ => unreachable!(),
        };
    };
    (@... ($name:ident, $vm:ident, $args:ident, $arg_index:ident) $( $($dederef_ty:ident)? $(&$deref_ty:ident)? )|+) => {
        impl_type! { @union [type] ($name, $vm, $args, $arg_index) $( $($dederef_ty)? $(&$deref_ty)? )|+ }

        let $name = match &$vm.memory[$args[$arg_index]].value {
            $crate::vm::value::Value::Array(v) => {
                v.iter().map(|k| {
                    impl_type! { @union [let] ($name, $vm, $args, $arg_index) $( $($dederef_ty)? $(&$deref_ty)? )|+ }
                    $name
                }).collect::<Vec<_>>()
            }
            _ => unreachable!(),
        };
    };

    (@extra Area ($name:ident, $vm:ident, $args:ident, $arg_index:ident) ) => {
        let $name = $vm.memory[$args[$arg_index]].area.clone();
    };
}

pub use impl_type;
