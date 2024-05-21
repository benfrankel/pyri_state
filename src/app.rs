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
    fn add_state<S: State>(&mut self) -> &mut Self;
}

impl AppExtAddState for App {
    fn add_state<S: State>(&mut self) -> &mut Self {
        S::config().configure_inner(self);
        self
    }
}

pub trait ConfigureState: Sized {
    type Target: State;

    fn configure_inner(self, app: &mut App);

    fn configure(self, app: &mut App) {
        if !app.world.contains_resource::<CurrentState<Self::Target>>() {
            self.configure_inner(app);
        }
    }
}

macro_rules! impl_configure_state {
    ($(($param:ident, $value:ident)), *) => {
        impl<$($param: ConfigureState),*> ConfigureState for ($($param,)*) {
            type Target = T0::Target;

            fn configure_inner(self, app: &mut App) {
                let ($($value,)*) = self;
                $($value.configure(app);)*
            }
        }
    };
}

all_tuples!(impl_configure_state, 1, 5, T, t);

pub struct StateConfigAdd<S: State>(pub PhantomData<S>);

impl<S: State> ConfigureState for StateConfigAdd<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        app.init_resource::<CurrentState<S>>()
            .init_resource::<NextState<S>>();
    }
}

pub struct StateConfigInsert<S: State>(pub S);

impl<S: State> ConfigureState for StateConfigInsert<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        app.init_resource::<CurrentState<S>>()
            .insert_resource(NextState::present(self.0));
    }
}

pub struct StateConfigInit<S: State + FromWorld>(pub PhantomData<S>);

impl<S: State + FromWorld> ConfigureState for StateConfigInit<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        let value = S::from_world(&mut app.world);
        app.init_resource::<CurrentState<S>>()
            .insert_resource(NextState::present(value));
    }
}

pub struct StateConfigOnFlush<S: State>(pub Vec<InternedSystemSet>, pub PhantomData<S>);

impl<S: State> ConfigureState for StateConfigOnFlush<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        schedule_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap(), &self.0);
    }
}

pub struct StateConfigSetFlushOnChange<S: State + Eq>(pub PhantomData<S>);

impl<S: State + Eq> ConfigureState for StateConfigSetFlushOnChange<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        schedule_set_flush_on_change::<S>(app.get_schedule_mut(PreStateFlush).unwrap());
    }
}

pub struct StateConfigSendEventOnFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigSendEventOnFlush<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        schedule_send_event_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

pub struct StateConfigApplyFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigApplyFlush<S> {
    type Target = S;

    fn configure_inner(self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(PostStateFlush).unwrap());
    }
}
