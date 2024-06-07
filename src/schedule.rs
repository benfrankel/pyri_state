//! TODO: Module-level documentation

use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{
        common_conditions::not, InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs,
        Schedule, ScheduleLabel, SystemSet,
    },
    system::{Res, ResMut},
};

#[cfg(feature = "bevy_state")]
use bevy_state::state::NextState;

use crate::state::{
    BevyState, CurrentState, NextStateMut, NextStateRef, State, StateFlushRef, StateMut,
    TriggerStateFlush,
};

/// The schedule that handles [`State`] flushes, added by [`StatePlugin`](crate::app::StatePlugin).
///
/// System ordering:
///
/// 1. [`StateFlushSet::<S>::Resolve`](StateFlushSet) per state type `S`.
/// 2. [`ApplyFlushSet`] for all state types.
#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

/// A suite of system sets in the [`StateFlush`] schedule per [`State`] type `S`.
///
/// Configured by default by [`ResolveStatePlugin<S>`](crate::app::ResolveStatePlugin) as follows:
///
/// 1. [`Self::Resolve`] before / after other `Resolve` system sets based on state dependencies,
/// and before [`ApplyFlushSet`].
///     1. [`Self::Compute`].
///     2. [`Self::Trigger`] if not yet triggered.
///     3. [`Self::Flush`] if triggered.
///         1. [`Self::Exit`].
///         2. [`Self::Trans`].
///         3. [`Self::Enter`].
///
/// See [`AddState`](crate::app::AddState) for how to opt out of default plugins.
#[derive(SystemSet)]
pub enum StateFlushSet<S: State> {
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
    Trans,
    /// Run enter hooks for `S`.
    Enter,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State> Clone for StateFlushSet<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolve => Self::Resolve,
            Self::Compute => Self::Compute,
            Self::Trigger => Self::Trigger,
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Trans => Self::Trans,
            Self::Enter => Self::Enter,
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

impl<S: State> PartialEq for StateFlushSet<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State> Eq for StateFlushSet<S> {}

impl<S: State> Hash for StateFlushSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State> Debug for StateFlushSet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve => write!(f, "Resolve"),
            Self::Compute => write!(f, "Compute"),
            Self::Trigger => write!(f, "Trigger"),
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Trans => write!(f, "Trans"),
            Self::Enter => write!(f, "Enter"),
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

/// A system set that applies all triggered [`State`] flushes at the end of
/// the [`StateFlush`] schedule.
#[derive(SystemSet, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ApplyFlushSet;

/// An event sent whenever the [`State`] type `S` flushes.
///
/// Added by default by [`FlushEventPlugin<S>`](crate::app::FlushEventPlugin). See
/// [`AddState`](crate::app::AddState) for how to opt out of default plugins.
#[derive(Event)]
pub struct StateFlushEvent<S: State> {
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
}

pub(crate) fn was_triggered<S: State>(trigger: Res<TriggerStateFlush<S>>) -> bool {
    trigger.0
}

/// Configure [`StateFlushSet`] system sets for the [`State`] type `S` to a schedule.
///
/// Used in [`ResolveStatePlugin<S>`](crate::app::ResolveStatePlugin).
pub fn schedule_resolve_state<S: State>(
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
            StateFlushSet::<S>::Trans,
            StateFlushSet::<S>::Enter,
        )
            .chain()
            .in_set(StateFlushSet::<S>::Flush),
    ));
}

/// Add change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`DetectChangePlugin<S>`](crate::app::DetectChangePlugin).
pub fn schedule_detect_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(|state: StateFlushRef<S>| matches!(state.get(), (x, y) if x != y))
            .in_set(StateFlushSet::<S>::Trigger),
    );
}

fn send_flush_event<S: State + Clone>(
    state: StateFlushRef<S>,
    mut events: EventWriter<StateFlushEvent<S>>,
) {
    let (old, new) = state.get();
    events.send(StateFlushEvent {
        old: old.cloned(),
        new: new.cloned(),
    });
}

/// Add a send [`StateFlushEvent`] system for the [`State`] type `S` to a schedule.
///
/// This is used by [`FlushEventPlugin`](crate::app::FlushEventPlugin).
pub fn schedule_send_event<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(send_flush_event::<S>.in_set(StateFlushSet::<S>::Flush));
}

/// Add [`BevyState<S>`] propagation systems for the [`State`] type `S` to a schedule.
///
/// Used in [`BevyStatePlugin<S>`](crate::app::BevyStatePlugin).
#[cfg(feature = "bevy_state")]
pub fn schedule_bevy_state<S: State + StateMut + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let update_bevy_state =
        |pyri_state: NextStateRef<S>, mut bevy_state: ResMut<NextState<BevyState<S>>>| {
            if matches!(bevy_state.as_ref(), NextState::Unchanged) {
                bevy_state.set(BevyState(pyri_state.get().cloned()));
            }
        };

    let update_pyri_state = |mut pyri_state: NextStateMut<S>,
                             bevy_state: Res<NextState<BevyState<S>>>| {
        if let NextState::Pending(bevy_state) = bevy_state.as_ref() {
            pyri_state.trigger().set(bevy_state.0.clone());
        }
    };

    schedule.add_systems((
        update_pyri_state.in_set(StateFlushSet::<S>::Compute),
        update_bevy_state.in_set(StateFlushSet::<S>::Flush),
    ));
}

fn apply_flush<S: State + Clone>(mut current: ResMut<CurrentState<S>>, next: NextStateRef<S>) {
    current.0 = next.get().cloned();
}

/// Add an apply flush system for the [`State`] type `S` to a schedule.
///
/// Used in [`ApplyFlushPlugin<S>`](crate::app::ApplyFlushPlugin).
pub fn schedule_apply_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::relax)
            .run_if(was_triggered::<S>)
            .in_set(ApplyFlushSet),
    );
}
