//! Mark entities to despawn on [`State`] exit.
//!
//! Enable the `entity_scope` feature flag to use this module.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use std::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::{schedule::StateFlush, state::State};

    use super::schedule_entity_scope;

    /// A plugin that adds a [`StateScope<S>`](super::StateScope) despawning system
    /// for the [`State`] type `S`.
    ///
    /// Calls [`schedule_entity_scope<S>`].
    pub struct EntityScopePlugin<S: State>(PhantomData<S>);

    impl<S: State> Plugin for EntityScopePlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_entity_scope::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State> Default for EntityScopePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

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
