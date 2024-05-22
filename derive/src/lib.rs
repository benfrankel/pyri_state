use bevy_macro_utils::BevyManifest;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, DeriveInput, Error, Meta, Path,
    PathSegment, Result, Token, Type,
};

fn concat(mut base_path: Path, suffix: impl Into<PathSegment>) -> Path {
    base_path.segments.push(suffix.into());
    base_path
}

#[proc_macro_derive(State_, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    let state_attrs = parse_state_attrs(&input).expect("Failed to parse state attributes");

    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_config_path = concat(crate_path.clone(), format_ident!("config"));
    let crate_state_path = concat(crate_path.clone(), format_ident!("state"));

    let state = concat(crate_state_path.clone(), format_ident!("State_"));
    let configure_state = concat(crate_config_path.clone(), format_ident!("ConfigureState"));

    let resolve_state = {
        let bevy_ecs_path = BevyManifest::default().get_path("bevy_ecs");
        let bevy_ecs_schedule_path = concat(bevy_ecs_path, format_ident!("schedule"));
        let system_set = concat(bevy_ecs_schedule_path.clone(), format_ident!("SystemSet"));

        let crate_schedule_path = concat(crate_path.clone(), format_ident!("schedule"));
        let state_flush_set = concat(crate_schedule_path.clone(), format_ident!("StateFlushSet"));

        let after = state_attrs
            .after
            .iter()
            .map(|state| {
                quote! {
                    <#state_flush_set::<#state> as #system_set>::intern(
                        &#state_flush_set::<#state>::Resolve,
                    )
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        let before = state_attrs
            .before
            .iter()
            .map(|state| {
                quote! {
                    <#state_flush_set::<#state> as #system_set>::intern(
                        &#state_flush_set::<#state>::Resolve,
                    )
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        let state_config_ty = concat(
            crate_config_path.clone(),
            format_ident!("StateConfigResolveState"),
        );
        quote! { #state_config_ty::<Self>::new(vec![#after], vec![#before]), }
    };

    let detect_change = if state_attrs.no_detect_change {
        quote! {}
    } else {
        let state_config_ty = concat(
            crate_config_path.clone(),
            format_ident!("StateConfigDetectChange"),
        );
        quote! { #state_config_ty::<Self>::new(), }
    };

    let send_event = if state_attrs.no_send_event {
        quote! {}
    } else {
        let state_config_ty = concat(
            crate_config_path.clone(),
            format_ident!("StateConfigSendEvent"),
        );
        quote! { #state_config_ty::<Self>::new(), }
    };

    let apply_flush = if state_attrs.no_apply_flush {
        quote! {}
    } else {
        let state_config_ty = concat(
            crate_config_path.clone(),
            format_ident!("StateConfigApplyFlush"),
        );
        quote! { #state_config_ty::<Self>::new(), }
    };

    quote! {
        impl #impl_generics #state for #ty_name #ty_generics #where_clause {
            fn config() -> impl #configure_state {
                (
                    #resolve_state
                    #detect_change
                    #send_event
                    #apply_flush
                )
            }
        }
    }
    .into()
}

#[derive(Default)]
struct StateAttrs {
    after: Punctuated<Type, Token![,]>,
    before: Punctuated<Type, Token![,]>,
    no_detect_change: bool,
    no_send_event: bool,
    no_apply_flush: bool,
}

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

                Meta::Path(path) if path.is_ident("no_detect_change") => {
                    state_attrs.no_detect_change = true;
                }

                Meta::Path(path) if path.is_ident("no_send_event") => {
                    state_attrs.no_send_event = true;
                }

                Meta::Path(path) if path.is_ident("no_apply_flush") => {
                    state_attrs.no_apply_flush = true;
                }

                _ => return Err(Error::new_spanned(meta, "unrecognized state attribute")),
            }
        }
    }

    Ok(state_attrs)
}
