//! Mark entities to automatically react to [`State`] flushes.
//!
//! Enable the `react` feature flag to use this module.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use std::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::{schedule::StateFlush, state::State};

    use super::schedule_react;

    /// A plugin that adds state flush reaction systems for the [`State`] type `S`.
    ///
    /// Calls [`schedule_react<S>`].
    pub struct ReactPlugin<S: State + Eq>(PhantomData<S>);

    impl<S: State + Eq> Plugin for ReactPlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_react::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State + Eq> Default for ReactPlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use std::marker::PhantomData;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    schedule::Schedule,
    system::{Commands, Query},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_render::view::visibility::Visibility;

use crate::{
    access::{CurrentRef, NextRef},
    pattern::StatePatternExtClone as _,
    state::State,
};

/// Add state flush reaction systems for the [`State`] type `S` to a schedule.
///
/// Used in [`ReactPlugin<S>`].
pub fn schedule_react<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(S::ANY.on_edge(
        (despawn_on_exit::<S>, hide_on_exit::<S>),
        show_on_enter::<S>,
    ));
}

/// A component that sets the despawn behavior on any exit of the [`State`] type `S`.
#[derive(Component, Default)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub enum DespawnOnExit<S: State> {
    /// Despawn the entity on any exit.
    Single,
    #[default]
    /// Despawn the entity and its descendants on any exit.
    Recursive,
    /// Despawn the entity's descendants on any exit.
    Descendants,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>),
}

fn despawn_on_exit<S: State>(
    mut commands: Commands,
    reaction_query: Query<(Entity, &DespawnOnExit<S>)>,
) {
    for (entity, reaction) in &reaction_query {
        match reaction {
            DespawnOnExit::Single => commands.entity(entity).despawn(),
            DespawnOnExit::Recursive => commands.entity(entity).despawn_recursive(),
            DespawnOnExit::Descendants => {
                commands.entity(entity).despawn_descendants();
            }
            DespawnOnExit::_PhantomData(_) => unreachable!(),
        }
    }
}

/// A component that shows an entity within a specific value of the [`State`] type `S`.
///
/// - On enter, the visibility will be set to inherited.
/// - On exit, the visibility will be set to hidden.
#[derive(Component, Default)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct StateVisible<S: State>(
    /// The state during which the entity should be visible.
    pub S,
);

fn hide_on_exit<S: State + Eq>(
    state: CurrentRef<S>,
    mut reaction_query: Query<(&mut Visibility, &StateVisible<S>)>,
) {
    for (mut visibility, reaction) in &mut reaction_query {
        if state.is_in(&reaction.0) {
            *visibility = Visibility::Hidden;
        }
    }
}

fn show_on_enter<S: State + Eq>(
    state: NextRef<S>,
    mut reaction_query: Query<(&mut Visibility, &StateVisible<S>)>,
) {
    for (mut visibility, reaction) in &mut reaction_query {
        if state.will_be_in(&reaction.0) {
            *visibility = Visibility::Inherited;
        }
    }
}
