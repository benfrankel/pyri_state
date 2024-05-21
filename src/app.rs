use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{schedule::Schedules, world::FromWorld};

use crate::{
    buffer::{CurrentState, NextState},
    prelude::StateFlushEvent,
    schedule::{PostStateFlush, PreStateFlush, StateFlush},
    state::State,
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PreStateFlush)
            .init_schedule(StateFlush)
            .init_schedule(PostStateFlush);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, PreStateFlush);
        order.insert_after(PreStateFlush, StateFlush);
        order.insert_after(StateFlush, PostStateFlush);
    }
}

fn set_up_schedules<S: State>(app: &mut App) -> &mut App {
    // TODO: Make this opt-out
    app.add_event::<StateFlushEvent<S>>();

    let mut schedules = app.world.resource_mut::<Schedules>();

    // TODO: ... is it possible to only call this if S also impls Eq?
    //PreStateFlush::register_state::<S>(schedules.get_mut(PreStateFlush).unwrap());
    StateFlush::register_state::<S>(schedules.get_mut(StateFlush).unwrap());
    PostStateFlush::register_state::<S>(schedules.get_mut(PostStateFlush).unwrap());

    app
}

pub trait AppStateExt {
    fn add_state<S: State>(&mut self) -> &mut Self;

    // TODO: Remove _ suffix in bevy 0.14
    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self;

    // TODO: Remove _ suffix in bevy 0.14
    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self;
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

    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        let value = S::from_world(&mut self.world);

        set_up_schedules::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::present(value))
    }

    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        set_up_schedules::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::present(value))
    }
}
