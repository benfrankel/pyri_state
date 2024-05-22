use std::marker::PhantomData;

use bevy_ecs::{
    all_tuples,
    schedule::{InternedSystemSet, Schedule},
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_on_flush, schedule_send_event_on_flush,
        schedule_trigger_flush_on_change,
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

pub struct StateConfigOnFlush<S: State>(pub Vec<InternedSystemSet>, pub PhantomData<S>);

impl<S: State> ConfigureState for StateConfigOnFlush<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_on_flush::<S>(schedule, &self.0);
    }
}

pub struct StateConfigTriggerFlushOnChange<S: State + Eq>(pub PhantomData<S>);

impl<S: State + Eq> ConfigureState for StateConfigTriggerFlushOnChange<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_trigger_flush_on_change::<S>(schedule);
    }
}

pub struct StateConfigSendEventOnFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigSendEventOnFlush<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_send_event_on_flush::<S>(schedule);
    }
}

pub struct StateConfigApplyFlush<S: State + Clone>(pub PhantomData<S>);

impl<S: State + Clone> ConfigureState for StateConfigApplyFlush<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_apply_flush::<S>(schedule);
    }
}
