use bevy_macro_utils::BevyManifest;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, punctuated::Punctuated, DeriveInput, Path, Token};

use crate::{util::concat, StateAttrs};

pub(crate) fn derive_register_state_helper(input: &DeriveInput, attrs: &StateAttrs) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    let bevy_app_path = BevyManifest::default().get_path("bevy_app");
    let app_ty = concat(bevy_app_path.clone(), "App");
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_extra_path = concat(crate_path.clone(), "extra");
    let crate_schedule_path = concat(crate_path.clone(), "schedule");
    let crate_setup_path = concat(crate_path.clone(), "setup");
    let register_state_trait = concat(crate_setup_path.clone(), "RegisterState");

    // Construct `ResolveStatePlugin`.
    let resolve_state = {
        let bevy_ecs_path = bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs");
        let bevy_ecs_schedule_path = concat(bevy_ecs_path, "schedule");
        let system_set = concat(bevy_ecs_schedule_path.clone(), "SystemSet");

        let crate_resolve_state_path = concat(crate_schedule_path.clone(), "resolve_state");
        let state_hook_ty = concat(crate_resolve_state_path.clone(), "StateHook");

        let after = attrs
            .after
            .iter()
            .map(|state| {
                quote! {
                    <#state_hook_ty::<#state> as #system_set>::intern(
                        &#state_hook_ty::<#state>::Resolve,
                    )
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        let before = attrs
            .before
            .iter()
            .map(|state| {
                quote! {
                    <#state_hook_ty::<#state> as #system_set>::intern(
                        &#state_hook_ty::<#state>::Resolve,
                    )
                }
            })
            .collect::<Punctuated<_, Token![,]>>();

        let state_plugin_ty = concat(crate_resolve_state_path.clone(), "ResolveStatePlugin");
        quote! { #state_plugin_ty::<Self>::new(vec![#after], vec![#before]), }
    };

    // Construct simple plugins.
    let plugin = |path: &Path, ty_prefix: &str, enable: bool, local: bool| {
        if !enable {
            return quote! {};
        }

        let state_plugin_ty = concat(path.clone(), &format!("{ty_prefix}Plugin"));
        let state_plugin = quote! { #state_plugin_ty::<Self>::default(), };
        if !local || !attrs.local {
            return state_plugin;
        }

        let local_state_plugin_ty = concat(path.clone(), &format!("Local{ty_prefix}Plugin"));
        let local_state_plugin = quote! { #local_state_plugin_ty::<Self>::default(), };

        quote! {
            #state_plugin
            #local_state_plugin
        }
    };

    let detect_change = {
        let crate_detect_change_path = concat(crate_schedule_path.clone(), "detect_change");
        plugin(
            &crate_detect_change_path,
            "DetectChange",
            attrs.detect_change,
            true,
        )
    };
    let flush_event = {
        let crate_flush_event_path = concat(crate_schedule_path.clone(), "flush_event");
        plugin(
            &crate_flush_event_path,
            "FlushEvent",
            attrs.flush_event,
            true,
        )
    };
    #[cfg(not(feature = "debug"))]
    let log_flush = quote! {};
    #[cfg(feature = "debug")]
    let log_flush = {
        let crate_debug_path = concat(crate_path.clone(), "debug");
        let crate_log_flush_path = concat(crate_debug_path.clone(), "log_flush");
        plugin(&crate_log_flush_path, "LogFlush", attrs.log_flush, true)
    };
    #[cfg(not(feature = "bevy_state"))]
    let bevy_state = quote! {};
    #[cfg(feature = "bevy_state")]
    let bevy_state = {
        let crate_bevy_state_path = concat(crate_extra_path.clone(), "bevy_state");
        plugin(&crate_bevy_state_path, "BevyState", attrs.bevy_state, false)
    };
    #[cfg(not(feature = "entity_scope"))]
    let entity_scope = quote! {};
    #[cfg(feature = "entity_scope")]
    let entity_scope = {
        let crate_entity_scope_path = concat(crate_extra_path.clone(), "entity_scope");
        plugin(
            &crate_entity_scope_path,
            "EntityScope",
            attrs.entity_scope,
            false,
        )
    };
    let apply_flush = {
        let crate_apply_flush_path = concat(crate_schedule_path.clone(), "apply_flush");
        plugin(
            &crate_apply_flush_path,
            "ApplyFlush",
            attrs.apply_flush,
            true,
        )
    };

    quote! {
        impl #impl_generics #register_state_trait for #ty_name #ty_generics #where_clause {
            fn register_state(app: &mut #app_ty) {
                app.add_plugins((
                    #resolve_state
                    #detect_change
                    #flush_event
                    #log_flush
                    #bevy_state
                    #entity_scope
                    #apply_flush
                ));
            }
        }
    }
    .into()
}
