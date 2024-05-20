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
        // TODO: Make flush on change opt-out
        schedule.add_systems(S::flush(true).run_if(S::will_change));
        schedule.add_systems(
            (|mut x: ResMut<NextState<S>>| {
                x.flush = true;
            })
            .run_if(S::will_change),
        );
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

impl StateFlush {
    // TODO: Configure the declared state dependencies
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        schedule.configure_sets((
            S::on_flush().run_if(|x: Res<NextState<S>>| x.flush),
            (
                S::on_exit().run_if(|x: Res<CurrentState<S>>| x.is_present()),
                S::on_transition().run_if(|x: StateRef<S>| matches!(x.get(), (Some(_), Some(_)))),
                S::on_enter().run_if(|x: Res<NextState<S>>| x.will_be_present()),
            )
                .chain()
                .in_set(S::on_flush()),
        ));
    }
}

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct PostStateFlush;

impl PostStateFlush {
    pub fn register_state<S: State>(schedule: &mut Schedule) {
        // TODO: Make flush event opt-out
        schedule.add_systems(
            (
                (send_flush_event::<S>, apply_flush::<S>).chain(),
                S::flush(false),
            )
                .run_if(|x: Res<NextState<S>>| x.flush),
        );
    }
}

#[derive(SystemSet, Clone, Default)]
pub enum OnState<S> {
    #[default]
    Flush,
    Exit,
    Transition,
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
            Self::Transition => write!(f, "Transition"),
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
