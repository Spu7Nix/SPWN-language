use proc_macro::TokenStream;
use proc_macro2::Ident;
// def_type! {

//     /// foo bar
//     impl @error {
//         const TYPE_MISMATCH = Int(0);

//         fn poo(
//             &self / &mut self,

//             Thing(...)
//             r: &mut Thing
//             r: &Thing
//             r where Area(a) Key(k),

//             where Area(...) Key(...) Value(...)
//
//             Range(start, end, step) | String where Area(a) Key(k)
//r: &String | &Int

//             r: Range(start, end, step) | &String where Area<bfg> Key<gfjfkhg>,

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
use quote::quote;
// use syn::parse::Parse;
// use syn::punctuated::Punctuated;
// use syn::{Block, Expr, Path, Token, TypeReference, Variant};

// struct TypeMember {
//     name: Ident,
//     value: Expr,
// }

// struct Ref {
//     name: Ident,
//     is_mut: bool,
// }

// struct TypeMacroArgWhere {}

// struct MacroArg {
//     binder: Option<Ident>,
//     typ: ArgType,
// }

// enum ArgType {
//     Destructure(Punctuated<Variant, Token![|]>),
//     Ref(Punctuated<Ref, Token![|]>),
//     Any { name: Ident },
// }

// struct TypeMacro {
//     args: Vec<MacroArg>,
//     block: Block,
//     ret_typ: Option<Path>,
// }

// struct TypeImpl {
//     name: Ident,
//     members: Vec<TypeMember>,
//     functions: Vec<TypeMacro>,
// }

// impl Parse for TypeImpl {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let _: Token![impl] = input.parse()?;
//         let _: Token![@] = input.parse()?;

//         let name: Ident = input.parse()?;
//     }
// }

#[proc_macro]
pub fn def_type(input: TokenStream) -> TokenStream {
    quote! {}.into()
}
