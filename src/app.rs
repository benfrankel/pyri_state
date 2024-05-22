use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::world::FromWorld;

use crate::{
    buffer::{CurrentState, NextState},
    config::ConfigureState,
    schedule::StateFlush,
    state::State,
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

pub trait AppExtAddState {
    fn add_state_<S: State>(&mut self) -> &mut Self;
    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self;
    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self;
}

impl AppExtAddState for App {
    fn add_state_<S: State>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            self.init_resource::<CurrentState<S>>()
                .init_resource::<NextState<S>>();
        }
        self
    }

    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            let value = S::from_world(&mut self.world);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState::present(value));
        }
        self
    }

    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState::present(value));
        }
        self
    }
}
