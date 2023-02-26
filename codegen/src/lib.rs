#![allow(warnings)]

use inflector::Inflector;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{self, Parse, Parser, Peek};
use syn::punctuated::Punctuated;
use syn::{
    braced, parenthesized, parse_macro_input, token, Attribute, Block, Expr, ItemConst, Lit, Meta,
    Path, Token, Type, Variant,
};

macro_rules! syn_err {
    ($l:literal $(, $a:expr)*) => {
        syn_err!(proc_macro2::Span::call_site(); $l $(, $a)*)
    };
    ($s:expr; $l:literal $(, $a:expr)*) => {
        return Err(syn::Error::new($s, format!($l $(, $a)*)))
    };
}

#[derive(Debug)]
struct SpwnAttrs {
    docs: Vec<Lit>,
    raw: Vec<TokenStream>,
}

impl Parse for SpwnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        let mut docs = vec![];
        let mut raw = vec![];

        for attr in attrs {
            if attr.path == Path::parse.parse_str("doc").unwrap() {
                docs.push(match attr.parse_meta()? {
                    Meta::NameValue(nv) => nv.lit,
                    _ => syn_err!(r#"expected #[doc = "..."]"#),
                });
            } else if attr.path == Path::parse.parse_str("raw").unwrap() {
                raw.push(attr.tokens);
            }
        }

        Ok(Self { docs, raw })
    }
}

#[derive(Debug)]
struct TypeConstant {
    name: Ident,
    ty: Ident,
    value: Expr,
    attrs: SpwnAttrs,
}

impl Parse for TypeConstant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;

        input.parse::<Token![const]>()?;
        let name = input.parse()?;
        input.parse::<Token![:]>()?;

        let ty = input.parse()?;
        input.parse::<Token![=]>()?;

        let value = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self {
            name,
            ty,
            value,
            attrs,
        })
    }
}

#[derive(Debug)]
struct Ref<T: Parse> {
    name: T,
    is_ref: bool,
    is_mut: bool,
}

impl<T: Parse> Parse for Ref<T> {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let is_ref = input.parse::<Token![&]>().map_or(false, |_| true);
        let is_mut = input.parse::<Token![mut]>().map_or(false, |_| true);

        Ok(Self {
            name: input.parse()?,
            is_ref,
            is_mut,
        })
    }
}

#[derive(Debug)]
struct MacroArgWhere {
    area: Option<Ident>,
    key: Option<Ident>,
}

impl Parse for MacroArgWhere {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        mod kw {
            syn::custom_keyword!(Area);
            syn::custom_keyword!(Key);
        }

        input.parse::<Token![where]>()?;

        fn parse_bound<T: Parse, U: Parse>(input: parse::ParseStream) -> syn::Result<U> {
            input.parse::<T>()?;
            input.parse::<Token![:]>()?;
            let out = input.parse::<U>()?;

            if !input.peek(Token![,]) {
                return syn_err!(input.span(); "expected comma");
            }

            Ok(out)
        }

        let mut area: Option<Ident> = None;
        let mut key: Option<Ident> = None;

        loop {
            let lk = input.lookahead1();
            if lk.peek(kw::Area) && area.is_none() {
                area = Some(parse_bound::<kw::Area, _>(input)?)
            } else if lk.peek(kw::Key) && key.is_none() {
                area = Some(parse_bound::<kw::Key, _>(input)?)
            } else if area.is_none() && key.is_none() {
                return Err(lk.error());
            } else {
                break;
            }
            //  else if area.is_none() && key.is_none() {
            //     return Err(lk.error());
            // }

            if input.is_empty() {
                break;
            }
        }

        Ok(Self { area, key })
    }
}

#[derive(Debug)]
struct Destructure(pub Punctuated<Ident, Token![,]>);

impl Parse for Destructure {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        Ok(Self(Punctuated::parse_terminated(&content)?))
    }
}

#[derive(Debug)]
enum ArgType {
    Spread(Ident),
    Destructure {
        name: Path,
        fields: Destructure,
    },
    Ref {
        binder: Ident,
        tys: Punctuated<Ref<Ident>, Token![|]>,
    },
    Any(Ident),
}

impl Parse for ArgType {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        return if input.peek2(Token![:]) {
            let binder = input.parse()?;
            input.parse::<Token![:]>()?;
            Ok(Self::Ref {
                binder,
                tys: Punctuated::parse_separated_nonempty(input)?,
            })
        } else if input.peek2(Token![where]) {
            let binder = input.parse()?;
            Ok(Self::Any(binder))
        } else if input.peek(Token![...]) {
            input.parse::<Token![...]>()?;
            Ok(Self::Spread(input.parse()?))
        } else {
            Ok(Self::Destructure {
                name: input.parse()?,
                fields: input.parse()?,
            })
        };
    }
}

#[derive(Debug)]
struct MacroArg {
    ty: ArgType,
    cwhere: Option<MacroArgWhere>,
}

impl Parse for MacroArg {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;

        let cwhere = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { ty, cwhere })
    }
}

#[derive(Debug)]
struct TypeMacro {
    name: Ident,
    slf: Ref<Token![self]>,
    args: Punctuated<MacroArg, Token![,]>,
    ret_ty: Option<Path>,
    block: Block,
    attrs: SpwnAttrs,
}

impl Parse for TypeMacro {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;
        input.parse::<Token![fn]>()?;

        let name = input.parse()?;

        let content;
        parenthesized!(content in input);

        let slf = content.parse()?;
        content.parse::<Token![,]>();

        let args = Punctuated::parse_terminated(&content)?;

        let ret_ty = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            name,
            slf,
            args,
            ret_ty,
            block: input.parse()?,
            attrs,
        })
    }
}

#[derive(Debug)]
struct TypeImpl {
    name: Ident,
    constants: Vec<TypeConstant>,
    macros: Vec<TypeMacro>,
    attrs: SpwnAttrs,
}

impl Parse for TypeImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: SpwnAttrs = input.parse()?;

        input.parse::<Token![impl]>()?;
        input.parse::<Token![@]>()?;

        let name = input.parse()?;

        let content;
        braced!(content in input);

        let mut constants = vec![];
        let mut macros = vec![];

        loop {
            if content.is_empty() {
                break;
            }

            match TypeConstant::parse(&content) {
                Ok(c) => {
                    constants.push(c);
                    continue;
                }
                Err(e) => (),
            };

            match TypeMacro::parse(&content) {
                Ok(m) => {
                    macros.push(m);
                    continue;
                }
                Err(_) => syn_err!("expected macro or constant"),
            }
        }

        Ok(Self {
            attrs,
            name,
            constants,
            macros,
        })
    }
}

#[proc_macro]
pub fn def_type(input: TokenStream1) -> TokenStream1 {
    let ty_impl = parse_macro_input!(input as TypeImpl);

    let name = ty_impl.name;

    let builtin_ident = format_ident!("{}", &name.to_string().to_pascal_case());
    let core_gen_ident = format_ident!("gen_{}_core", &name);

    let impl_doc = ty_impl.attrs.docs;
    let impl_raw = ty_impl.attrs.raw;

    let consts = ty_impl.constants.iter().map(|c| {
        let raw = &c.attrs.raw;
        let docs = &c.attrs.docs;
        let name = &c.name;
        let ty = format_ident!("{}", &c.ty.to_string().to_camel_case());
        let val = &c.value;
        quote! {
            indoc::formatdoc!("
                    \t{const_raw}
                    \t#[doc(u{const_doc:?})]
                    \t{const_name}: @{const_type} = {const_val},
                ",
                const_raw = stringify!(#(#raw),*),
                const_doc = <[String]>::join(&[#(#docs .to_string()),*], "\n"),
                const_name = stringify!(#name),
                const_type = stringify!(#ty),
                const_val = stringify!(#val),
            )
        }
    });

    quote! {
        impl crate::vm::value::type_aliases::#builtin_ident {
            pub fn get_override_fn(self, name: &'static str) -> Option<crate::vm::value::BuiltinFn> {
                None
            }
            pub fn get_override_const(self, name: &'static str) -> Option<crate::compiling::bytecode::Constant> {
                None
            }
        }

        #[cfg(test)]
        mod #core_gen_ident {
            #[test]
            pub fn #core_gen_ident() {
                let path = std::path::PathBuf::from(format!("{}{}.spwn", crate::CORE_PATH, stringify!(#name)));
                let out = indoc::formatdoc!(r#"
                        {impl_raw}
                        #[doc(u{impl_doc:?})]
                        impl @{typ} {{
                            {consts}
                        }}
                    "#,
                    typ = stringify!(#name),
                    impl_raw = stringify!(#(#impl_raw),*),
                    impl_doc = <[String]>::join(&[#(#impl_doc .to_string()),*], "\n"),
                    consts = <[String]>::join(&[#(#consts .to_string()),*], ""),
                );

                std::fs::write(path, &out).unwrap();
            }
        }
    }
    .into()
}
