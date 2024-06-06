//! TODO: Module-level documentation

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
        schedule_apply_flush, schedule_bevy_state, schedule_detect_change, schedule_resolve_state,
        schedule_send_event, StateFlush, StateFlushEvent, StateFlushSet,
    },
    state::{BevyState, CurrentState, StateMut, State_},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(StateFlush)
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);
    }
}

pub trait AppExtState {
    fn add_state_<S: AddState>(&mut self) -> &mut Self;

    fn init_state_<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld;

    fn insert_state_<T: AddStateStorage>(&mut self, storage: T) -> &mut Self;
}

impl AppExtState for App {
    fn add_state_<S: AddState>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::AddStorage::add_state_storage(self, None);
            S::add_state(self);
        }
        self
    }

    fn init_state_<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld,
    {
        if !self.world.contains_resource::<CurrentState<S>>() {
            let storage = S::AddStorage::from_world(&mut self.world);
            S::AddStorage::add_state_storage(self, Some(storage));
            S::add_state(self);
        }
        self
    }

    fn insert_state_<T: AddStateStorage>(&mut self, storage: T) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<T::AddState>>() {
            T::add_state_storage(self, Some(storage));
            T::AddState::add_state(self);
        }
        self
    }
}

pub trait AddStateStorage: Sized {
    type AddState: AddState;

    fn add_state_storage(app: &mut App, storage: Option<Self>);
}

/// TODO
///
/// ```rust
/// // Clone + PartialEq + Eq are required by the derive macro by default.
/// #[derive(State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// #[derive(State)]
/// #[state(no_defaults)]
/// struct RawState;
///
/// // The following options are provided by the derive macro:
/// #[derive(State, Clone, PartialEq, Eq, Hash, Debug)]
/// #[state(
///     // Disable default plugins: detect_change, flush_event, apply_flush.
///     no_defaults,
///     // Trigger a flush on any state change (requires PartialEq, Eq).
///     detect_change,
///     // Send an event on flush (requires Clone).
///     flush_event,
///     // Log on exit, transition, and enter (requires Debug).
///     log_flush,
///     // Include a `BevyState<Self>` wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
///     bevy_state,
///     // Clone the next state into the current state on flush (requires Clone).
///     apply_flush,
///     // Swap out the default `StateBuffer<Self>` for a custom storage type.
///     storage(StateStack<Self>),
///     // Run this state's on flush systems after resolving the listed states.
///     after(GameState),
///     // Run this state's on flush systems before resolving the listed states.
///     before(RawState),
/// )]
/// struct ConfiguredState;
/// ```
pub trait AddState: State_ {
    type AddStorage: AddStateStorage;

    fn add_state(app: &mut App);
}

pub struct ResolveStatePlugin<S: State_> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: State_> Plugin for ResolveStatePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_resolve_state::<S>(
            app.get_schedule_mut(StateFlush).unwrap(),
            &self.after,
            &self.before,
        );
    }
}

impl<S: State_> Default for ResolveStatePlugin<S> {
    fn default() -> Self {
        Self {
            after: Vec::new(),
            before: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S: State_> ResolveStatePlugin<S> {
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }

    pub fn after<T: State_>(mut self) -> Self {
        self.after.push(StateFlushSet::<T>::Resolve.intern());
        self
    }

    pub fn before<T: State_>(mut self) -> Self {
        self.before.push(StateFlushSet::<T>::Resolve.intern());
        self
    }
}

pub struct DetectChangePlugin<S: State_ + Eq>(PhantomData<S>);

impl<S: State_ + Eq> Plugin for DetectChangePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State_ + Eq> Default for DetectChangePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct FlushEventPlugin<S: State_ + Clone>(PhantomData<S>);

impl<S: State_ + Clone> Plugin for FlushEventPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_send_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State_ + Clone> Default for FlushEventPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct BevyStatePlugin<S: StateMut + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_state::<BevyState<S>>();
        schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct ApplyFlushPlugin<S: State_ + Clone>(PhantomData<S>);

impl<S: State_ + Clone> Plugin for ApplyFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State_ + Clone> Default for ApplyFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
