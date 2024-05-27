#[cfg(feature = "bevy_app")]
mod app;
mod util;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_str, DeriveInput, Path};

use crate::util::concat;

fn derive_raw_state_helper(input: &DeriveInput) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_state_path = concat(crate_path.clone(), format_ident!("state"));
    let raw_state_trait = concat(crate_state_path.clone(), format_ident!("RawState"));

    // Construct trait impl for the decorated type
    quote! {
        impl #impl_generics #raw_state_trait for #ty_name #ty_generics #where_clause {}
    }
}

#[proc_macro_derive(RawState)]
pub fn derive_raw_state(input: TokenStream) -> TokenStream {
    derive_raw_state_helper(&parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(State, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    // Parse the decorated type
    let input = parse_macro_input!(input as DeriveInput);

    let impl_raw_state = derive_raw_state_helper(&input);

    #[cfg(not(feature = "bevy_app"))]
    let impl_configure_state = quote! {};
    #[cfg(feature = "bevy_app")]
    let impl_add_state = app::derive_add_state_helper(&input);

    quote! {
        #impl_raw_state
        #impl_add_state
    }
    .into()
}
