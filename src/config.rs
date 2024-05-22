use std::marker::PhantomData;

use bevy_ecs::{
    all_tuples,
    schedule::{InternedSystemSet, Schedule},
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_detect_change, schedule_resolve_state, schedule_send_event,
    },
    state::State_,
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

pub struct StateConfigResolveState<S: State_> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: State_> ConfigureState for StateConfigResolveState<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_resolve_state::<S>(schedule, &self.after, &self.before);
    }
}

impl<S: State_> StateConfigResolveState<S> {
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct StateConfigDetectChange<S: State_ + Eq>(PhantomData<S>);

impl<S: State_ + Eq> ConfigureState for StateConfigDetectChange<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_detect_change::<S>(schedule);
    }
}

impl<S: State_ + Eq> StateConfigDetectChange<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Default)]
pub struct StateConfigSendEvent<S: State_ + Clone>(PhantomData<S>);

impl<S: State_ + Clone> ConfigureState for StateConfigSendEvent<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_send_event::<S>(schedule);
    }
}

impl<S: State_ + Clone> StateConfigSendEvent<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Default)]
pub struct StateConfigApplyFlush<S: State_ + Clone>(PhantomData<S>);

impl<S: State_ + Clone> ConfigureState for StateConfigApplyFlush<S> {
    fn configure(self, schedule: &mut Schedule) {
        schedule_apply_flush::<S>(schedule);
    }
}

impl<S: State_ + Clone> StateConfigApplyFlush<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
