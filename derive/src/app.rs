use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_str, punctuated::Punctuated, DeriveInput, Error, Meta, Path, Result, Token, Type};

use crate::util::concat;

pub(crate) fn derive_get_state_config(input: &DeriveInput) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Parse #[state(...)] attributes
    let state_attrs = parse_state_attrs(&input).expect("Failed to parse state attributes");

    // Construct trait paths
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_app_path = concat(crate_path.clone(), format_ident!("app"));
    let get_state_config_trait = concat(crate_app_path.clone(), format_ident!("GetStateConfig"));
    let configure_state_trait = concat(crate_app_path.clone(), format_ident!("ConfigureState"));

    // Construct state configs
    let resolve_state = {
        let bevy_ecs_path = bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs");
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
            crate_app_path.clone(),
            format_ident!("StateConfigResolveState"),
        );
        quote! { #state_config_ty::<Self>::new(vec![#after], vec![#before]), }
    };

    let simple_flag = |ty_suffix: &str, enable: bool| {
        if enable {
            let state_config_ty = concat(
                crate_app_path.clone(),
                format_ident!("StateConfig{ty_suffix}"),
            );
            quote! { #state_config_ty::<Self>::default(), }
        } else {
            quote! {}
        }
    };

    let detect_change = simple_flag("DetectChange", state_attrs.detect_change);
    let send_event = simple_flag("SendEvent", state_attrs.send_event);
    let bevy_state = simple_flag("BevyState", state_attrs.bevy_state);
    let apply_flush = simple_flag("ApplyFlush", state_attrs.apply_flush);

    quote! {
        impl #impl_generics #get_state_config_trait for #ty_name #ty_generics #where_clause {
            fn get_config() -> impl #configure_state_trait {
                (
                    #resolve_state
                    #detect_change
                    #send_event
                    #bevy_state
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
    no_defaults: bool,
    detect_change: bool,
    send_event: bool,
    bevy_state: bool,
    apply_flush: bool,
}

// Parse #[state(...)] attributes
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

                Meta::Path(path) => {
                    let Some(ident) = path.get_ident() else {
                        return Err(Error::new_spanned(path, "invalid state attribute"));
                    };

                    match ident.to_string().as_str() {
                        "no_defaults" => state_attrs.no_defaults = true,
                        "detect_change" => state_attrs.detect_change = true,
                        "send_event" => state_attrs.send_event = true,
                        "bevy_state" => state_attrs.bevy_state = true,
                        "apply_flush" => state_attrs.apply_flush = true,
                        _ => return Err(Error::new_spanned(ident, "invalid state attribute")),
                    }
                }

                _ => return Err(Error::new_spanned(meta, "invalid state attribute")),
            }
        }
    }

    // Enable defaults
    if !state_attrs.no_defaults {
        state_attrs.detect_change = true;
        state_attrs.send_event = true;
        state_attrs.apply_flush = true;
    }

    Ok(state_attrs)
}
