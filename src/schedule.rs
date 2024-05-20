use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{IntoSystemConfigs, IntoSystemSetConfigs, Schedule, ScheduleLabel, SystemSet},
    system::{Res, ResMut},
};

use crate::{
    state::{CurrentState, NextState, State, StateRef},
    systems::{
        clear_flush_state, flush_state, state_is_present, state_will_flush, state_will_mutate,
        state_would_be_present,
    },
};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PreStateFlush;

impl PreStateFlush {
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        // TODO: Make this opt-out
        schedule.add_systems(flush_state::<S>.run_if(state_will_mutate::<S>));
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

impl StateFlush {
    // TODO: Configure the declared state dependencies
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        schedule.configure_sets((
            OnFlush::<S>::Any.run_if(state_will_flush::<S>),
            (
                OnFlush::<S>::Exit.run_if(state_is_present::<S>),
                OnFlush::<S>::Enter.run_if(state_would_be_present::<S>),
            )
                .chain()
                .in_set(OnFlush::<S>::Any),
        ));
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PostStateFlush;

impl PostStateFlush {
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        // TODO: Make send_flush_event opt-out
        schedule.add_systems(
            ((
                (send_flush_event::<S>, apply_flush::<S>).chain(),
                clear_flush_state::<S>,
            ),)
                .run_if(state_will_flush::<S>),
        );
    }
}

#[derive(SystemSet, Clone, PartialEq, Eq, Default)]
pub enum OnFlush<S> {
    #[default]
    Any,
    Exit,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S> Hash for OnFlush<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S> Debug for OnFlush<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "Any"),
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
