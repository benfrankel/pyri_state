//! TODO: Module-level documentation

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
    BevyState, CurrentState, NextStateMut, NextStateRef, StateFlushRef, StateMut, State_,
    TriggerStateFlush,
};

/// The schedule that handles [`State_`] flushes, added by [`StatePlugin`](crate::app::StatePlugin).
///
/// System ordering:
///
/// 1. [`StateFlushSet::<S>::Resolve`](StateFlushSet) per state type `S`.
/// 2. [`ApplyFlushSet`] for all state types.
#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

/// A suite of system sets in the [`StateFlush`] schedule per [`State_`] type `S`.
///
/// Configured by default by [`ResolveStatePlugin<S>`](crate::app::ResolveStatePlugin) as follows:
///
/// 1. [`Self::Resolve`] before / after other `Resolve` system sets based on state dependencies,
/// and before [`ApplyFlushSet`].
///     1. [`Self::Compute`].
///     2. [`Self::Trigger`] if not yet triggered.
///     3. [`Self::Flush`] if triggered.
///         1. [`Self::Exit`].
///         2. [`Self::Transition`].
///         3. [`Self::Enter`].
///
/// See [`AddState`](crate::app::AddState) for how to opt out of default plugins.
#[derive(SystemSet)]
pub enum StateFlushSet<S: State_> {
    /// Resolve the state flush logic for `S` this frame.
    Resolve,
    /// Optionally compute the next value for `S`.
    Compute,
    /// Decide whether to trigger a flush for `S` this frame.
    Trigger,
    /// Run flush hooks for `S`.
    Flush,
    /// Run exit hooks for `S`.
    Exit,
    /// Run transition hooks for `S`.
    Transition,
    /// Run enter hooks for `S`.
    Enter,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State_> Clone for StateFlushSet<S> {
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

impl<S: State_> PartialEq for StateFlushSet<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State_> Eq for StateFlushSet<S> {}

impl<S: State_> Hash for StateFlushSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State_> Debug for StateFlushSet<S> {
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

/// A system set that applies all triggered [`State_`] flushes at the end of
/// the [`StateFlush`] schedule.
#[derive(SystemSet, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ApplyFlushSet;

/// An event sent whenever the [`State_`] type `S` flushes.
///
/// Added by default by [`FlushEventPlugin<S>`](crate::app::FlushEventPlugin). See
/// [`AddState`](crate::app::AddState) for how to opt out of default plugins.
#[derive(Event)]
pub struct StateFlushEvent<S: State_> {
    pub old: Option<S>,
    pub new: Option<S>,
}

fn was_triggered<S: State_>(trigger: Res<TriggerStateFlush<S>>) -> bool {
    trigger.0
}

/// Configure [`StateFlushSet`] system sets for the [`State_`] type `S` to a schedule.
///
/// This is used by [`ResolveStatePlugin<S>`](crate::app::ResolveStatePlugin).
pub fn schedule_resolve_state<S: State_>(
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

/// Add change detection systems for the [`State_`] type `S` to a schedule.
///
/// This is used by [`DetectChangePlugin<S>`](crate::app::DetectChangePlugin).
pub fn schedule_detect_change<S: State_ + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(|state: StateFlushRef<S>| matches!(state.get(), (x, y) if x != y))
            .in_set(StateFlushSet::<S>::Trigger),
    );
}

fn send_flush_event<S: State_ + Clone>(
    state: StateFlushRef<S>,
    mut events: EventWriter<StateFlushEvent<S>>,
) {
    let (old, new) = state.get();
    events.send(StateFlushEvent {
        old: old.cloned(),
        new: new.cloned(),
    });
}

/// Add a send [`StateFlushEvent`] system for the [`State_`] type `S` to a schedule.
///
/// This is used by [`FlushEventPlugin<S>`](crate::app::FlushEventPlugin).
pub fn schedule_send_event<S: State_ + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(send_flush_event::<S>.in_set(StateFlushSet::<S>::Flush));
}

/// Add [`BevyState<S>`] propagation systems for the [`State_`] type `S` to a schedule.
///
/// This is used by [`BevyStatePlugin<S>`](crate::app::BevyStatePlugin).
pub fn schedule_bevy_state<S: State_ + StateMut + Clone + PartialEq + Eq + Hash + Debug>(
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
        update_pyri_state.in_set(StateFlushSet::<S>::Compute),
        update_bevy_state.in_set(StateFlushSet::<S>::Flush),
    ));
}

fn apply_flush<S: State_ + Clone>(mut current: ResMut<CurrentState<S>>, next: NextStateRef<S>) {
    current.0 = next.get().cloned();
}

/// Add an apply flush system for the [`State_`] type `S` to a schedule.
///
/// This is used by [`ApplyFlushPlugin<S>`](crate::app::ApplyFlushPlugin).
pub fn schedule_apply_flush<S: State_ + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::relax)
            .run_if(was_triggered::<S>)
            .in_set(ApplyFlushSet),
    );
}
