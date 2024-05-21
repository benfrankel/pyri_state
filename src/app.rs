use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};

use crate::{
    config::ConfigureState,
    prelude::State,
    schedule::{PostStateFlush, PreStateFlush, StateFlush},
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

pub trait AppStateExt {
    fn add_state<S: State>(&mut self) -> &mut Self;
}

impl AppStateExt for App {
    fn add_state<S: State>(&mut self) -> &mut Self {
        S::config().configure(self);
        self
    }
}
