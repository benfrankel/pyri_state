use std::marker::PhantomData;

use bevy_ecs::{
    all_tuples,
    schedule::{InternedSystemSet, Schedule},
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_detect_change, schedule_resolve_state, schedule_send_event,
    },
    state::State,
};

pub trait ConfigureState {
    fn configure(self, schedule: &mut Schedule);
}

macro_rules! impl_configure_state {
    ($(($param:ident, $value:ident)), *) => {
        impl<$($param: ConfigureState),*> ConfigureState for ($($param,)*) {
            fn configure(self, schedule: &mut Schedule) {
                let ($($value,)*) = self;
                $($value.configure(schedule);)*
                let _ = schedule;
            }
        }
    };
}

all_tuples!(impl_configure_state, 0, 4, T, t);

pub struct StateConfigResolveState<S: State>(Vec<InternedSystemSet>, PhantomData<S>);

impl<S: State> ConfigureState for StateConfigResolveState<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_resolve_state::<S>(schedule, &self.0);
    }
}

impl<S: State> StateConfigResolveState<S> {
    pub fn new() -> Self {
        Self(vec![], PhantomData)
    }

    pub fn after(states: Vec<InternedSystemSet>) -> Self {
        Self(states, PhantomData)
    }
}

#[derive(Default)]
pub struct StateConfigDetectChange<S: State + Eq>(PhantomData<S>);

impl<S: State + Eq> ConfigureState for StateConfigDetectChange<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_detect_change::<S>(schedule);
    }
}

impl<S: State + Eq> StateConfigDetectChange<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Default)]
pub struct StateConfigSendEvent<S: State + Clone>(PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigSendEvent<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_send_event::<S>(schedule);
    }
}

impl<S: State + Clone> StateConfigSendEvent<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Default)]
pub struct StateConfigApplyFlush<S: State + Clone>(PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigApplyFlush<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_apply_flush::<S>(schedule);
    }
}

impl<S: State + Clone> StateConfigApplyFlush<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
