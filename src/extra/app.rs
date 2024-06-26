//! State configuration tools.
//!
//! Enable the `bevy_app` feature flag to use this module.
//!
//! See the [derive macro](pyri_state_derive::State) for an easy way to impl [`AddState`] and
//! enable the plugins provided by this module.

use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{InternedSystemSet, SystemSet},
    world::{FromWorld, World},
};

use crate::{
    access::GlobalStatesEntity,
    schedule::{
        schedule_apply_flush, schedule_detect_change, schedule_flush_event, schedule_resolve_state,
        StateFlush, StateFlushEvent, StateHook,
    },
    state::{NextState, State, TriggerStateFlush},
};

/// A plugin that performs the required setup for [`State`] types to function:
///
/// - Adds the [`StateFlush`] schedule to the [`MainScheduleOrder`] after [`PreUpdate`].
/// - Spawns the [`GlobalStates`](crate::access::GlobalStates) entity.
/// - Adds the [`bevy_state` plugin](bevy_state::app::StatesPlugin) if the
/// `bevy_state` feature is enabled.
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        // Add the `StateFlush` schedule.
        app.init_schedule(StateFlush)
            .world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);

        // Spawn the `GlobalStates` entity.
        app.init_resource::<GlobalStatesEntity>();

        // Add the `bevy_state` plugin.
        #[cfg(feature = "bevy_state")]
        app.add_plugins(bevy_state::app::StatesPlugin);
    }
}

/// An extension trait for [`App`] that provides methods for adding [`State`] types.
pub trait AppExtState {
    /// Register `S` without initializing it.
    fn register_state<S: RegisterState>(&mut self) -> &mut Self;

    /// Initialize `S` with empty next state.
    ///
    /// Calls [`S::Next::empty`](NextState::empty).
    fn add_state<S: RegisterState>(&mut self) -> &mut Self;

    /// Initialize `S` with default next state.
    fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self;

    /// Initialize `S` with specific next state.
    fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self;
}

fn state_exists<S: State>(world: &World) -> bool {
    let global = world.resource::<GlobalStatesEntity>().0;
    world.entity(global).contains::<TriggerStateFlush<S>>()
}

fn insert_state_helper<T: NextState<State: RegisterState>>(app: &mut App, next: Option<T>) {
    let global = app.world().resource::<GlobalStatesEntity>().0;
    app.world_mut().entity_mut(global).insert((
        next.unwrap_or_else(T::empty),
        TriggerStateFlush::<T::State>::default(),
    ));
}

impl AppExtState for App {
    fn register_state<S: RegisterState>(&mut self) -> &mut Self {
        if !state_exists::<S>(self.world()) {
            S::register_state(self);
        }
        self
    }

    fn add_state<S: RegisterState>(&mut self) -> &mut Self {
        if !state_exists::<S>(self.world()) {
            insert_state_helper(self, None::<S::Next>);
            S::register_state(self);
        }
        self
    }

    fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self {
        if !state_exists::<S>(self.world()) {
            let next = S::Next::from_world(self.world_mut());
            insert_state_helper(self, Some(next));
            S::register_state(self);
        }
        self
    }

    fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self {
        insert_state_helper(self, Some(next));
        if !state_exists::<T::State>(self.world()) {
            T::State::register_state(self);
        }
        self
    }
}

/// A [`State`] type that can be registered with an [`App`].
pub trait RegisterState: State {
    /// Register this state type with the app.
    ///
    /// The following plugins may be useful when implementing this method:
    ///
    /// - [`ResolveStatePlugin<Self>`]
    /// - [`DetectChangePlugin<Self>`]
    /// - [`FlushEventPlugin<Self>`]
    /// - [`LogFlushPlugin<Self>`](crate::extra::debug::LogFlushPlugin)
    /// - [`BevyStatePlugin<Self>`](crate::extra::bevy_state::BevyStatePlugin)
    /// - [`EntityScopePlugin<Self>`](crate::extra::entity_scope::EntityScopePlugin)
    /// - [`ApplyFlushPlugin<Self>`]
    fn register_state(app: &mut App);
}

/// A plugin that configures the [`StateHook<S>`] system sets for the [`State`] type `S`
/// in the [`StateFlush`] schedule.
///
/// To specify a dependency relative to another `State` type `T`, add
/// [`StateHook::<T>::Resolve`] to [`after`](Self::after) or [`before`](Self::before).
///
/// Calls [`schedule_resolve_state<S>`].
pub struct ResolveStatePlugin<S: State> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: State> Plugin for ResolveStatePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_resolve_state::<S>(
            app.get_schedule_mut(StateFlush).unwrap(),
            &self.after,
            &self.before,
        );
    }
}

impl<S: State> Default for ResolveStatePlugin<S> {
    fn default() -> Self {
        Self {
            after: Vec::new(),
            before: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S: State> ResolveStatePlugin<S> {
    /// Create a [`ResolveStatePlugin`] from `.after` and `.before` system sets.
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }

    /// Configure a `.after` system set.
    pub fn after<T: State>(mut self) -> Self {
        self.after.push(StateHook::<T>::Resolve.intern());
        self
    }

    /// Configure a `.before` system set.
    pub fn before<T: State>(mut self) -> Self {
        self.before.push(StateHook::<T>::Resolve.intern());
        self
    }
}

/// A plugin that adds change detection systems for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_detect_change<S>`].
pub struct DetectChangePlugin<S: State + Eq>(PhantomData<S>);

impl<S: State + Eq> Plugin for DetectChangePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State + Eq> Default for DetectChangePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// A plugin that adds a [`StateFlushEvent<S>`] sending system for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_flush_event<S>`].
pub struct FlushEventPlugin<S: State + Clone>(PhantomData<S>);

impl<S: State + Clone> Plugin for FlushEventPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_flush_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State + Clone> Default for FlushEventPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// A plugin that adds an apply flush system for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_apply_flush<S>`].
pub struct ApplyFlushPlugin<S: State + Clone>(PhantomData<S>);

impl<S: State + Clone> Plugin for ApplyFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State + Clone> Default for ApplyFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
