use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{IntoSystemConfigs, IntoSystemSetConfigs, Schedule, ScheduleLabel, SystemSet},
    system::{Res, ResMut},
};

use crate::state::{CurrentState, NextState, State, StateExtEq, StateRef};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PreStateFlush;

impl PreStateFlush {
    pub fn register_state<S: State + Eq>(schedule: &mut Schedule) {
        // TODO: Make "flush on any change" opt-out
        schedule.add_systems(S::flush(true).run_if(S::will_any_change));
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

impl StateFlush {
    // TODO: Configure any declared state dependencies
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        schedule.configure_sets((
            OnState::<S>::Flush.run_if(S::will_any_flush),
            (OnState::<S>::Exit, OnState::<S>::Enter)
                .chain()
                .in_set(OnState::<S>::Flush),
        ));
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PostStateFlush;

impl PostStateFlush {
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        // TODO: Make "send flush event" opt-out
        schedule.add_systems(
            (
                (send_flush_event::<S>, apply_flush::<S>).chain(),
                S::flush(false),
            )
                .run_if(S::will_any_flush),
        );
    }
}

// Used for system ordering relative to other states, and only runs on flush.
#[derive(SystemSet, Clone, Default)]
pub enum OnState<S> {
    #[default]
    Flush,
    Exit,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S> PartialEq for OnState<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S> Eq for OnState<S> {}

impl<S> Hash for OnState<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S> Debug for OnState<S> {
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

fn send_flush_event<S: State>(state: StateRef<S>, mut events: EventWriter<StateFlushEvent<S>>) {
    events.send(StateFlushEvent {
        before: state.current.inner.clone(),
        after: state.next.inner.clone(),
    });
}

fn apply_flush<S: State>(mut current: ResMut<CurrentState<S>>, next: Res<NextState<S>>) {
    current.inner.clone_from(&next.inner);
}
