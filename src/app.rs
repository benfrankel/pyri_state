use bevy_app::{App, Plugin};
use bevy_ecs::{schedule::IntoSystemSetConfigs, world::FromWorld};

use crate::{
    conditions::{state_is_present, state_will_be_present, state_will_flush},
    schedule::{HandleTrans, OnTrans},
    state::{CurrentState, NextState, State},
};

fn configure_system_sets<S: State>(app: &mut App) -> &mut App {
    app.configure_sets(
        OnTrans,
        (
            HandleTrans::<S>::Any.run_if(state_will_flush::<S>),
            (
                HandleTrans::<S>::Exit.run_if(state_is_present::<S>),
                HandleTrans::<S>::Enter.run_if(state_will_be_present::<S>),
            )
                .chain()
                .in_set(HandleTrans::<S>::Any),
        ),
    )
}

// TODO: add_state_with_settings, init_state_with_settings, insert_state_with_settings
pub trait AppStateExt {
    fn add_state<S: State>(&mut self) -> &mut Self;

    fn init_state<S: State + FromWorld>(&mut self) -> &mut Self;

    fn insert_state<S: State>(&mut self, value: S) -> &mut Self;
}

impl AppStateExt for App {
    fn add_state<S: State>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        configure_system_sets::<S>(self)
            .init_resource::<CurrentState<S>>()
            .init_resource::<NextState<S>>()
    }

    fn init_state<S: State + FromWorld>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        let value = S::from_world(&mut self.world);

        configure_system_sets::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }

    fn insert_state<S: State>(&mut self, value: S) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        configure_system_sets::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, _app: &mut App) {
        // TODO: An opt-out system that checks if state_will_change.or_else(not(state_will_remain_present)), and if so, sets the flush flag.
        // TODO: A system that (after OnTrans schedule) flushes the next state into the current state, then resets the flush flag.
        todo!()
    }
}
