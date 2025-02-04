use proc_macro::TokenStream;
use std::collections::HashMap;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use venial::{parse_declaration, Declaration, NamedField, StructFields};

pub fn config(tokens: TokenStream) -> TokenStream {
    let mut output = HashMap::new();
    let mut defaults = HashMap::new();

    let input = proc_macro2::TokenStream::from(tokens);
    let root_struct = parse_struct(input, vec![], &mut output, &mut defaults);

    let output = output
        .into_iter()
        .map(|(k, v)| {
            let mut stream = proc_macro2::TokenStream::new();
            stream.append_all(quote!(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bevy_ecs::prelude::Resource)]));
            stream.append(TokenTree::Ident(Ident::new("pub", Span::call_site())));
            stream.append(TokenTree::Ident(Ident::new("struct", Span::call_site())));
            stream.append(TokenTree::Ident(k));
            stream.append(TokenTree::Group(Group::new(Delimiter::Brace, v)));

            stream
        });

    let mut iter = output.into_iter();
    let mut stream = iter.next().unwrap();

    for s in iter {
        stream.append_all(s);
    }

    let mut defaults_stream = proc_macro2::TokenStream::new();
    for (key, value) in defaults {
        defaults_stream.append_all(quote!(
            builder = builder.set_default(#key, #value).unwrap();
        ));
    }

    let defaults_impl = quote!(
        impl #root_struct {
            pub fn defaults(mut builder: config::ConfigBuilder<config::builder::DefaultState>) -> config::ConfigBuilder<config::builder::DefaultState> {
                #defaults_stream

                builder
            }
        }
    );

    stream.append_all(defaults_impl);

    stream.into()
}

fn parse_struct(
    input: proc_macro2::TokenStream,
    keys: Vec<String>,
    output: &mut HashMap<Ident, proc_macro2::TokenStream>,
    defaults: &mut HashMap<String, proc_macro2::TokenStream>,
) -> Ident {
    let input = parse_declaration(input).unwrap();
    let Declaration::Struct(s) = input else {
        panic!("Expected struct declaration")
    };
    let StructFields::Named(named) = s.fields else {
        panic!("config! macro currently supports only Named struct fields. But found Unit or Tuple.")
    };

    for (field, _) in named.fields.inner {
        let tokens = field.clone().ty;
        let mut iter = tokens.clone().tokens.into_iter();
        let TokenTree::Ident(struct_or_type) = iter.next().unwrap() else {
            panic!("Expected Ident token")
        };

        if struct_or_type.to_string() == "struct" {
            let tokens = tokens.to_token_stream();

            let mut keys = keys.clone();
            keys.push(field.name.to_string());

            let ty = parse_struct(tokens.clone(), keys, output, defaults);

            append_field(
                field.clone(),
                quote!(#ty),
                s.name.clone(),
                output
            );
        } else {
            let generic_or_default = iter.next().unwrap();
            let mut ts = proc_macro2::TokenStream::new();
            ts.append(struct_or_type);

            if let TokenTree::Punct(punct) = generic_or_default {
                if punct.as_char() == '<' {
                    ts.append(punct);

                    let mut token = iter.next();
                    while token.is_some() {
                        let t = token.unwrap();
                        ts.append(t.clone());

                        if let TokenTree::Punct(punct) = t {
                            // Break loop on closed brace of generic
                            if punct.as_char() == '>' {
                                break
                            }
                        }

                        token = iter.next();
                    }
                } else {
                    panic!("Expected Punct '<', but {} given.", punct)
                }

                append_field(field.clone(), ts, s.name.clone(), output);

                let mut keys = keys.clone();
                keys.push(field.name.to_string());

                let mut default_tokens = proc_macro2::TokenStream::new();

                let mut default_token = iter.next();
                while default_token.is_some() {
                    default_tokens.append(default_token.unwrap());

                    default_token = iter.next();
                }

                defaults.insert(keys.join("."), default_tokens);
            } else {
                append_field(field.clone(), ts, s.name.clone(), output);

                let colon_first = iter.next();
                let default = if colon_first.is_some() {
                    let colon_first = colon_first.unwrap();

                    if let TokenTree::Punct(colon_first) = colon_first {
                        if colon_first.as_char() != ':' {
                            panic!("Expected colon as enum type-value delimiter")
                        }

                        let colon_second = iter.next();
                        if colon_second.is_some() {
                            let colon_second = colon_second.unwrap();

                            if let TokenTree::Punct(colon_second) = colon_second {
                                if colon_second.as_char() != ':' {
                                    panic!("Expected colon as enum type-value delimiter")
                                }

                                let enum_value = iter.next().unwrap();

                                let args = iter.next();
                                if args.is_some() {
                                    quote!(#generic_or_default::#enum_value #args)
                                } else {
                                    quote!(#generic_or_default::#enum_value)
                                }
                            } else {
                                panic!("Expected Punct token as enum type-value delimiter")
                            }
                        } else {
                            panic!("Unexpected end-of-sequence for enum type-value delimiter")
                        }
                    } else {
                        panic!("Expected Punct token as enum type-value delimiter")
                    }
                } else {
                    quote!(#generic_or_default)
                };

                let mut keys = keys.clone();
                keys.push(field.name.to_string());

                defaults.insert(keys.join("."), default);
            }
        }
    }

    return s.name
}

fn append_field(field: NamedField, ty: proc_macro2::TokenStream, outer_type: Ident, output: &mut HashMap<Ident, proc_macro2::TokenStream>) {
    if !output.contains_key(&outer_type) {
        let stream = proc_macro2::TokenStream::new();

        output.insert(outer_type.clone(), stream);
    }

    let mut stream = output.get_mut(&outer_type).unwrap();
    stream.append(TokenTree::Ident(Ident::new("pub", Span::call_site())));
    stream.append(field.name.clone());
    stream.append(field.tk_colon);
    stream.append_all(ty);
    stream.append(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
}
