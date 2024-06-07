use bevy_macro_utils::BevyManifest;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, punctuated::Punctuated, DeriveInput, Path, Token};

use crate::{util::concat, StateAttrs};

pub(crate) fn derive_add_state_helper(input: &DeriveInput, attrs: &StateAttrs) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    let bevy_app_path = BevyManifest::default().get_path("bevy_app");
    let app_ty = concat(bevy_app_path.clone(), "App");
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_app_path = concat(crate_path.clone(), "app");
    let add_state_trait = concat(crate_app_path.clone(), "AddState");
    let crate_state_path = concat(crate_path.clone(), "state");
    let current_state_ty = concat(crate_state_path.clone(), "CurrentState");
    let trigger_state_flush_ty = concat(crate_state_path.clone(), "TriggerStateFlush");
    #[cfg(feature = "debug")]
    let crate_debug_path = concat(crate_path.clone(), "debug");

    // Construct resolve state plugin.
    let resolve_state = {
        let bevy_ecs_path = bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs");
        let bevy_ecs_schedule_path = concat(bevy_ecs_path, "schedule");
        let system_set = concat(bevy_ecs_schedule_path.clone(), "SystemSet");

        let crate_schedule_path = concat(crate_path.clone(), "schedule");
        let state_flush_set = concat(crate_schedule_path.clone(), "StateFlushSet");

        let after = attrs
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

        let before = attrs
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

        let state_plugin_ty = concat(crate_app_path.clone(), "ResolveStatePlugin");
        quote! { #state_plugin_ty::<Self>::new(vec![#after], vec![#before]), }
    };

    // Construct simple plugins.
    let simple_plugin = |path: &Path, ty_prefix: &str, enable: bool| {
        if enable {
            let state_plugin_ty = concat(path.clone(), &format!("{ty_prefix}Plugin"));
            quote! { #state_plugin_ty::<Self>::default(), }
        } else {
            quote! {}
        }
    };
    let detect_change = simple_plugin(&crate_app_path, "DetectChange", attrs.detect_change);
    let flush_event = simple_plugin(&crate_app_path, "FlushEvent", attrs.flush_event);
    #[cfg(not(feature = "debug"))]
    let log_flush = quote! {};
    #[cfg(feature = "debug")]
    let log_flush = simple_plugin(&crate_debug_path, "LogFlush", attrs.log_flush);
    #[cfg(not(feature = "bevy_state"))]
    let bevy_state = quote! {};
    #[cfg(feature = "bevy_state")]
    let bevy_state = simple_plugin(&crate_app_path, "BevyState", attrs.bevy_state);
    let apply_flush = simple_plugin(&crate_app_path, "ApplyFlush", attrs.apply_flush);

    quote! {
        impl #impl_generics #add_state_trait for #ty_name #ty_generics #where_clause {
            type AddStorage = Self::Storage;

            fn add_state(app: &mut #app_ty) {
                app.init_resource::<#current_state_ty<Self>>()
                    .init_resource::<#trigger_state_flush_ty<Self>>()
                    .add_plugins((
                        #resolve_state
                        #detect_change
                        #flush_event
                        #log_flush
                        #bevy_state
                        #apply_flush
                    ));
            }
        }
    }
    .into()
}
