//! State flush scheduling types and functions.
//!
//! The [`StateFlush`] schedule handles all [`State`] flush logic and emits [`StateFlushEvent`].

use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::{Event, EventWriter},
    schedule::{
        common_conditions::not, InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs,
        Schedule, ScheduleLabel, SystemSet,
    },
    system::{Res, ResMut},
};

use crate::state::{CurrentState, NextStateRef, State, StateFlushRef, TriggerStateFlush};

/// The schedule that handles all [`State`] flush logic, added after
/// [`PreUpdate`](bevy_app::PreUpdate) by [`StatePlugin`](crate::extra::app::StatePlugin).
///
/// State flush hooks run in [`StateHook<S>`] system sets and the flush is applied in
/// [`ApplyFlushSet`].
#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

/// A suite of system sets in the [`StateFlush`] schedule for each [`State`] type `S`.
///
/// Configured [by default](pyri_state_derive::State) by
/// [`ResolveStatePlugin<S>`](crate::extra::app::ResolveStatePlugin) as follows:
///
/// 1. [`Resolve`](Self::Resolve) (before or after other `Resolve` system sets based on
/// state dependencies, and before [`ApplyFlushSet`])
///     1. [`Compute`](Self::Compute)
///     2. [`Trigger`](Self::Trigger) (if not yet triggered)
///     3. [`Flush`](Self::Flush) (if triggered)
///         1. [`Exit`](Self::Exit)
///         2. [`Trans`](Self::Trans)
///         3. [`Enter`](Self::Enter)
#[derive(SystemSet)]
pub enum StateHook<S: State> {
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

impl<S: State> Clone for StateHook<S> {
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

impl<S: State> PartialEq for StateHook<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State> Eq for StateHook<S> {}

impl<S: State> Hash for StateHook<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State> Debug for StateHook<S> {
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
/// Added [by default](pyri_state_derive::State) by
/// [`FlushEventPlugin<S>`](crate::extra::app::FlushEventPlugin).
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

/// Configure [`StateHook<S>`] system sets for the [`State`] type `S` in a schedule.
///
/// To specify a dependency relative to another `State` type `T`, include
/// [`StateHook::<T>::Resolve`] in `after` or `before`.
///
/// Used in [`ResolveStatePlugin<S>`](crate::extra::app::ResolveStatePlugin).
pub fn schedule_resolve_state<S: State>(
    schedule: &mut Schedule,
    after: &[InternedSystemSet],
    before: &[InternedSystemSet],
) {
    // External ordering
    for &system_set in after {
        schedule.configure_sets(StateHook::<S>::Resolve.after(system_set));
    }
    for &system_set in before {
        schedule.configure_sets(StateHook::<S>::Resolve.before(system_set));
    }

    // Internal ordering
    schedule.configure_sets((
        StateHook::<S>::Resolve.before(ApplyFlushSet),
        (
            StateHook::<S>::Compute,
            StateHook::<S>::Trigger.run_if(not(was_triggered::<S>)),
            StateHook::<S>::Flush.run_if(was_triggered::<S>),
        )
            .chain()
            .in_set(StateHook::<S>::Resolve),
        (
            StateHook::<S>::Exit,
            StateHook::<S>::Trans,
            StateHook::<S>::Enter,
        )
            .chain()
            .in_set(StateHook::<S>::Flush),
    ));
}

/// Add change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`DetectChangePlugin<S>`](crate::extra::app::DetectChangePlugin).
pub fn schedule_detect_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(|state: StateFlushRef<S>| matches!(state.get(), (x, y) if x != y))
            .in_set(StateHook::<S>::Trigger),
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

/// Add a [`StateFlushEvent<S>`] sending system for the [`State`] type `S` to a schedule.
///
/// Used in [`FlushEventPlugin<S>`](crate::extra::app::FlushEventPlugin).
pub fn schedule_flush_event<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(send_flush_event::<S>.in_set(StateHook::<S>::Flush));
}

fn apply_flush<S: State + Clone>(mut current: ResMut<CurrentState<S>>, next: NextStateRef<S>) {
    current.0 = next.get().cloned();
}

/// Add an apply flush system for the [`State`] type `S` to a schedule.
///
/// Used in [`ApplyFlushPlugin<S>`](crate::extra::app::ApplyFlushPlugin).
pub fn schedule_apply_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::relax)
            .run_if(was_triggered::<S>)
            .in_set(ApplyFlushSet),
    );
}
