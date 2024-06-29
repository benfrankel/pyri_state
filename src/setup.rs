//! State setup and configuration tools.
//!
//! See the [derive macro](pyri_state_derive::State) for an easy way to implement
//! [`RegisterState`].

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
    use bevy_ecs::world::FromWorld;

    use crate::{
        next_state::{NextState, TriggerStateFlush},
        schedule::StateFlush,
        state::State,
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
}
