use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::world::FromWorld;

use crate::{
    buffer::{CurrentState, NextState_},
    config::ConfigureState,
    schedule::StateFlush,
    state::State_,
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
    fn add_state_<S: State_>(&mut self) -> &mut Self;
    fn init_state_<S: State_ + FromWorld>(&mut self) -> &mut Self;
    fn insert_state_<S: State_>(&mut self, value: S) -> &mut Self;
}

impl AppExtAddState for App {
    fn add_state_<S: State_>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            self.init_resource::<CurrentState<S>>()
                .init_resource::<NextState_<S>>();
        }
        self
    }

    fn init_state_<S: State_ + FromWorld>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            let value = S::from_world(&mut self.world);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState_::present(value));
        }
        self
    }

    fn insert_state_<S: State_>(&mut self, value: S) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self.get_schedule_mut(StateFlush).unwrap());
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState_::present(value));
        }
        self
    }
}
