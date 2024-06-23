//! TODO: Module-level docs

use bevy_ecs::component::Component;

/// A marker [`Component`] for the global states entity spawned by [`StatePlugin`].
#[derive(Component, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct GlobalStates;

// TODO: Move system params here. Move `StateStorage` into `state.rs`. Rename `storage.rs` to `buffer.rs`.
