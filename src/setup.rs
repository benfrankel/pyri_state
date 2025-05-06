//! State setup and configuration tools.
//!
//! See the [derive macro](pyri_state_derive::State) for an easy way to implement
//! [`RegisterState`].

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
    use tiny_bail::prelude::*;

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that performs the required setup for [`State`] types to function:
    ///
    /// - Adds the [`StateFlush`] schedule to the [`MainScheduleOrder`] before [`PreUpdate`].
    /// - Adds the [`bevy_state` plugin](bevy_state::app::StatesPlugin) if the
    ///   `bevy_state` feature is enabled.
    pub struct StatePlugin;

    impl Plugin for StatePlugin {
        fn build(&self, app: &mut App) {
            // Add the `bevy_state` plugin.
            #[cfg(feature = "bevy_state")]
            app.add_plugins(bevy_state::app::StatesPlugin);

            // Add the `StateFlush` schedule.
            r!(app
                .init_schedule(StateFlush)
                .world_mut()
                .get_resource_mut::<MainScheduleOrder>())
            .insert_before(PreUpdate, StateFlush);
        }
    }

    /// An extension trait for [`App`] that provides methods for adding [`State`] types.
    pub trait AppExtState {
        /// Register a `State` type without initializing it.
        fn register_state<S: RegisterState>(&mut self) -> &mut Self;

        /// Initialize a `State` type with an empty `NextState`.
        ///
        /// Calls [`S::Next::empty`](NextState::empty).
        fn add_state<S: RegisterState>(&mut self) -> &mut Self;

        /// Initialize a `State` type with a default `NextState`.
        fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self;

        /// Initialize a `State` type with a specific `NextState`.
        fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self;
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
                insert_state(self.world_mut(), None::<S::Next>);
                S::register_state(self);
            }
            self
        }

        fn init_state<S: RegisterState<Next: FromWorld>>(&mut self) -> &mut Self {
            if !state_exists::<S>(self.world()) {
                let next = S::Next::from_world(self.world_mut());
                insert_state(self.world_mut(), Some(next));
                S::register_state(self);
            }
            self
        }

        fn insert_state<T: NextState<State: RegisterState>>(&mut self, next: T) -> &mut Self {
            insert_state(self.world_mut(), Some(next));
            if !state_exists::<T::State>(self.world()) {
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
}

use bevy_ecs::{
    system::{Commands, EntityCommands},
    world::{EntityWorldMut, FromWorld, World},
};

use crate::{
    next_state::{NextState, TriggerStateFlush},
    prelude::State,
    state::LocalState,
};

fn state_exists<S: State>(world: &World) -> bool {
    world.contains_resource::<TriggerStateFlush<S>>()
}

fn insert_state<Next: NextState>(world: &mut World, next: Option<Next>) {
    world.insert_resource(next.unwrap_or_else(Next::empty));
    world.init_resource::<TriggerStateFlush<Next::State>>();
}

/// An extension trait for [`Commands`] that provides methods for adding [`State`] types.
pub trait CommandsExtState {
    /// Queue a command to initialize a `State` type with an empty `NextState`.
    ///
    /// Calls [`S::Next::empty`](NextState::empty).
    fn add_state<S: State>(&mut self);

    /// Queue a command to initialize a `State` type with a default `NextState`.
    fn init_state<S: State<Next: FromWorld>>(&mut self);

    /// Queue a command to initialize a `State` type with a specific `NextState`.
    fn insert_state<T: NextState>(&mut self, next: T);
}

impl CommandsExtState for Commands<'_, '_> {
    fn add_state<S: State>(&mut self) {
        self.queue(|world: &mut World| {
            if !state_exists::<S>(world) {
                insert_state(world, None::<S::Next>);
            }
        });
    }

    fn init_state<S: State<Next: FromWorld>>(&mut self) {
        self.queue(|world: &mut World| {
            if !state_exists::<S>(world) {
                let next = S::Next::from_world(world);
                insert_state(world, Some(next));
            }
        });
    }

    fn insert_state<T: NextState>(&mut self, next: T) {
        self.queue(|world: &mut World| insert_state(world, Some(next)));
    }
}

fn local_state_exists<S: LocalState>(entity: &EntityWorldMut) -> bool {
    entity.contains::<TriggerStateFlush<S>>()
}

fn insert_local_state<Next: NextState<State: LocalState<Next = Next>>>(
    entity: &mut EntityWorldMut,
    next: Option<Next>,
) {
    entity.insert((
        next.unwrap_or_else(Next::empty),
        TriggerStateFlush::<Next::State>::default(),
    ));
}

/// An extension trait for [`EntityCommands`] that provides methods for adding
/// [`LocalState`] types.
pub trait EntityCommandsExtState {
    /// Queue a command to initialize a `LocalState` type with an empty `NextState`.
    ///
    /// Calls [`S::Next::empty`](NextState::empty).
    fn add_state<S: LocalState>(&mut self);

    /// Queue a command to initialize a `LocalState` type with a default `NextState`.
    fn init_state<S: LocalState<Next: FromWorld>>(&mut self);

    /// Queue a command to initialize a `LocalState` type with a specific `NextState`.
    fn insert_state<T: NextState<State: LocalState<Next = T>>>(&mut self, next: T);
}

impl EntityCommandsExtState for EntityCommands<'_> {
    fn add_state<S: LocalState>(&mut self) {
        self.queue(|mut entity: EntityWorldMut| {
            if !local_state_exists::<S>(&entity) {
                insert_local_state(&mut entity, None::<S::Next>);
            }
        });
    }

    fn init_state<S: LocalState<Next: FromWorld>>(&mut self) {
        self.queue(|mut entity: EntityWorldMut| {
            if !local_state_exists::<S>(&entity) {
                let next = entity.world_scope(S::Next::from_world);
                insert_local_state(&mut entity, Some(next));
            }
        });
    }

    fn insert_state<T: NextState<State: LocalState<Next = T>>>(&mut self, next: T) {
        self.queue(|mut entity: EntityWorldMut| insert_local_state(&mut entity, Some(next)));
    }
}
