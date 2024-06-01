#[cfg(feature = "bevy_app")]
mod app;
mod util;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, DeriveInput, Error, Meta, Path, Result,
    Token, Type,
};

use crate::util::concat;

#[proc_macro_derive(State, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    // Parse the type and #[state(...)] attributes.
    let input = parse_macro_input!(input as DeriveInput);
    let attrs = parse_state_attrs(&input).expect("Failed to parse state attributes");

    // Construct RawState impl.
    let impl_raw_state = derive_state_helper(&input, &attrs);

    // Construct AddState impl.
    #[cfg(not(feature = "bevy_app"))]
    let impl_add_state = quote! {};
    #[cfg(feature = "bevy_app")]
    let impl_add_state = app::derive_add_state_helper(&input, &attrs);

    quote! {
        #impl_raw_state
        #impl_add_state
    }
    .into()
}

fn derive_state_helper(input: &DeriveInput, attrs: &StateAttrs) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_state_path = concat(crate_path.clone(), format_ident!("state"));
    let raw_state_trait = concat(crate_state_path.clone(), format_ident!("RawState"));

    // Construct storage type.
    let storage_ty = if let Some(storage) = attrs.storage.as_ref() {
        quote! {
            #storage
        }
    } else {
        let crate_storage_path = concat(crate_path.clone(), format_ident!("storage"));
        let state_slot_ty = concat(crate_storage_path.clone(), format_ident!("StateSlot"));

        quote! {
            #state_slot_ty<Self>
        }
    };

    // Construct RawState impl.
    quote! {
        impl #impl_generics #raw_state_trait for #ty_name #ty_generics #where_clause {
            type Storage = #storage_ty;
        }
    }
    .into()
}

#[derive(Default)]
struct StateAttrs {
    storage: Option<Type>,
    after: Punctuated<Type, Token![,]>,
    before: Punctuated<Type, Token![,]>,
    no_defaults: bool,
    detect_change: bool,
    flush_event: bool,
    log_flush: bool,
    bevy_state: bool,
    apply_flush: bool,
}

// Parse #[state(...)] attributes.
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
                        .expect("invalid after states");
                }

                Meta::List(meta) if meta.path.is_ident("before") => {
                    state_attrs.before = meta
                        .parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated)
                        .expect("invalid before states");
                }

                Meta::List(meta) if meta.path.is_ident("storage") => {
                    state_attrs.storage = Some(meta.parse_args().expect("invalid state storage"));
                }

                Meta::Path(path) => {
                    let Some(ident) = path.get_ident() else {
                        return Err(Error::new_spanned(path, "invalid state attribute"));
                    };

                    match ident.to_string().as_str() {
                        "no_defaults" => state_attrs.no_defaults = true,
                        "detect_change" => state_attrs.detect_change = true,
                        "flush_event" => state_attrs.flush_event = true,
                        "log_flush" => state_attrs.log_flush = true,
                        "bevy_state" => state_attrs.bevy_state = true,
                        "apply_flush" => state_attrs.apply_flush = true,
                        _ => return Err(Error::new_spanned(ident, "invalid state attribute")),
                    }
                }

                _ => return Err(Error::new_spanned(meta, "invalid state attribute")),
            }
        }
    }

    // Enable defaults.
    if !state_attrs.no_defaults {
        state_attrs.detect_change = true;
        state_attrs.flush_event = true;
        state_attrs.apply_flush = true;
    }

    Ok(state_attrs)
}
