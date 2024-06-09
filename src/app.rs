//! State configuration tools (behind the `bevy_app` feature flag).

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{InternedSystemSet, SystemSet},
    world::FromWorld,
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_bevy_state, schedule_detect_change, schedule_flush_event,
        schedule_resolve_state, StateFlush, StateFlushEvent, StateFlushSet,
    },
    state::{BevyState, CurrentState, State, StateMut},
};

/// A plugin that adds the [`StateFlush`] schedule to the [`MainScheduleOrder`].
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(StateFlush)
            .world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);
    }
}

/// An extension trait for [`App`] that provides methods for adding [`State`] types.
pub trait AppExtState {
    /// Initialize `S` with empty storage.
    fn add_state<S: AddState>(&mut self) -> &mut Self;

    /// Initialize `S` with default storage.
    fn init_state<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld;

    /// Initialize `S` with specific storage.
    fn insert_state<T: AddStateStorage>(&mut self, storage: T) -> &mut Self;
}

impl AppExtState for App {
    fn add_state<S: AddState>(&mut self) -> &mut Self {
        if !self.world().contains_resource::<CurrentState<S>>() {
            S::AddStorage::add_state_storage(self, None);
            S::add_state(self);
        }
        self
    }

    fn init_state<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld,
    {
        if !self.world().contains_resource::<CurrentState<S>>() {
            let storage = S::AddStorage::from_world(self.world_mut());
            S::AddStorage::add_state_storage(self, Some(storage));
            S::add_state(self);
        }
        self
    }

    fn insert_state<T: AddStateStorage>(&mut self, storage: T) -> &mut Self {
        if !self
            .world()
            .contains_resource::<CurrentState<T::AddState>>()
        {
            T::add_state_storage(self, Some(storage));
            T::AddState::add_state(self);
        }
        self
    }
}

/// A data type that can add a [`StateStorage`](crate::storage::StateStorage) to an [`App`].
pub trait AddStateStorage: Sized {
    /// The [`State`] type stored in the `StateStorage`.
    type AddState: AddState;

    /// Add the state storage, or empty storage if `None`.
    fn add_state_storage(app: &mut App, storage: Option<Self>);
}

/// A [`State`] type that can be added to an [`App`].
pub trait AddState: State {
    /// An [`AddStateStorage`] for this state type's
    /// [`StateStorage`](crate::storage::StateStorage).
    type AddStorage: AddStateStorage;

    /// Add this state type to the app.
    ///
    /// The following plugins may be useful when implementing this method:
    ///
    /// - [`ResolveStatePlugin<Self>`]
    /// - [`DetectChangePlugin<Self>`]
    /// - [`FlushEventPlugin<Self>`]
    /// - [`LogFlushPlugin<Self>`](crate::debug::LogFlushPlugin)
    /// - [`BevyStatePlugin<Self>`]
    /// - [`ApplyFlushPlugin<Self>`]
    fn add_state(app: &mut App);
}

/// A plugin that configures the [`StateFlushSet<S>`] system sets for the [`State`] type `S`
/// in the [`StateFlush`] schedule.
///
/// To specify a dependency relative to another `State` type `T`, add
/// [`StateFlushSet::<T>::Resolve`] to [`after`](Self::after) or [`before`](Self::before).
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
        self.after.push(StateFlushSet::<T>::Resolve.intern());
        self
    }

    /// Configure a `.before` system set.
    pub fn before<T: State>(mut self) -> Self {
        self.before.push(StateFlushSet::<T>::Resolve.intern());
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

/// A plugin that adds [`BevyState<S>`] propagation systems for the [`State`] type `S`
/// to the [`StateFlush`] schedule.
///
/// Calls [`schedule_bevy_state<S>`].
#[cfg(feature = "bevy_state")]
pub struct BevyStatePlugin<S: StateMut + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

#[cfg(feature = "bevy_state")]
impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
    fn build(&self, app: &mut App) {
        bevy_state::app::AppExtStates::init_state::<BevyState<S>>(app);
        schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_state")]
impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
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
