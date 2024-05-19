use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{schedule::Schedules, world::FromWorld};

use crate::{
    schedule::{PostStateTransition, PreStateTransition, StateTransition},
    state::{CurrentState, NextState, State},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PreStateTransition)
            .init_schedule(StateTransition)
            .init_schedule(PostStateTransition);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, PreStateTransition);
        order.insert_after(PreStateTransition, StateTransition);
        order.insert_after(StateTransition, PostStateTransition);
    }
}

fn set_up_schedules<S: State>(app: &mut App) -> &mut App {
    let mut schedules = app.world.resource_mut::<Schedules>();

    PreStateTransition::register_state::<S>(schedules.get_mut(PreStateTransition).unwrap());
    StateTransition::register_state::<S>(schedules.get_mut(StateTransition).unwrap());
    PostStateTransition::register_state::<S>(schedules.get_mut(PostStateTransition).unwrap());

    app
}

pub trait AppStateExt {
    fn add_state<S: State>(&mut self) -> &mut Self;

    fn init_state_ext<S: State + FromWorld>(&mut self) -> &mut Self;

    fn insert_state_ext<S: State>(&mut self, value: S) -> &mut Self;
}

impl AppStateExt for App {
    fn add_state<S: State>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        set_up_schedules::<S>(self)
            .init_resource::<CurrentState<S>>()
            .init_resource::<NextState<S>>()
    }

    fn init_state_ext<S: State + FromWorld>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        let value = S::from_world(&mut self.world);

        set_up_schedules::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }

    fn insert_state_ext<S: State>(&mut self, value: S) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        set_up_schedules::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }
}
