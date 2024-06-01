use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{
        common_conditions::not, InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs,
        NextState, Schedule, ScheduleLabel, SystemSet,
    },
    system::{Res, ResMut},
};

use crate::state::{
    BevyState, CurrentState, GetState, NextStateMut, NextStateRef, RawState, SetState,
    StateFlushRef, TriggerStateFlush,
};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

// Provides system ordering for state flush handling systems.
#[derive(SystemSet)]
pub enum StateFlushSet<S: RawState> {
    Resolve,
    Compute,
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
            Self::Compute => Self::Compute,
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
            Self::Compute => write!(f, "Compute"),
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
    pub old: Option<S>,
    pub new: Option<S>,
}

fn was_triggered<S: RawState>(trigger: Res<TriggerStateFlush<S>>) -> bool {
    trigger.0
}

fn send_flush_event<S: GetState + Clone>(
    state: StateFlushRef<S>,
    mut events: EventWriter<StateFlushEvent<S>>,
) {
    let (old, new) = state.get();
    events.send(StateFlushEvent {
        old: old.cloned(),
        new: new.cloned(),
    });
}

fn apply_flush<S: GetState + Clone>(mut current: ResMut<CurrentState<S>>, next: NextStateRef<S>) {
    current.0 = next.get().cloned();
}

pub fn schedule_detect_change<S: GetState + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(|state: StateFlushRef<S>| matches!(state.get(), (x, y) if x != y))
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
            StateFlushSet::<S>::Compute,
            StateFlushSet::<S>::Trigger.run_if(not(was_triggered::<S>)),
            StateFlushSet::<S>::Flush.run_if(was_triggered::<S>),
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

pub fn schedule_send_event<S: GetState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(S::on_flush(send_flush_event::<S>));
}

#[cfg(feature = "debug")]
pub fn schedule_log_flush<S: GetState + Debug>(schedule: &mut Schedule) {
    use bevy_core::FrameCount;
    use bevy_ecs::schedule::Condition;
    use bevy_log::info;

    use crate::pattern::{StatePattern, StatePatternExtGet};

    fn log_state_exit<S: GetState + Debug>(frame: Res<FrameCount>, old: Res<CurrentState<S>>) {
        let frame = frame.0;
        let old = old.unwrap();
        info!("[Frame {frame}] Exit: {old:?}");
    }

    fn log_state_transition<S: GetState + Debug>(frame: Res<FrameCount>, state: StateFlushRef<S>) {
        let frame = frame.0;
        let (old, new) = state.unwrap();
        info!("[Frame {frame}] Transition: {old:?} -> {new:?}");
    }

    fn log_state_enter<S: GetState + Debug>(frame: Res<FrameCount>, new: NextStateRef<S>) {
        let frame = frame.0;
        let new = new.unwrap();
        info!("[Frame {frame}] Enter: {new:?}");
    }

    schedule.add_systems((
        S::ANY.on_exit(log_state_exit::<S>),
        // TODO: The story for flush / transition handling is not great right now.
        S::on_transition(
            log_state_transition::<S>.run_if(S::ANY.will_exit().and_then(S::ANY.will_enter())),
        ),
        S::ANY.on_enter(log_state_enter::<S>),
    ));
}

pub fn schedule_apply_flush<S: GetState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::relax)
            .run_if(was_triggered::<S>)
            .in_set(ApplyFlushSet),
    );
}

pub fn schedule_bevy_state<S: GetState + SetState + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let update_bevy_state =
        |pyri_state: NextStateRef<S>, mut bevy_state: ResMut<NextState<BevyState<S>>>| {
            if bevy_state.0.is_none() {
                bevy_state.set(BevyState(pyri_state.get().cloned()));
            }
        };

    let update_pyri_state = |mut pyri_state: NextStateMut<S>,
                             bevy_state: Res<NextState<BevyState<S>>>| {
        if let Some(value) = bevy_state.0.clone() {
            pyri_state.trigger().set(value.0);
        }
    };

    schedule.add_systems((
        update_pyri_state.in_set(StateFlushSet::<S>::Trigger),
        S::on_flush(update_bevy_state),
    ));
}
