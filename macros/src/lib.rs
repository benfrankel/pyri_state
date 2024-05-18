use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(State)]
pub fn derive_state(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ty_name = &ast.ident;

    quote! {
        impl #impl_generics State for #ty_name #ty_generics #where_clause {}
    }
    .into()
}
