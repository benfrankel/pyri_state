//! Mark entities to automatically react to [`State`] flushes.
//!
//! Enable the `react` feature flag to use this module.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

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

use core::marker::PhantomData;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectComponent;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    entity_disabling::Disabled,
    hierarchy::Children,
    query::With,
    schedule::Schedule,
    system::{Commands, Query},
};
use bevy_render::view::visibility::Visibility;

use crate::{
    access::{CurrentRef, NextRef},
    pattern::StatePattern as _,
    state::State,
};

/// Add state flush reaction systems for the [`State`] type `S` to a schedule.
///
/// Used in [`ReactPlugin<S>`].
pub fn schedule_react<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems((
        S::ANY.on_disable((
            despawn_on_disable_state::<S>,
            hide_on_disable_state::<S>,
            disable_on_disable_state::<S>,
        )),
        S::ANY.on_enable((show_on_enable_state::<S>, enable_on_enable_state::<S>)),
        S::ANY.on_exit((
            despawn_on_exit_state::<S>,
            hide_on_exit_state::<S>,
            disable_on_exit_state::<S>,
        )),
        S::ANY.on_enter((show_on_enter_state::<S>, enable_on_enter_state::<S>)),
    ));
}

/// A component that despawns an entity on any exit of the [`State`] type `S`.
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub enum DespawnOnExitState<S: State> {
    #[default]
    /// Despawn the entity and its descendants on any exit.
    Recursive,
    /// Despawn the entity's descendants on any exit.
    Descendants,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>),
}

fn despawn_on_exit_state<S: State>(
    mut commands: Commands,
    reaction_query: Query<(Entity, &DespawnOnExitState<S>)>,
) {
    for (entity, reaction) in &reaction_query {
        match reaction {
            DespawnOnExitState::Recursive => commands.entity(entity).try_despawn(),
            DespawnOnExitState::Descendants => {
                commands.entity(entity).despawn_related::<Children>();
            }
            DespawnOnExitState::_PhantomData(_) => unreachable!(),
        }
    }
}

/// A component that despawns an entity on any disable of the [`State`] type `S`.
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub enum DespawnOnDisableState<S: State> {
    #[default]
    /// Despawn the entity and its descendants on any disable.
    Recursive,
    /// Despawn the entity's descendants on any disable.
    Descendants,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>),
}

fn despawn_on_disable_state<S: State>(
    mut commands: Commands,
    reaction_query: Query<(Entity, &DespawnOnDisableState<S>)>,
) {
    for (entity, reaction) in &reaction_query {
        match reaction {
            DespawnOnDisableState::Recursive => commands.entity(entity).try_despawn(),
            DespawnOnDisableState::Descendants => {
                commands.entity(entity).despawn_related::<Children>();
            }
            DespawnOnDisableState::_PhantomData(_) => unreachable!(),
        }
    }
}

/// A component that shows an entity while in a specific value of the [`State`] type `S`.
///
/// - On enter, the visibility will be set to [`Visibility::Inherited`].
/// - On exit, the visibility will be set to [`Visibility::Hidden`].
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub struct VisibleInState<S: State>(
    /// The state during which the entity should be visible.
    pub S,
);

fn hide_on_exit_state<S: State + Eq>(
    state: CurrentRef<S>,
    mut reaction_query: Query<(&mut Visibility, &VisibleInState<S>)>,
) {
    for (mut visibility, reaction) in &mut reaction_query {
        if state.is_in(&reaction.0) {
            *visibility = Visibility::Hidden;
        }
    }
}

fn show_on_enter_state<S: State + Eq>(
    state: NextRef<S>,
    mut reaction_query: Query<(&mut Visibility, &VisibleInState<S>)>,
) {
    for (mut visibility, reaction) in &mut reaction_query {
        if state.will_be_in(&reaction.0) {
            *visibility = Visibility::Inherited;
        }
    }
}

/// A component that shows an entity while the [`State`] type `S` is enabled.
///
/// - On any enable, the visibility will be set to [`Visibility::Inherited`].
/// - On any disable, the visibility will be set to [`Visibility::Hidden`].
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub struct VisibleInEnabledState<S: State>(PhantomData<S>);

fn hide_on_disable_state<S: State + Eq>(
    mut reaction_query: Query<&mut Visibility, With<VisibleInEnabledState<S>>>,
) {
    for mut visibility in &mut reaction_query {
        *visibility = Visibility::Hidden;
    }
}

fn show_on_enable_state<S: State + Eq>(
    mut reaction_query: Query<&mut Visibility, With<VisibleInEnabledState<S>>>,
) {
    for mut visibility in &mut reaction_query {
        *visibility = Visibility::Inherited;
    }
}

/// A component that enables an entity (and its descendants) while in a specific value of the [`State`] type `S`.
///
/// - On enter, the [`Disabled`] component will be removed recursively.
/// - On exit, the [`Disabled`] component will be inserted recursively.
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub struct EnabledInState<S: State>(
    /// The state during which the entity should be enabled.
    pub S,
);

fn disable_on_exit_state<S: State + Eq>(
    mut commands: Commands,
    state: CurrentRef<S>,
    reaction_query: Query<(Entity, &EnabledInState<S>)>,
) {
    for (entity, reaction) in &reaction_query {
        if state.is_in(&reaction.0) {
            commands
                .entity(entity)
                .insert_recursive::<Children>(Disabled);
        }
    }
}

fn enable_on_enter_state<S: State + Eq>(
    mut commands: Commands,
    state: NextRef<S>,
    reaction_query: Query<(Entity, &EnabledInState<S>)>,
) {
    for (entity, reaction) in &reaction_query {
        if state.will_be_in(&reaction.0) {
            commands
                .entity(entity)
                .remove_recursive::<Children, Disabled>();
        }
    }
}

/// A component that enables an entity (and its descendants) while the [`State`] type `S` is enabled.
///
/// - On any enable, the [`Disabled`] component will be recursively removed.
/// - On any disable, the [`Disabled`] component will be recursively inserted.
#[derive(Component, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Component)
)]
pub struct EnabledInEnabledState<S: State>(PhantomData<S>);

fn disable_on_disable_state<S: State + Eq>(
    mut commands: Commands,
    reaction_query: Query<Entity, With<EnabledInEnabledState<S>>>,
) {
    for entity in &reaction_query {
        commands
            .entity(entity)
            .insert_recursive::<Children>(Disabled);
    }
}

fn enable_on_enable_state<S: State + Eq>(
    mut commands: Commands,
    reaction_query: Query<Entity, With<EnabledInEnabledState<S>>>,
) {
    for entity in &reaction_query {
        commands
            .entity(entity)
            .remove_recursive::<Children, Disabled>();
    }
}
