// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>, and
// Andrew Hobden (@hoverbear) <andrew@hoverbear.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// This work was derived from Structopt (https://github.com/TeXitoi/structopt)
// commit#ea76fa1b1b273e65e3b0b1046643715b49bec51f which is licensed under the
// MIT/Apache 2.0 license.
use proc_macro2;
use syn;
use syn::punctuated;
use syn::token;
use syn::spanned::Spanned as _;
use proc_macro_error::{span_error, call_site_error};

use derives;
use derives::attrs::{Attrs, Kind, Parser, Ty};
use derives::from_argmatches;
use derives::into_app;

/// Generate a block of code to add arguments/subcommands corresponding to
/// the `fields` to an app.
fn gen_app_augmentation(
    fields: &punctuated::Punctuated<syn::Field, token::Comma>,
    app_var: &syn::Ident,
    parent_attribute: &Attrs,
) -> proc_macro2::TokenStream {
    let mut subcmds = fields
        .iter()
        .filter_map(|field| {
            let attrs = Attrs::from_field(&field, parent_attribute.casing());
            if let Kind::Subcommand(ty) = &*attrs.kind() {
                let subcmd_type = match (**ty, derives::sub_type(&field.ty)) {
                    (Ty::Option, Some(sub_type)) => sub_type,
                    _ => &field.ty,
                };
                let required = if **ty == Ty::Option {
                    quote!()
                } else {
                    quote! {
                        let #app_var = #app_var.setting(
                            ::clap::AppSettings::SubcommandRequiredElseHelp
                        );
                    }
                };

                let span = field.span();
                let ts = quote! {
                    let #app_var = <#subcmd_type>::augment_app( #app_var );
                    #required
                };
                Some((span, ts))
            } else {
                None
            }
        });

    let subcmd = subcmds.next().map(|(_, ts)| ts);
    if let Some((span, _)) = subcmds.next() {
        span_error!(
            span,
            "nested subcommands are not allowed, that's the second"
        );
    }

    let args = fields.iter().filter_map(|field| {
        let attrs = Attrs::from_field(field, parent_attribute.casing());
        match &*attrs.kind() {
            Kind::Subcommand(_) | Kind::Skip => None,
            Kind::FlattenStruct => {
                let ty = &field.ty;
                Some(quote! {
                    let #app_var = <#ty>::augment_app(#app_var);
                    let #app_var = if <#ty>::is_subcommand() {
                        #app_var.setting(::clap::AppSettings::SubcommandRequiredElseHelp)
                    } else {
                        #app_var
                    };
                })
            }
            Kind::Arg(ty) => {
                let convert_type = match **ty {
                    Ty::Vec | Ty::Option => derives::sub_type(&field.ty).unwrap_or(&field.ty),
                    Ty::OptionOption | Ty::OptionVec => derives::sub_type(&field.ty).and_then(derives::sub_type).unwrap_or(&field.ty),
                    _ => &field.ty,
                };

                let occurrences = *attrs.parser().0 == Parser::FromOccurrences;

                let (parser, f) = attrs.parser();
                let validator = match **parser {
                    Parser::TryFromStr => quote! {
                        .validator(|s| {
                            #f(&s)
                            .map(|_: #convert_type| ())
                            .map_err(|e| e.to_string())
                        })
                    },
                    Parser::TryFromOsStr => quote! {
                        .validator_os(|s| #f(&s).map(|_: #convert_type| ()))
                    },
                    _ => quote!(),
                };

                // @TODO remove unneccessary builders
                let modifier = match **ty {
                    Ty::Bool => quote!(),
                    Ty::Option => quote!( .takes_value(true) #validator ),
                    Ty::OptionOption => {
                        quote! ( .takes_value(true).multiple(false).min_values(0).max_values(1) #validator )
                    }
                    Ty::OptionVec => {
                        quote! ( .takes_value(true).multiple(true).min_values(0) #validator )
                    }
                    Ty::Vec => quote!( .takes_value(true).multiple(true) #validator ),
                    Ty::Other if occurrences => quote!( .multiple_occurrences(true) ),
                    Ty::Other => {
                        let required = !attrs.has_method("default_value");
                        quote!( .takes_value(true).required(#required) #validator )
                    }
                };

                let name = attrs.cased_name();
                let methods = attrs.field_methods();

                Some(quote! {
                    let #app_var = #app_var.arg(
                        ::clap::Arg::with_name(#name)
                            #modifier
                            #methods
                    );
                })
            }
        }
    });

    quote! {{
        #( #args )*
        #subcmd
        #app_var
    }}
}

fn gen_augment_app_fn(
    fields: &punctuated::Punctuated<syn::Field, token::Comma>,
    parent_attribute: &Attrs,
) -> proc_macro2::TokenStream {
    let app_var = syn::Ident::new("app", proc_macro2::Span::call_site());
    let augmentation = gen_app_augmentation(fields, &app_var, parent_attribute);
    quote! {
        pub fn augment_app<'b>(
            #app_var: ::clap::App<'b>
        ) -> ::clap::App<'b> {
            #augmentation
        }
    }
}

fn gen_augment_app_for_enum(
    variants: &punctuated::Punctuated<syn::Variant, token::Comma>,
    parent_attribute: &Attrs,
) -> proc_macro2::TokenStream {
    use syn::Fields::*;

    let subcommands = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(
            &variant.attrs,
            variant.ident.clone().into(),
            parent_attribute.casing(),
        );
        let app_var = syn::Ident::new("subcommand", proc_macro2::Span::call_site());
        let arg_block = match variant.fields {
            Named(ref fields) => gen_app_augmentation(&fields.named, &app_var, &attrs),
            Unit => quote!( #app_var ),
            Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) if unnamed.len() == 1 => {
                let ty = &unnamed[0];
                quote! {
                    {
                        let #app_var = <#ty>::augment_app(#app_var);
                        if <#ty>::is_subcommand() {
                            #app_var.setting(
                                ::clap::AppSettings::SubcommandRequiredElseHelp
                            )
                        } else {
                            #app_var
                        }
                    }
                }
            }
            Unnamed(..) => call_site_error!("{}: tuple enums are not supported", variant.ident),
        };

        let name = attrs.cased_name();
        let from_attrs = attrs.top_level_methods();

        quote! {
            .subcommand({
                let #app_var = ::clap::App::new(#name);
                let #app_var = #arg_block;
                #app_var#from_attrs
            })
        }
    });

    quote! {
        pub fn augment_app<'b>(
            app: ::clap::App<'b>
        ) -> ::clap::App<'b> {
            app #( #subcommands )*
        }
    }
}

fn gen_from_subcommand(
    name: &syn::Ident,
    variants: &punctuated::Punctuated<syn::Variant, token::Comma>,
    parent_attribute: &Attrs,
) -> proc_macro2::TokenStream {
    use syn::Fields::*;

    let match_arms = variants.iter().map(|variant| {
        let attrs = Attrs::from_struct(
            &variant.attrs,
            variant.ident.clone().into(),
            parent_attribute.casing()
        );
        let sub_name = attrs.cased_name();
        let variant_name = &variant.ident;
        let constructor_block = match variant.fields {
            Named(ref fields) => from_argmatches::gen_constructor(&fields.named, &attrs),
            Unit => quote!(),
            Unnamed(ref fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0];
                quote!( ( <#ty as ::clap::FromArgMatches>::from_argmatches(matches) ) )
            }
            Unnamed(..) => call_site_error!("{}: tuple enums are not supported", variant.ident),
        };

        quote! {
            (#sub_name, Some(matches)) =>
                Some(#name :: #variant_name #constructor_block)
        }
    });

    quote! {
        pub fn from_subcommand<'b>(
            sub: (&'b str, Option<&'b ::clap::ArgMatches>)
        ) -> Option<Self> {
            match sub {
                #( #match_arms ),*,
                _ => None
            }
        }
    }
}

fn clap_impl_for_struct(
    name: &syn::Ident,
    fields: &punctuated::Punctuated<syn::Field, token::Comma>,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let into_app_impl = into_app::gen_into_app_impl_for_struct(name, attrs);
    let into_app_impl_tokens = into_app_impl.tokens;
    let augment_app_fn = gen_augment_app_fn(fields, &into_app_impl.attrs);
    let from_argmatches_impl = from_argmatches::gen_from_argmatches_impl_for_struct(name, fields, &into_app_impl.attrs);
    let parse_fns = gen_parse_fns(name);

    quote! {
        #[allow(unused_variables)]
        impl ::clap::Clap for #name { }

        #into_app_impl_tokens

        #from_argmatches_impl

        #[allow(dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_app_fn

            #parse_fns

            pub fn is_subcommand() -> bool { false }
        }
    }
}

fn clap_impl_for_enum(
    name: &syn::Ident,
    variants: &punctuated::Punctuated<syn::Variant, token::Comma>,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let into_app_impl = into_app::gen_into_app_impl_for_enum(name, attrs);
    let into_app_impl_tokens = into_app_impl.tokens;
    let augment_app_fn = gen_augment_app_for_enum(variants, &into_app_impl.attrs);
    let from_argmatches_impl = from_argmatches::gen_from_argmatches_impl_for_enum(name);
    let from_subcommand = gen_from_subcommand(name, variants, &into_app_impl.attrs);
    let parse_fns = gen_parse_fns(name);

    quote! {
        #[allow(unused_variables)]
        impl ::clap::Clap for #name { }

        #into_app_impl_tokens

        #from_argmatches_impl

        #[allow(unused_variables, dead_code, unreachable_code)]
        #[doc(hidden)]
        impl #name {
            #augment_app_fn

            #from_subcommand

            #parse_fns

            pub fn is_subcommand() -> bool { true }
        }
    }
}

pub fn derive_clap(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    use syn::Data::*;

    let struct_name = &input.ident;
    match input.data {
        Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => clap_impl_for_struct(struct_name, &fields.named, &input.attrs),
        Enum(ref e) => clap_impl_for_enum(struct_name, &e.variants, &input.attrs),
        _ => call_site_error!("clap_derive only supports non-tuple structs and enums"),
    }
}

fn gen_parse_fns(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        fn parse() -> #name {
            use ::clap::{FromArgMatches, IntoApp};
            #name::from_argmatches(&#name::into_app().get_matches())
        }

        fn try_parse() -> ::std::result::Result<#name, ::clap::Error> {
            use ::clap::{FromArgMatches, IntoApp};
            Ok(#name::from_argmatches(&#name::into_app().try_get_matches()?))
        }

        fn parse_from<I, T>(itr: I) -> #name
        where
            I: ::std::iter::IntoIterator<Item = T>,
            T: Into<::std::ffi::OsString> + Clone {
            use ::clap::{FromArgMatches, IntoApp};
            #name::from_argmatches(&#name::into_app().get_matches_from(itr))
        }

        fn try_parse_from<I, T>(itr: I) -> ::std::result::Result<#name, ::clap::Error>
        where
            I: ::std::iter::IntoIterator<Item = T>,
            T: Into<::std::ffi::OsString> + Clone {
            use ::clap::{FromArgMatches, IntoApp};
            Ok(#name::from_argmatches(&#name::into_app().try_get_matches_from(itr)?))
        }
    }
}
