use super::vm::ValueRef;
use crate::gd::ids::Id;

#[rustfmt::skip]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:expr])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl $value_typ:ident {
            $(
                $(#[doc = $const_doc:expr])*
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                fn $func_name:ident($(
                    $(
                        ...$var_spread:ident :
                    )?

                    $(
                        $(
                            &$ident_ref:ident
                        )?
                        $(
                            $ident:ident
                        )?

                        $(
                            ( $( $tuple_field:ident $(,)? ),* )
                        )?
                        $(
                            { $(
                                $struct_field:ident $(:$struct_bind:ident)? $(,)?
                            ),* }
                        )?
                        $(
                            : $($var_typ:ident)|+
                        )?
                        as $spwn_name:literal
                    )?

                    $(where ( $($pat:tt)* ) )?

                    $(
                        = { $($default:tt)* }
                    )?
                    
                    $(,)?


                ),*) $(-> $ret_type:ident)? {
                    $($code:tt)*
                }
            )*
        }
    ) => {
        impl $crate::interpreting::value::type_aliases::$value_typ {
            pub fn get_override_fn(self, name: &str) -> Option<$crate::interpreting::value::BuiltinFn> {
                $(
                    fn $func_name(
                        args: Vec<$crate::interpreting::vm::ValueRef>,
                        vm: &mut $crate::Vm,
                        area: $crate::sources::CodeArea,
                    ) -> $crate::interpreting::vm::RuntimeResult<$crate::interpreting::value::Value> {
                        todo!()
                    }
                )*
                todo!()
            }
        }


    };
}

impl_type! {
    impl Array {
        fn push(String(s) as "drfgdf") {


            sex.value_area()

            slf.group.borrow()

            // match gug.borrow().value {
            //     String::
            //     _ => unreachable!(),
            // }
        }

    }
}
