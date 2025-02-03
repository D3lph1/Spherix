// #![feature(proc_macro_quote)]

use proc_macro::TokenStream;

use quote::quote;
use regex::Regex;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Token, Type};

mod config;

struct TypeDebug {
    name: Expr,
    ty: Type,
}

impl Parse for TypeDebug {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: Type = input.parse()?;

        Ok(TypeDebug {
            name,
            ty,
        })
    }
}

fn dump(last_str: String, name: Expr) -> proc_macro2::TokenStream {
    match last_str.as_str() {
        "String" => {
            quote! {
                format!("{:?}", owo_colors::OwoColorize::green(&match #name.char_indices().nth(16) {
                    None => #name.to_owned(),
                    Some((idx, _)) => #name[..16].to_owned() + & String::from("..."),
                }))
            }
        }
        "VarInt" => {
            quote! {
                format!(
                    "VarInt({}{}",
                    owo_colors::OwoColorize::blue(&format!("{}", #name)),
                    owo_colors::OwoColorize::bright_black(&")")
                )
            }
        }
        "u16" | "i16" | "UnsignedShort" | "Short"
        | "u32" | "i32" | "UnsignedInt" | "Int"
        | "u64" | "i64" | "UnsignedLong" | "Long"
        | "Byte" | "bool" => {
            quote!(
                format!(
                    "{}",
                    owo_colors::OwoColorize::blue(&#name)
                )
            )
        }
        "f32" | "f64" | "Float" | "Double" => {
            quote!(
                format!(
                    "{:.2}",
                    owo_colors::OwoColorize::blue(&#name)
                )
            )
        }
        "Uuid" => {
            quote!(
                        format!(
                            "{}",
                            owo_colors::OwoColorize::yellow(&#name)
                        )
                    )
        }
        "Box < [u8] >" | "Box < [Byte] >" | "Box < [UnsignedByte] >" | "ByteArray" => {
            quote!(
                format!(
                    "[{} array: {}{}",
                    owo_colors::OwoColorize::blue(&"Byte"),
                    owo_colors::OwoColorize::blue(&#name.len()),
                    owo_colors::OwoColorize::bright_black(&"]")
                )
            )
        }
        "Value" | "Blob" => {
            quote!("[NBT]")
        }
        x => {
            let re = Regex::new("Vec < (.+) >").unwrap();
            let capt = re.captures(x);

            if capt.is_some() {
                let capt = capt.unwrap();
                let rec = dump_inner(&capt[1]);

                return quote!(format!(
                    "[{} array: {}{}",
                    #rec,
                    owo_colors::OwoColorize::blue(&#name.len()),
                    owo_colors::OwoColorize::bright_black(&"]")
                ));
            }

            let re = Regex::new("Option < (.+) >").unwrap();
            let capt = re.captures(x);
            if capt.is_some() {
                let capt = capt.unwrap();
                let rec = dump_inner(&capt[1]);

                return quote!(format!(
                    "Option({}{}",
                    #rec,
                    owo_colors::OwoColorize::bright_black(&")")
                ));
            }

            let mb_array = dump_array(x);
            if mb_array.is_some() {
                return mb_array.unwrap();
            }

            quote!(format!("{:?}", #name))
        }
    }
}

fn dump_inner(t: &str) -> proc_macro2::TokenStream {
    match t {
        "u8" | "i8"
        | "u16" | "i16" | "UnsignedShort" | "Short"
        | "u32" | "i32" | "UnsignedInt" | "Int"
        | "u64" | "i64" | "UnsignedLong" | "Long"
        | "Byte" | "bool" => {
            quote!(
                format!(
                    "{}",
                    owo_colors::OwoColorize::blue(&#t)
                )
            )
        }
        x => {
            let re = Regex::new(r"Box < \[(.+)] >").unwrap();
            let capt = re.captures(x);

            if capt.is_some() {
                let capt = capt.unwrap();
                let rec = dump_inner(&capt[1]);

                return quote!(format!(
                    "[{} array: {}{}",
                    #rec,
                    owo_colors::OwoColorize::blue(&#t.len()),
                    owo_colors::OwoColorize::bright_black(&"]")
                ));
            }

            let mb_array = dump_array(x);
            if mb_array.is_some() {
                return mb_array.unwrap();
            }

            quote!(owo_colors::OwoColorize::default_color(&#t))
        }
    }
}

fn dump_array(x: &str) -> Option<proc_macro2::TokenStream> {
    let re = Regex::new(r"\[([a-zA-Z0-9_]+) ; (\d+)]").unwrap();
    let capt = re.captures(x);

    if capt.is_none() {
        return None;
    }

    let capt = capt.unwrap();
    let capt_type = dump_inner(&capt[1]);
    let capt_len = &capt[2];

    return Some(quote!(format!(
        "[{} array: {}{}",
        #capt_type,
        owo_colors::OwoColorize::blue(&#capt_len),
        owo_colors::OwoColorize::bright_black(&"]")
    )));
}

#[proc_macro]
pub fn type_debug(tokens: TokenStream) -> TokenStream {
    let TypeDebug {
        name,
        ty
    } = parse_macro_input!(tokens as TypeDebug);

    let res = match ty {
        Type::Group(p) => {
            let last = quote!(#p);
            let last_str = last.to_string();

            dump(last_str, name)
        }
        _ => panic!("Expected Expr::Path, {:?} given", ty)
    };

    return proc_macro::TokenStream::from(res);
}

#[proc_macro_derive(Cases)]
pub fn derive_cases(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse(input).unwrap();

    let variants = match syn_item.data {
        syn::Data::Enum(enum_item) => {
            enum_item.variants.into_iter().map(|v| v.ident)
        }
        _ => panic!("AllVariants only works on enums"),
    };
    let enum_name = syn_item.ident;

    let expanded = quote! {
        impl #enum_name {
            pub fn cases() -> Vec<#enum_name> {
                vec![ #(#enum_name::#variants),* ]
            }
        }
    };
    expanded.into()
}

#[proc_macro]
pub fn to_snake(tokens: TokenStream) -> TokenStream {
    let ty: Type = syn::parse(tokens).unwrap();
    let ty_str = quote!(#ty);

    let re = regex::Regex::new("([a-z])([A-Z]+)").unwrap();
    let s = re.replace_all(&ty_str.to_string(), "${1}_${2}").to_lowercase();

    proc_macro::TokenStream::from(quote!(#s))
}

#[proc_macro]
pub fn config(tokens: TokenStream) -> TokenStream {
    config::config(tokens)
}
