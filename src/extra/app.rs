//! State configuration tools.
//!
//! Enable the `bevy_app` feature flag to use this module.
//!
//! See the [derive macro](pyri_state_derive::State) for an easy way to implement
//! [`RegisterState`] and enable the plugins provided by this module.

use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{InternedSystemSet, SystemSet},
    world::FromWorld,
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_detect_change, schedule_flush_event,
        schedule_local_apply_flush, schedule_local_detect_change, schedule_local_flush_event,
        schedule_resolve_state, LocalStateFlushEvent, StateFlush, StateFlushEvent, StateHook,
    },
    state::{LocalState, NextState, State, TriggerStateFlush},
};

/// A plugin that performs the required setup for [`State`] types to function:
///
/// - Adds the [`StateFlush`] schedule to the [`MainScheduleOrder`] after [`PreUpdate`].
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

        // Add the `bevy_state` plugin.
        #[cfg(feature = "bevy_state")]
        app.add_plugins(bevy_state::app::StatesPlugin);
    }
}

/// An extension trait for [`App`] that provides methods for adding [`State`] types.
pub trait AppExtState {
    /// Register `S` without initializing it.
    fn register_state<S: RegisterState>(&mut self) -> &mut Self;

    /// Initialize `S` with an empty next state value.
    ///
    /// Calls [`S::Next::empty`](NextState::empty).
    fn add_state<S: RegisterState>(&mut self) -> &mut Self;

    /// Initialize `S` with a default next state value.
    fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self;

    /// Initialize `S` with a specific next state value.
    fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self;
}

fn state_exists<S: State>(app: &App) -> bool {
    app.world().contains_resource::<TriggerStateFlush<S>>()
}

fn insert_state<T: NextState<State: RegisterState>>(app: &mut App, next: Option<T>) {
    app.insert_resource(next.unwrap_or_else(T::empty))
        .init_resource::<TriggerStateFlush<T::State>>();
}

impl AppExtState for App {
    fn register_state<S: RegisterState>(&mut self) -> &mut Self {
        if !state_exists::<S>(self) {
            S::register_state(self);
        }
        self
    }

    fn add_state<S: RegisterState>(&mut self) -> &mut Self {
        if !state_exists::<S>(self) {
            insert_state(self, None::<S::Next>);
            S::register_state(self);
        }
        self
    }

    fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self {
        if !state_exists::<S>(self) {
            let next = S::Next::from_world(self.world_mut());
            insert_state(self, Some(next));
            S::register_state(self);
        }
        self
    }

    fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self {
        insert_state(self, Some(next));
        if !state_exists::<T::State>(self) {
            T::State::register_state(self);
        }
        self
    }
}

/// A [`State`] type that can be registered with an [`App`].
pub trait RegisterState: State {
    /// Register this state type with the app.
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

/// A plugin that adds a change detection system for the [`State`] type `S`
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

/// A plugin that adds a local change detection system for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_local_detect_change<S>`].
pub struct LocalDetectChangePlugin<S: LocalState + Eq>(PhantomData<S>);

impl<S: LocalState + Eq> Plugin for LocalDetectChangePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_local_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: LocalState + Eq> Default for LocalDetectChangePlugin<S> {
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

/// A plugin that adds a [`LocalStateFlushEvent<S>`] sending system for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_local_flush_event<S>`].
pub struct LocalFlushEventPlugin<S: State + Clone>(PhantomData<S>);

impl<S: LocalState + Clone> Plugin for LocalFlushEventPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<LocalStateFlushEvent<S>>();
        schedule_local_flush_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: LocalState + Clone> Default for LocalFlushEventPlugin<S> {
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

/// A plugin that adds a local apply flush system for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_local_apply_flush<S>`].
pub struct LocalApplyFlushPlugin<S: State + Clone>(PhantomData<S>);

impl<S: LocalState + Clone> Plugin for LocalApplyFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_local_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: LocalState + Clone> Default for LocalApplyFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
