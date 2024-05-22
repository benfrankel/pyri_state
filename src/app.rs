use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{all_tuples, schedule::InternedSystemSet, world::FromWorld};

use crate::{
    buffer::{CurrentState, NextState},
    schedule::{
        schedule_apply_flush, schedule_on_flush, schedule_send_event_on_flush,
        schedule_set_flush_on_change, PostStateFlush, PreStateFlush, StateFlush,
    },
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

pub trait AppExtAddState {
    fn add_state_<S: State>(&mut self) -> &mut Self;
    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self;
    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self;
}

impl AppExtAddState for App {
    fn add_state_<S: State>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self);
            self.init_resource::<CurrentState<S>>()
                .init_resource::<NextState<S>>();
        }
        self
    }

    fn init_state_<S: State + FromWorld>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self);
            let value = S::from_world(&mut self.world);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState::present(value));
        }
        self
    }

    fn insert_state_<S: State>(&mut self, value: S) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::config().configure(self);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState::present(value));
        }
        self
    }
}

pub trait ConfigureState {
    fn configure(self, app: &mut App);
}

macro_rules! impl_configure_state {
    ($(($param:ident, $value:ident)), *) => {
        impl<$($param: ConfigureState),*> ConfigureState for ($($param,)*) {
            fn configure(self, app: &mut App) {
                let ($($value,)*) = self;
                $($value.configure(app);)*
                let _ = app;
            }
        }
    };
}

all_tuples!(impl_configure_state, 0, 4, T, t);

pub struct StateConfigOnFlush<S: State>(pub Vec<InternedSystemSet>, pub PhantomData<S>);

impl<S: State> ConfigureState for StateConfigOnFlush<S> {
    fn configure(self, app: &mut App) {
        schedule_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap(), &self.0);
    }
}

pub struct StateConfigSetFlushOnChange<S: State + Eq>(pub PhantomData<S>);

impl<S: State + Eq> ConfigureState for StateConfigSetFlushOnChange<S> {
    fn configure(self, app: &mut App) {
        schedule_set_flush_on_change::<S>(app.get_schedule_mut(PreStateFlush).unwrap());
    }
}

pub struct StateConfigSendEventOnFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigSendEventOnFlush<S> {
    fn configure(self, app: &mut App) {
        schedule_send_event_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

pub struct StateConfigApplyFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigApplyFlush<S> {
    fn configure(self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(PostStateFlush).unwrap());
    }
}
