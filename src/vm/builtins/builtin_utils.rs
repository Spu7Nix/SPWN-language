use std::marker::PhantomData;
use std::sync::Mutex;

use crate::sources::CodeArea;
use crate::vm::interpreter::{RuntimeResult, ValueKey, Vm};
use crate::vm::value::{BuiltinFn, Value, ValueType};

pub trait Invoke<const N: usize, Args = ()> {
    fn invoke(&self, args: Vec<ValueKey>, vm: &mut Vm, area: CodeArea) -> RuntimeResult<Value>;
}

pub trait IntoArg<O> {
    fn into_arg(self, vm: &mut Vm) -> O;
}

pub trait GetMutRefArg<'a> {
    type Output;

    fn get_mut_ref_arg(key: ValueKey, vm: &'a mut Vm) -> Self::Output;
}
pub trait GetRefArg<'a> {
    type Output;

    fn get_ref_arg(key: ValueKey, vm: &'a Vm) -> Self::Output;
}

pub struct Or<T>(T);
pub struct Spread<T>(Vec<T>);

impl<T> std::ops::Deref for Spread<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct Ref<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> {
    key: ValueKey,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> Ref<'a, T> {
    pub fn get_mut_ref(&self, vm: &'a mut Vm) -> <T as GetMutRefArg<'a>>::Output {
        T::get_mut_ref_arg(self.key, vm)
    }

    pub fn get_ref(&self, vm: &'a Vm) -> <T as GetRefArg<'a>>::Output {
        T::get_ref_arg(self.key, vm)
    }
}

////////////////////

impl<'a, T: GetMutRefArg<'a> + GetRefArg<'a>> IntoArg<Ref<'a, T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, _: &mut Vm) -> Ref<'a, T> {
        Ref {
            key: self,
            _phantom: PhantomData,
        }
    }
}

impl<T> IntoArg<Spread<T>> for ValueKey
where
    ValueKey: IntoArg<T>,
{
    fn into_arg(self, vm: &mut Vm) -> Spread<T> {
        match &vm.memory[self].value {
            Value::Array(arr) => Spread(arr.clone().iter().map(|k| k.into_arg(vm)).collect()),
            _ => unreachable!(),
        }
    }
}

impl IntoArg<Value> for ValueKey {
    fn into_arg(self, vm: &mut Vm) -> Value {
        vm.memory[self].value.clone()
    }
}

impl IntoArg<ValueKey> for ValueKey {
    fn into_arg(self, _: &mut Vm) -> ValueKey {
        self
    }
}

impl IntoArg<CodeArea> for ValueKey {
    fn into_arg(self, vm: &mut Vm) -> CodeArea {
        vm.memory[self].area.clone()
    }
}

macro_rules! tuple_macro {
    (@gen $($ident:ident)*) => {

        impl<$($ident,)*> IntoArg<Or<($(Option<$ident>,)*)>> for ValueKey
        where $( ValueKey: IntoArg<$ident> ),*
        {
            fn into_arg(self, _vm: &mut Vm) -> Or<($(Option<$ident>,)*)> {
                todo!()
            }
        }

        impl<$($ident,)*> IntoArg<($($ident,)*)> for ValueKey
        where $( ValueKey: IntoArg<$ident> ),*
        {
            #[allow(clippy::unused_unit)]
            fn into_arg(self, vm: &mut Vm) -> ($($ident,)*) {
                (
                    $({
                        stringify!($ident);
                        self.into_arg(vm)
                    },)*
                )
            }
        }

        impl<Fun, $($ident,)*> Invoke<0, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)* &mut Vm, CodeArea) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _area: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* _vm, _area )
            }
        }

        impl<Fun, $($ident,)*> Invoke<1, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)* &mut Vm) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* _vm, )
            }
        }

        impl<Fun, $($ident,)*> Invoke<2, ($($ident,)*)> for Fun
        where
            $( ValueKey: IntoArg<$ident>, )*
            Fun: Fn($($ident,)*) -> RuntimeResult<Value>
        {
            #[allow(non_snake_case)]
            fn invoke(&self, _args: Vec<ValueKey>, _vm: &mut Vm, _: CodeArea) -> RuntimeResult<Value> {
                let mut _args = _args.into_iter();
                $(
                    let $ident = <ValueKey as IntoArg<$ident>>::into_arg(_args.next().unwrap(), _vm);
                )*
                (self)( $($ident,)* )
            }
        }
    };

    ($first:ident $( $name:ident )* ) => {
        tuple_macro!( @gen $first $( $name )* );

        tuple_macro!( $( $name )* );
    };

    () => {
        tuple_macro!(@gen);
    };
}

tuple_macro! { A B C D }

//     impl @error {
//         const TYPE_MISMATCH = Int(0);

//         < #[deprecated] >
//         fn poo(
//           #[self]
//             &self / &mut self,

//             Thing(...)
//             r: mut ref Thing
//             r: ref Thing
//             r where Area(a) Key(k),

//             where Area(...) Key(...) Value(...)
//
//             _: Range(start, end, step) | String where CodeArea(a) ValueKey(k)

//r: &String | &Int

//             r: &mut Range @ range_area,
//             r: &Range,

//             a: Range(start, end, step),

//             raw k,
//         ) {
//             if let Some(v) = r.is::<Range>() {
//                 //dfdfdf
//             }
//             if let Some(v) = r.is::<String>() {
//                 / fdfd fd f
//             }

//             match vm.memory.get_mut(key) {
//                 Range(ref mut start, ref mut end, ref mut step)
//             }

//             r.get_mut_ref()
//         }?
//     }
// }

#[rustfmt::skip]
macro_rules! builtin_impl {
    (
        $(#[doc = $impl_doc:literal])*
        // temporary until 1.0
        $(#[raw($($impl_raw:tt)*)])?
        impl @$builtin:ident {
            // $(
                
            //     $(#[doc = $adoc:literal])+
            //     // temporary until 1.0
            //     // $(#[raw($($doc_raw:tt)*)])?
                
            //     // $(
            //     //     const $const_name:ident : $const_ty:ty = $const_val:literal;
            //     // )?

            //     // $(
            //     //     fn $fn_name:ident (
            //     //         $(
            //     //             $(
            //     //                 $var:ident $(: $(
            //     //                     $(ref $ref_variant:ident)?
            //     //                     $(*$val_variant:ident)?
            //     //                 )|+)?
            //     //             )?
    
            //     //             $(
            //     //                 _: $variant:ident $(($($tok1:tt)*))? $({$($tok2:tt)*})?
            //     //             )?
    
            //     //             $(where $($extra:ident($bind:ident))+)?
    
            //     //             ,
            //     //         )*
            //     //     ) $(-> $ret_type:ty)? $code:block
            //     // )?
            // )*

            $(
                $(#[doc = $adoc:literal])+

                const $const_name:ident : $const_ty:ty = $const_val:literal;
            )*

            $(
                $(#[doc = $adoc2:literal])+

                fn
            )*
        }
    ) => {
        paste::paste! {
            impl $crate::vm::value::type_aliases::[<$builtin:camel>] {
                /// <img src="https://cdn.discordapp.com/attachments/909974406850281472/1077264823802417162/lara-hughes-blahaj-spin-compressed.gif" width=64><img src="https://cdn.discordapp.com/attachments/909974406850281472/1077264823802417162/lara-hughes-blahaj-spin-compressed.gif" width=64>
                pub fn get_override_fn(self, name: &'static str) -> Option<BuiltinFn> {
                    None
                    // match name {
                    //     $(
                    //         stringify!($fn_name) => {
                    //             fn inner(keys: Vec<ValueKey>, vm: &mut Vm, call_area: CodeArea) -> RuntimeResult<Value> {
                    //                 // $(
                                        
                    //                 // )*
                    //                 todo!()
                    //             }

                    //             Some($crate::vm::value::BuiltinFn(&inner))
                    //         },
                    //     )*
                    //     _ => None
                    // }
                }
                pub fn get_override_const(self, name: &'static str) -> Option<$crate::compiling::bytecode::Constant> {
                    None
                    // match name {
                    //     $(
                    //         stringify!($const_name) => Some($crate::compiling::bytecode::Constant::$const_ty($const_val)),
                    //     )*
                    //     _ => None,
                    // }
                }
            }
            
            #[test]
            pub fn [<$builtin _core_gen>]() {
                //let (slf, line, col) = (file!(), line!(), column!());
                let path = std::path::PathBuf::from(format!("{}{}.spwn", $crate::CORE_PATH, stringify!($builtin)));

                // let consts = &[
                //     $(
                //         indoc::formatdoc!(r#"
                //                 {const_raw}
                //                 #[doc(u{const_doc:?})]
                //                 {const_name}: @${const_type} = {const_val}
                //             "#,
                //             const_raw = stringify!($doc_raw)*),
                //             const_doc = &[$($doc),*].join("\n"),
                //             const_name = stringify!($const_name),
                //             const_type = stringify!($const_type),
                //             const_val = stringify!($const_val),
                //         )
                //     )*
                // ];

                let consts = &[
                    $(
                        indoc::formatdoc!(r#"
                                #[doc(u{const_doc:?})]
                            "#,
                            const_doc = $doc,
                        ),
                    )*
                ];

                let out = indoc::formatdoc!(r#"
                        {impl_raw}
                        #[doc(u{impl_doc:?})]
                        impl @{typ} {{
                            {consts}

                            #[doc(...)]
                            <>

                            #[doc(...)]
                            <>
                        }}
                    "#, 
                    impl_raw = stringify!($($($impl_raw)*)?),
                    impl_doc = &[$($impl_doc),*].join("\n"),
                    typ = stringify!($builtin),
                    consts = consts.join("\n"),
                );

                std::fs::write(path, &out).unwrap();
            }
        }
    };
}

// builtin_impl! {
//     /// aaaaaaaa
//     /// bbbbbbbb
//     /// cccccccc
//     /// dddddddd
//     impl @string {
//         /// a
//         /// b
//         /**

//         bunky

//         */
//         fn
//         // #[raw( #[deprecated] )]
//         // const A: Int = 0;

//         // fn bunk(
//         //     thing: ref Range where ValueKey(a),
//         //     _: Range(start, end, step),
//         //     v: *Range | *Thing,
//         //     farter: ref Int,
//         // ) {
//         //     // if let Some(ARange(....)) =
//         //     // farter.get_mut_ref() MutAInt
//         // }
//     }
// }

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
    let arg1 = match vm.memory[v[0]].value {
        Value::TriggerFunction {
            group, dfsfsdf
        } => Arg1 { group, dfsfsfsfsgr},
        ...
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
//             String(s) as self,
//             arg1: Int, Int
//             arg2: &Int,
//             Range(start, end, step) as arg2 where Key(b_k),
//             arg4: &String | AFloat,
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
