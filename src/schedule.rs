use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::Event,
    schedule::{
        InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs, Schedule, ScheduleLabel,
        SystemSet,
    },
};

use crate::state::{State, StateExtClone, StateExtEq};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PreStateFlush;

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PostStateFlush;

// Used for system ordering relative to other states, and only runs on flush.
#[derive(SystemSet, Default)]
pub enum OnState<S: State> {
    #[default]
    Flush,
    Exit,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State> Clone for OnState<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Enter => Self::Enter,
            _ => unreachable!(),
        }
    }
}

impl<S: State> PartialEq for OnState<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State> Eq for OnState<S> {}

impl<S: State> Hash for OnState<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State> Debug for OnState<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Enter => write!(f, "Enter"),
            _ => unreachable!(),
        }
    }
}

#[derive(Event)]
pub struct StateFlushEvent<S: State> {
    pub before: Option<S>,
    pub after: Option<S>,
}

// TODO: Consider moving this up to immediately before OnState::Flush so that
// last-minute changes to the state (e.g. from handling source state changes) will be detected
pub fn schedule_set_flush_on_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(S::set_flush(true).run_if(S::will_any_change));
}

pub fn schedule_on_flush<S: State>(schedule: &mut Schedule, after: &[InternedSystemSet]) {
    // Internal ordering
    schedule.configure_sets((
        OnState::<S>::Flush.run_if(S::will_any_flush),
        (OnState::<S>::Exit, OnState::<S>::Enter)
            .chain()
            .in_set(OnState::<S>::Flush),
    ));

    // External ordering
    for &system_set in after {
        schedule.configure_sets(OnState::<S>::Flush.after(system_set));
    }
}

pub fn schedule_send_event_on_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(S::on_any_flush(S::send_flush_event));
}

pub fn schedule_apply_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems((S::apply_flush, S::set_flush(false)).run_if(S::will_any_flush));
}
