//! Derive macros for `pyri_state`.

#[cfg(feature = "bevy_app")]
mod app;
mod util;

use bevy_macro_utils::BevyManifest;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Error, Meta, Path, Result, Token, Type, parse_macro_input, parse_str,
    punctuated::Punctuated,
};

use crate::util::concat;

#[proc_macro_derive(State, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    // Parse the type and `#[state(...)]` attributes.
    let input = parse_macro_input!(input as DeriveInput);
    let attrs = parse_state_attrs(&input).expect("Failed to parse state attributes");

    // Construct `State` impl.
    let impl_state = derive_state_helper(&input, &attrs);

    // Construct `RegisterState` impl.
    #[cfg(not(feature = "bevy_app"))]
    let impl_register_state = quote! {};
    #[cfg(feature = "bevy_app")]
    let impl_register_state = app::derive_register_state_helper(&input, &attrs);

    // Construct `Resource` impl.
    let impl_resource = derive_resource_helper(&input);

    quote! {
        #impl_state
        #impl_register_state
        #impl_resource
    }
    .into()
}

fn derive_resource_helper(input: &DeriveInput) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    let bevy_ecs_path = BevyManifest::shared().get_path("bevy_ecs");
    let bevy_ecs_resource_path = concat(&bevy_ecs_path, "resource");
    let resource_trait = concat(&bevy_ecs_resource_path, "Resource");

    quote! {
        impl #impl_generics #resource_trait for #ty_name #ty_generics #where_clause {}
    }
    .into()
}

fn derive_state_helper(input: &DeriveInput, attrs: &StateAttrs) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_state_path = concat(&crate_path, "state");
    let state_trait = concat(&crate_state_path, "State");

    // Construct `NextState` type.
    let next_ty = if let Some(next) = attrs.next.as_ref() {
        quote! {
            #next
        }
    } else {
        let crate_next_state_path = concat(&crate_path, "next_state");
        let crate_buffer_path = concat(&crate_next_state_path, "buffer");
        let state_buffer_ty = concat(&crate_buffer_path, "NextStateBuffer");

        quote! {
            #state_buffer_ty<Self>
        }
    };

    // Construct `State` impl.
    quote! {
        impl #impl_generics #state_trait for #ty_name #ty_generics #where_clause {
            type Next = #next_ty;
        }
    }
    .into()
}

#[derive(Default)]
struct StateAttrs {
    next: Option<Type>,
    local: bool,
    after: Punctuated<Type, Token![,]>,
    before: Punctuated<Type, Token![,]>,
    no_defaults: bool,
    detect_change: bool,
    flush_message: bool,
    log_flush: bool,
    bevy_state: bool,
    react: bool,
    apply_flush: bool,
}

// Parse `#[state(...)]` attributes.
fn parse_state_attrs(input: &DeriveInput) -> Result<StateAttrs> {
    let mut state_attrs = StateAttrs::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("state") {
            continue;
        }

        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            match meta {
                Meta::List(meta) if meta.path.is_ident("after") => {
                    state_attrs.after = meta
                        .parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated)
                        .expect("invalid `after` states");
                }

                Meta::List(meta) if meta.path.is_ident("before") => {
                    state_attrs.before = meta
                        .parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated)
                        .expect("invalid `before` states");
                }

                Meta::List(meta) if meta.path.is_ident("next") => {
                    state_attrs.next = Some(meta.parse_args().expect("invalid `next` type"));
                }

                Meta::Path(path) => {
                    let Some(ident) = path.get_ident() else {
                        return Err(Error::new_spanned(path, "invalid state attribute"));
                    };

                    match ident.to_string().as_str() {
                        "no_defaults" => state_attrs.no_defaults = true,
                        "local" => state_attrs.local = true,
                        "detect_change" => state_attrs.detect_change = true,
                        "flush_message" => state_attrs.flush_message = true,
                        "log_flush" => state_attrs.log_flush = true,
                        "bevy_state" => state_attrs.bevy_state = true,
                        "react" => state_attrs.react = true,
                        "apply_flush" => state_attrs.apply_flush = true,
                        _ => return Err(Error::new_spanned(ident, "invalid state attribute")),
                    }
                }

                _ => return Err(Error::new_spanned(meta, "invalid state attribute")),
            }
        }
    }

    // Enable default options.
    if !state_attrs.no_defaults {
        state_attrs.detect_change = true;
        state_attrs.flush_message = true;
        state_attrs.apply_flush = true;
    }

    Ok(state_attrs)
}
