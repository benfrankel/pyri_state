use bevy_macro_utils::BevyManifest;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_str, punctuated::Punctuated, DeriveInput, Path, Token};

use crate::{util::concat, StateAttrs};

pub(crate) fn derive_add_state_helper(input: &DeriveInput, attrs: &StateAttrs) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ty_name = &input.ident;

    // Construct paths.
    let bevy_app_path = BevyManifest::default().get_path("bevy_app");
    let app_ty = concat(bevy_app_path.clone(), format_ident!("App"));
    // TODO: This is not 100% portable I guess, but probably good enough.
    let crate_path = parse_str::<Path>("pyri_state").unwrap();
    let crate_app_path = concat(crate_path.clone(), format_ident!("app"));
    let add_state_trait = concat(crate_app_path.clone(), format_ident!("AddState"));
    let add_state_storage_trait = concat(crate_app_path.clone(), format_ident!("AddStateStorage"));
    let crate_state_path = concat(crate_path.clone(), format_ident!("state"));
    let current_state_ty = concat(crate_state_path.clone(), format_ident!("CurrentState"));
    let flush_state_ty = concat(crate_state_path.clone(), format_ident!("FlushState"));

    // Construct resolve state plugin.
    let resolve_state = {
        let bevy_ecs_path = bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs");
        let bevy_ecs_schedule_path = concat(bevy_ecs_path, format_ident!("schedule"));
        let system_set = concat(bevy_ecs_schedule_path.clone(), format_ident!("SystemSet"));

        let crate_schedule_path = concat(crate_path.clone(), format_ident!("schedule"));
        let state_flush_set = concat(crate_schedule_path.clone(), format_ident!("StateFlushSet"));

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

        let state_plugin_ty = concat(crate_app_path.clone(), format_ident!("ResolveStatePlugin"));
        quote! { #state_plugin_ty::<Self>::new(vec![#after], vec![#before]), }
    };

    // Construct simple plugins.
    let simple_plugin = |ty_prefix: &str, enable: bool| {
        if enable {
            let state_plugin_ty =
                concat(crate_app_path.clone(), format_ident!("{ty_prefix}Plugin"));
            quote! { #state_plugin_ty::<Self>::default(), }
        } else {
            quote! {}
        }
    };
    let detect_change = simple_plugin("DetectChange", attrs.detect_change);
    let flush_event = simple_plugin("FlushEvent", attrs.flush_event);
    let bevy_state = simple_plugin("BevyState", attrs.bevy_state);
    let apply_flush = simple_plugin("ApplyFlush", attrs.apply_flush);

    quote! {
        impl #impl_generics #add_state_trait for #ty_name #ty_generics #where_clause {
            fn add_state(app: &mut #app_ty, state: Option<Self>) {
                <Self::Storage as #add_state_storage_trait<Self>>::add_state_storage(app, state);
                app.init_resource::<#current_state_ty<Self>>()
                    .init_resource::<#flush_state_ty<Self>>()
                    .add_plugins((
                        #resolve_state
                        #detect_change
                        #flush_event
                        #bevy_state
                        #apply_flush
                    ));
            }
        }
    }
    .into()
}
