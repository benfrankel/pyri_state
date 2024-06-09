//! Mark entities to despawn on [`State`] exit.
//!
//! Enable the `entity_scope` feature flag to use this module.

use std::marker::PhantomData;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    schedule::Schedule,
    system::{Commands, Query},
};
use bevy_hierarchy::DespawnRecursiveExt;

use crate::{pattern::StatePattern, state::State};

/// A component that marks an entity to despawn recursively on any exit of the [`State`] type `S`.
#[derive(Component)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct StateScope<S: State>(PhantomData<S>);

impl<S: State> Default for StateScope<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn despawn_scoped_entities<S: State>(
    mut commands: Commands,
    entity_query: Query<Entity, With<StateScope<S>>>,
) {
    for entity in &entity_query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Add a [`StateScope<S>`] despawning system for the [`State`] type `S` to a schedule.
///
/// Used in [`EntityScopePlugin<S>`].
pub fn schedule_entity_scope<S: State>(schedule: &mut Schedule) {
    schedule.add_systems(S::ANY.on_exit(despawn_scoped_entities::<S>));
}

/// A plugin that adds a [`StateScope<S>`] despawning system for the [`State`] type `S`.
///
/// Calls [`schedule_entity_scope<S>`].
#[cfg(feature = "bevy_app")]
pub struct EntityScopePlugin<S: State>(PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: State> bevy_app::Plugin for EntityScopePlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        schedule_entity_scope::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: State> Default for EntityScopePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
