use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{
        InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs, NextState, Schedule,
        ScheduleLabel, SystemSet,
    },
    system::{Res, ResMut},
};

use crate::{
    buffer::{BevyState, CurrentState, NextState_, StateRef},
    state::RawState,
};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

// Provides system ordering for state flush handling systems.
#[derive(SystemSet)]
pub enum StateFlushSet<S: RawState> {
    Resolve,
    Trigger,
    Flush,
    Exit,
    Transition,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: RawState> Clone for StateFlushSet<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolve => Self::Resolve,
            Self::Trigger => Self::Trigger,
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Transition => Self::Transition,
            Self::Enter => Self::Enter,
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

impl<S: RawState> PartialEq for StateFlushSet<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: RawState> Eq for StateFlushSet<S> {}

impl<S: RawState> Hash for StateFlushSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: RawState> Debug for StateFlushSet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve => write!(f, "Resolve"),
            Self::Trigger => write!(f, "Trigger"),
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Transition => write!(f, "Transition"),
            Self::Enter => write!(f, "Enter"),
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

#[derive(SystemSet, Clone, Hash, PartialEq, Eq, Debug)]
struct ApplyFlushSet;

#[derive(Event)]
pub struct StateFlushEvent<S: RawState> {
    pub before: Option<S>,
    pub after: Option<S>,
}

fn check_flush_flag<S: RawState>(state: Res<NextState_<S>>) -> bool {
    state.flush
}

fn send_flush_event<S: RawState + Clone>(
    state: StateRef<S>,
    mut events: EventWriter<StateFlushEvent<S>>,
) {
    events.send(StateFlushEvent {
        before: state.current.inner.clone(),
        after: state.next.inner.clone(),
    });
}

fn apply_flush<S: RawState + Clone>(
    mut current: ResMut<CurrentState<S>>,
    next: Res<NextState_<S>>,
) {
    current.inner.clone_from(&next.inner);
}

pub fn schedule_detect_change<S: RawState + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::set_flush(true)
            .run_if(|state: StateRef<S>| matches!(state.get(), (x, y) if x != y))
            .in_set(StateFlushSet::<S>::Trigger),
    );
}

pub fn schedule_resolve_state<S: RawState>(
    schedule: &mut Schedule,
    after: &[InternedSystemSet],
    before: &[InternedSystemSet],
) {
    // External ordering
    for &system_set in after {
        schedule.configure_sets(StateFlushSet::<S>::Resolve.after(system_set));
    }
    for &system_set in before {
        schedule.configure_sets(StateFlushSet::<S>::Resolve.before(system_set));
    }

    // Internal ordering
    schedule.configure_sets((
        StateFlushSet::<S>::Resolve.before(ApplyFlushSet),
        (
            StateFlushSet::<S>::Trigger,
            StateFlushSet::<S>::Flush.run_if(check_flush_flag::<S>),
        )
            .chain()
            .in_set(StateFlushSet::<S>::Resolve),
        (
            StateFlushSet::<S>::Exit,
            StateFlushSet::<S>::Transition,
            StateFlushSet::<S>::Enter,
        )
            .chain()
            .in_set(StateFlushSet::<S>::Flush),
    ));
}

pub fn schedule_send_event<S: RawState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(S::on_flush(send_flush_event::<S>));
}

pub fn schedule_apply_flush<S: RawState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::set_flush(false))
            .run_if(check_flush_flag::<S>)
            .in_set(ApplyFlushSet),
    );
}

pub fn schedule_bevy_state<S: RawState + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let update_bevy_state =
        |pyri_state: Res<NextState_<S>>, mut bevy_state: ResMut<NextState<BevyState<S>>>| {
            if bevy_state.0.is_none() {
                bevy_state.set(BevyState(pyri_state.get().cloned()));
            }
        };

    let update_pyri_state = |mut pyri_state: ResMut<NextState_<S>>,
                             bevy_state: Res<NextState<BevyState<S>>>| {
        if let Some(value) = bevy_state.0.clone() {
            pyri_state.set_flush(true).inner = value.0;
        }
    };

    schedule.add_systems((
        update_pyri_state.in_set(StateFlushSet::<S>::Trigger),
        S::on_flush(update_bevy_state),
    ));
}
