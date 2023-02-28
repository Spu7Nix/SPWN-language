/*
fn poo(v: Vec<ValueKey>, vm: &mut Vm, area: CodeArea) -> RuntimeResult<Value> {
    mod arg1 {
        pub struct Arg1 {
            group: Id,
            sfddsfsf: Id
        };
    }
    use arg1::Arg1;

    mod arg2 {
        pub struct Arg2(i64);
    }
    use arg2::Arg2;

    mod arg4 {
        pub struct StringGetter(ValueKey);

        pub struct StringRef<'a>(&'a String);
        pub struct RangeRef<'a>(&'a (i64,));
        pub struct StringMutRef<'a>(&'a mut String);

        impl StringGetter {
            pub fn get_ref(&self, vm: &Vm) -> StringRef<'_> {
                match &vm.memory[self.0].value {
                    Value::String(s) => StringRef(s)
                    _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                }
            }
            pub fn get_mut_ref(&self, vm: &mut Vm) -> StringMutRef<'_> {
                match &vm.memory[self.0].value {
                    Value::String(s) => StringMutRef(s)
                    _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                }
            }
        }

        pub enum Arg4 {
            String(StringGetter),
            Float(f64),
            TriggerFunction {
                a: id,
                b: id
            }
        }
    }
    use arg4::Arg4;

    let Value::String(s) = vm.memory[v[0]].value.clone() else {
        unreachable!();
    };
    let arg1 = match &vm.memory[v[0]].value {
        Value::TriggerFunction {
            group, dfsfsdf
        } => Arg1 { group, dfsfsfsfsgr},
        ...
    }

    mod cockle {
        pub enum CockleElem {
            Int(i64),
            Float(f64),
        }

        pub type Cockle = Vec<CockleElem>;
    }

    let cockle = match vm.memory[v[0]].value {
        Value::Array(v) => {
            let elems = v.iter().map(||)

            let mut elems = vec![];
            for k in v {
                elems.push(match &vm.memory[k].value {

                })
            }
        },
        ...
    }



    enum Piss {
        Int(i64),
        String(StringGetter),
    }

    let arg = match vm.memory[v[0]].value {
        Value::String{..} => Piss::String(StringGetter(v[0])),
        _ => match vm.memory[v[0]].value.clone() {
            Value::Int(a) => Piss::Int(a)
        }
    }


}

match arg4.get() {
    AFloat(f) =>
}

*/

// spwn_codegen::def_type! {
//     /// aaa
//     #[raw( #[deprecated] )]
//     impl @string {
//         /// bbb
//         const A = Range(0, 0, 0);

//         fn poo(
//             //String(s) as self = r#"obj { HSV: "aaa",  }"#,
//             ...cockle: Int | &String
//             //arg1: Int | Int = 10,
//             //arg2: &Int,
//             //Range(start, end, step) as arg2 where Key(b_k),
//             arg4: &String | Float,
//         ) {
//             // block
//         }

//         // fn poo() {}

//         // fn poo(&self) {}

//         // /// ccc
//         // fn poo(&self) -> Test {}

//         // fn poo(
//         //     &self,
//         //     Thing1 as r,
//         //     Thing2 { a, b } as r,
//         //     Thing3(a, b) as r where Key(k),
//         //     a: A | B,
//         //     b: &C,
//         //     c: &D,
//         //     d: &E | &F |, // enum D { E(ERef), F(FRef) } .get_ref
//         //     ...e,
//         //     f where Key(k),
//         //     g where Area(a) Key(k),
//         // ) -> Test {}
//     }
// }

// #[rustfmt::skip]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:literal])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl $impl_var:ident {
            Constants:
            $(
                $(#[doc = $const_doc:literal])*
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                const $const:ident = $c_name:ident
                                        $( ( $( $c_val:expr ),* ) )?
                                        $( { $( $c_n:ident: $c_val_s:expr ,)* } )?;
            )*

            Functions:
            $(
                $(#[doc = $fn_doc:literal])*
                // temporary until 1.0
                $(#[raw($($fn_raw:tt)*)])?
                fn $fn_name:ident($(
                    $(

                        $arg_name:ident
                            $(:
                                $(
                                    $(&$ref_ty:ident)? $($deref_ty:ident)?
                                )|+
                            )?
                            $(
                                $(
                                    ( $( $v_val:ident ),* )
                                )?
                                $(
                                    { $( $v_n:ident $(: $v_val_s:ident)? ,)* }
                                )?
                                as $binder:ident
                            )?

                        $(
                            = $default:literal
                        )?
                    )?

                    $(...$spread_arg:ident)?

                    $(
                        where $($extra:ident($extra_bind:ident))+
                    )?

                    ,
                )*) $(-> $ret_type:ident)? $b:block
            )*
        }
    ) => {
        impl crate::vm::value::type_aliases::$impl_var {
            pub fn get_override_fn(self, name: &'static str) -> Option<crate::vm::value::BuiltinFn> {
                $(
                    #[allow(unused_assignments)]
                    fn $fn_name(
                        v: Vec<crate::vm::interpreter::ValueKey>,
                        vm: &mut crate::vm::interpreter::Vm,
                        area: crate::sources::CodeArea
                    ) -> crate::vm::interpreter::RuntimeResult<crate::vm::value::Value> {

                        let mut arg_idx = 0usize;

                        $(
                            $(
                                paste::paste! {
                                    $(
                                        mod $arg_name {
                                            use crate::vm::value::gen_wrapper;

                                            $(
                                                $(
                                                    impl_type! { @make_getter $ref_ty }
                                                )?
                                            )+

                                            impl_type! {
                                                @gen_wrapper [<$arg_name:camel>] $( $(&$ref_ty)? $($deref_ty)? )|+
                                            }
                                        }

                                        #[allow(clippy::let_unit_value)]
                                        let $arg_name = match vm.memory[v[arg_idx]].value {
                                            $(
                                                $(
                                                    crate::vm::value::Value::$ref_ty{..} => $arg_name::[<$arg_name:camel>]::$ref_ty(
                                                        $arg_name::[<$ref_ty Getter>](v[arg_idx])
                                                    ),
                                                )?
                                            )+
                                            _ => {
                                                use eager::*;

                                                use crate::vm::value::destructure_names;
                                                use crate::vm::value::gen_wrapper;
                                                eager! {
                                                    match vm.memory[v[arg_idx]].value.clone() {
                                                        $(
                                                            $(
                                                                $crate::vm::value::Value::$deref_ty gen_wrapper!{ @destruct $deref_ty } => $arg_name::[<$arg_name:camel>]::$deref_ty gen_wrapper!{@destruct $deref_ty},
                                                            )?
                                                        )+
                                                        _ => lazy! { unreachable!() }
                                                    }
                                                }
                                            },
                                        };

                                    )?
                                    $(
                                        let crate::vm::value::Value::$arg_name
                                            $(
                                                ( $( $v_val ),* )
                                            )?
                                            $(
                                                { $( $v_n $(: $v_val_s)? ,)* }
                                            )?
                                        = vm.memory[v[arg_idx]].value.clone() else {
                                            unreachable!();
                                        };
                                    )?
                                }
                            )?

                            arg_idx += 1;
                        )*

                        todo!()
                    }
                )*

                match name {
                    $(
                        stringify!($fn_name) => Some(crate::vm::value::BuiltinFn(&$fn_name)),
                    )*
                    _ => None
                }
            }
            pub fn get_override_const(self, name: &'static str) -> Option<crate::compiling::bytecode::Constant> {
                None
            }
        }

        paste::paste! {
            #[cfg(test)]
            mod [<$impl_var:snake _core_gen>] {
                #[test]
                pub fn [<$impl_var:snake _core_gen>]() {
                    let path = std::path::PathBuf::from(format!("{}{}.spwn", crate::CORE_PATH, stringify!( [<$impl_var:snake>] )));

                    paste::paste! {
                        let consts: &[String] = &[
                            $(
                                indoc::formatdoc!("\t{const_raw}
                                    \t#[doc(u{const_doc:?})]
                                    \t{const_name}: @{const_type} = {const_val:?},",
                                    const_raw = stringify!($($const_raw)*),
                                    const_doc = <[String]>::join(&[$($const_doc)*], "\n"),
                                    const_name = stringify!($const),
                                    const_type = stringify!([<$c_name:snake>]),
                                    const_val = crate::compiling::bytecode::Constant::
                                        $c_name
                                            $( ( $( $c_val ),* ) )?
                                            $( { $( $c_n : $c_val_s ,)* } )?,
                                ),
                            )*
                        ];

                        let macros: &[String] = &[
                            $(
                                indoc::formatdoc!("\t{macro_raw}
                                    \t#[doc(u{macro_doc:?})]
                                    \t{macro_name}: ({macro_args}){macro_ret}{{
                                        \t/* compiler built-in */
                                    \t}},",
                                    macro_raw = stringify!($($fn_raw)*),
                                    macro_doc = <[&'static str]>::join(&[$($fn_doc),*], "\n"),
                                    macro_name = stringify!($fn_name),
                                    macro_args = <[String]>::join(&[
                                        $(
                                            $(
                                                $(
                                                    format!("{}: @{}",
                                                        stringify!($binder),
                                                        stringify!([<$arg_name:snake>]),
                                                    ),
                                                )?
                                                $(
                                                    format!("{}: @{}",
                                                        stringify!($arg_name),
                                                        <[&'static str]>::join(&[
                                                            $(
                                                                $(
                                                                    stringify!([<$ref_ty:snake>]),
                                                                )?
                                                                $(
                                                                    stringify!([<$deref_ty:snake>]),
                                                                )?
                                                            )+
                                                        ], " | @")
                                                    ),
                                                )?
                                            )?
                                        )*
                                    ], ", "),
                                    macro_ret = " -> @todo ",
                                )
                            )*
                        ];
                    }

                    let out = indoc::formatdoc!(r#"
                            /* 
                             * This file is automatically generated!
                             * Do not modify or your changes will be overwritten!  
                            */
                            {impl_raw}
                            #[doc(u{impl_doc:?})]
                            impl @{typ} {{{consts}
                            {macros}
                            }}
                        "#,
                        impl_raw = stringify!($($impl_raw),*),
                        impl_doc = <[String]>::join(&[$($impl_doc .to_string()),*], "\n"),
                        typ = stringify!( [<$impl_var:snake>] ),
                        consts = consts.join(""),
                        macros = macros.join(""),
                    );

                    std::fs::write(path, &out).unwrap();
                }
            }

        }
    };

    (@gen_wrapper $name:ident $(&$ref_ty:ident)? $($deref_ty:ident)?) => {
        $(
            gen_wrapper! {
                pub struct $name: *$deref_ty
            }
        )?
        $(
            paste::paste! {
                pub type $name = [<$ref_ty Getter>];
            }
        )?
    };
    (@gen_wrapper $name:ident $( $(&$ref_ty:ident)? $($deref_ty:ident)? )|+) => {
        paste::paste! {
            gen_wrapper! {
                pub enum $name: $( $($deref_ty |)? )+; $( $( $ref_ty( [<$ref_ty Getter>] ) ,)? )+
            }
        }
    };

    (@make_getter $ref_ty:ident) => {
        paste::paste! {
            pub struct [<$ref_ty Getter>](pub crate::vm::interpreter::ValueKey);

            gen_wrapper! {
                pub struct [<$ref_ty Ref>]: & $ref_ty
            }
            gen_wrapper! {
                pub struct [<$ref_ty MutRef>]: mut & $ref_ty
            }

            impl [<$ref_ty Getter>] {
                pub fn get_ref(&self, vm: &crate::vm::interpreter::Vm) -> [<$ref_ty Ref>]<'_> {
                    todo!()
                    // match &vm.memory[self.0].value {
                    //     Value::String(s) => StringRef(s)
                    //     _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                    // }
                }
                pub fn get_mut_ref(&self, vm: &mut crate::vm::interpreter::Vm) -> [<$ref_ty MutRef>]<'_> {
                    todo!()
                    // match &vm.memory[self.0].value {
                    //     Value::String(s) => StringMutRef(s)
                    //     _ => panic!("valuekey does not point to value of correct type !!!!!!!!")
                    // }
                }
            }
        }
    };
}

impl_type! {
    impl String {
        Constants:
        const POO = Int(10);
        const FOO = String("aaaa".into());

        Functions:
        fn poo(
            // String(s) as self = r#"bunkledo"#,
            arg1: Int | &Range = 10,
            // arg2: Int,
            // Range(start, end, step) as arg2 where Key(b_k),
            // arg4: &String | Float,
        ) -> Range {
            // block
        }
    }

}

struct RangeGetter;

use crate::vm::value::gen_wrapper;
gen_wrapper! {
  pub enum Arg1:Int| ;
  Range(RangeGetter),
}
