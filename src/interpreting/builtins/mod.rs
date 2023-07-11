#[rustfmt::skip]
macro_rules! impl_type {
    (
        $(#[doc = $impl_doc:expr])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl $typ:ident {
            $(
                $(#[doc = $const_doc:expr])*
                // temporary until 1.0
                $(#[raw($($const_raw:tt)*)])?
                fn $func_name:ident($(
                    $spwn_var:literal:

                    $(
                        $bind:ident
                        $( ( $($tuple_var:ident),* $(,)? ) )?
                        $( { $($struct_var:ident),* $(,)? } )?
                    )|*

                )*) $(-> $ret_type:ident)? {
                    $($code:tt)*
                }
            )*
        }
    ) => {

        

    };
}

impl_type! {
    impl Array {

        fn join("self": String(s)) {

        }


    }
}

enum ValueType {
    Int,
    Float,
    String,
    Array,

    A(bool),
}

impl ValueType {
    pub fn get_override_fn(&self) {
        match self {
            ValueType::Int => <ValueType as Foo<
                { unsafe { std::mem::transmute::<_, u8>(ValueType::Int) } },
            >>::builtin_fns(self),
            ValueType::Float => <ValueType as Foo<
                { unsafe { std::mem::transmute::<_, u8>(ValueType::Float) } },
            >>::builtin_fns(self),
            ValueType::String => <ValueType as Foo<
                { unsafe { std::mem::transmute::<_, u8>(ValueType::String) } },
            >>::builtin_fns(self),
            ValueType::Array => <ValueType as Foo<
                { unsafe { std::mem::transmute::<_, u8>(ValueType::Array) } },
            >>::builtin_fns(self),
            _ => todo!(),
        }
    }
}

trait Foo<const A: u8> {
    fn builtin_fns(&self);
}

impl Foo<{ unsafe { std::mem::transmute::<_, u8>(ValueType::Int) } }> for ValueType {
    fn builtin_fns(&self) {
        //
    }
}

impl Foo<{ unsafe { std::mem::transmute::<_, u8>(ValueType::Float) } }> for ValueType {
    fn builtin_fns(&self) {
        //
    }
}
