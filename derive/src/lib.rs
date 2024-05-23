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
    // Construct crate module paths for portability
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_app_path = concat(crate_path.clone(), format_ident!("app"));
    let crate_state_path = concat(crate_path.clone(), format_ident!("state"));

    // Construct trait paths
    let state_trait = concat(crate_state_path.clone(), format_ident!("State_"));
    let get_state_config_trait = concat(crate_app_path.clone(), format_ident!("GetStateConfig"));
    let configure_state_trait = concat(crate_app_path.clone(), format_ident!("ConfigureState"));

    // Parse the decorated type
    let input = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Parse #[state(...)] attributes
    let state_attrs = parse_state_attrs(&input).expect("Failed to parse state attributes");

    // Construct state configs
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
            quote! { #state_config_ty::<Self>::new(), }
        } else {
            quote! {}
        }
    };

    let detect_change = simple_flag("DetectChange", state_attrs.detect_change);
    let send_event = simple_flag("SendEvent", state_attrs.send_event);
    let bevy_state = simple_flag("BevyState", state_attrs.bevy_state);
    let apply_flush = simple_flag("ApplyFlush", state_attrs.apply_flush);

    // Construct trait impls for the decorated type
    quote! {
        impl #impl_generics #state_trait for #ty_name #ty_generics #where_clause {}

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

struct StateAttrs {
    after: Punctuated<Type, Token![,]>,
    before: Punctuated<Type, Token![,]>,
    detect_change: bool,
    send_event: bool,
    bevy_state: bool,
    apply_flush: bool,
}

impl Default for StateAttrs {
    fn default() -> Self {
        Self {
            after: Default::default(),
            before: Default::default(),
            detect_change: true,
            send_event: true,
            bevy_state: false,
            apply_flush: true,
        }
    }
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

                Meta::Path(path) if path.is_ident("detect_change") => {
                    state_attrs.detect_change = true;
                }

                Meta::Path(path) if path.is_ident("no_detect_change") => {
                    state_attrs.detect_change = false;
                }

                Meta::Path(path) if path.is_ident("send_event") => {
                    state_attrs.send_event = true;
                }

                Meta::Path(path) if path.is_ident("no_send_event") => {
                    state_attrs.send_event = false;
                }

                Meta::Path(path) if path.is_ident("bevy_state") => {
                    state_attrs.bevy_state = true;
                }

                Meta::Path(path) if path.is_ident("no_bevy_state") => {
                    state_attrs.bevy_state = false;
                }

                Meta::Path(path) if path.is_ident("apply_flush") => {
                    state_attrs.apply_flush = true;
                }

                Meta::Path(path) if path.is_ident("no_apply_flush") => {
                    state_attrs.apply_flush = false;
                }

                _ => return Err(Error::new_spanned(meta, "unrecognized state attribute")),
            }
        }
    }

    Ok(state_attrs)
}
