//! State flush scheduling types and functions.
//!
//! The [`StateFlush`] schedule handles all [`State`] flush logic and emits [`StateFlushEvent`].

use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    entity::Entity,
    event::{Event, EventWriter},
    schedule::{
        common_conditions::not, Condition, InternedSystemSet, IntoSystemConfigs,
        IntoSystemSetConfigs, Schedule, ScheduleLabel, SystemSet,
    },
    system::{Commands, Query, StaticSystemParam},
};

use crate::{
    access::{CurrentMut, FlushRef, NextRef},
    state::{LocalState, NextState, State, StateExtEq as _, TriggerStateFlush},
};

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
///     2. [`Trigger`](Self::Trigger)
///     3. [`Flush`](Self::Flush)
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

/// An event sent whenever a local [`State`] type `S` flushes.
///
/// Added [by default](pyri_state_derive::State) by
/// [`LocalFlushEventPlugin<S>`](crate::extra::app::FlushEventPlugin).
#[derive(Event)]
pub struct LocalStateFlushEvent<S: LocalState> {
    /// The entity for which the state flush occurred.
    pub entity: Entity,
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
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
            // Systems in this system set should only run if not triggered.
            StateHook::<S>::Trigger,
            // Systems in this system set should only run if triggered.
            StateHook::<S>::Flush,
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
            .run_if(not(S::is_triggered).and_then(S::will_change))
            .in_set(StateHook::<S>::Trigger),
    );
}

fn local_detect_change<S: LocalState + Eq>(
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    mut state_query: Query<(Option<&S>, &S::Next, &mut TriggerStateFlush<S>)>,
) {
    for (current, next, mut trigger) in &mut state_query {
        if !trigger.0 && current != next.next_state(&next_param) {
            trigger.0 = true;
        }
    }
}

/// Add local change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalDetectChangePlugin<S>`](crate::extra::app::LocalDetectChangePlugin).
pub fn schedule_local_detect_change<S: LocalState + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(local_detect_change::<S>.in_set(StateHook::<S>::Trigger));
}

fn send_flush_event<S: State + Clone>(
    state: FlushRef<S>,
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
    schedule.add_systems(
        send_flush_event::<S>
            .run_if(S::is_triggered)
            .in_set(StateHook::<S>::Flush),
    );
}

fn send_local_flush_event<S: LocalState + Clone>(
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, Option<&S>, &S::Next, &TriggerStateFlush<S>)>,
    mut events: EventWriter<LocalStateFlushEvent<S>>,
) {
    for (entity, current, next, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }

        events.send(LocalStateFlushEvent {
            entity,
            old: current.cloned(),
            new: next.next_state(&next_param).cloned(),
        });
    }
}

/// Add a local [`StateFlushEvent<S>`] sending system for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalFlushEventPlugin<S>`](crate::extra::app::LocalFlushEventPlugin).
pub fn schedule_local_flush_event<S: LocalState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(send_local_flush_event::<S>.in_set(StateHook::<S>::Flush));
}

fn apply_flush<S: State + Clone>(
    mut commands: Commands,
    mut current: CurrentMut<S>,
    next: NextRef<S>,
) {
    match (current.get_mut(), next.get()) {
        (Some(x), Some(y)) => *x = y.clone(),
        (Some(_), None) => {
            commands.remove_resource::<S>();
        }
        (None, Some(y)) => {
            commands.insert_resource(y.clone());
        }
        _ => (),
    }
}

/// Add an apply flush system for the [`State`] type `S` to a schedule.
///
/// Used in [`ApplyFlushPlugin<S>`](crate::extra::app::ApplyFlushPlugin).
pub fn schedule_apply_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::reset_trigger)
            .run_if(S::is_triggered)
            .in_set(ApplyFlushSet),
    );
}

fn local_apply_flush<S: LocalState + Clone>(
    mut commands: Commands,
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    mut state_query: Query<(Entity, Option<&mut S>, &S::Next, &TriggerStateFlush<S>)>,
) {
    for (entity, current, next, trigger) in &mut state_query {
        if !trigger.0 {
            continue;
        }

        match (current, next.next_state(&next_param)) {
            (Some(mut x), Some(y)) => *x = y.clone(),
            (Some(_), None) => {
                commands.entity(entity).remove::<S>();
            }
            (None, Some(y)) => {
                commands.entity(entity).insert(y.clone());
            }
            _ => (),
        }
    }
}

fn local_reset_trigger<S: LocalState>(mut state_query: Query<&mut TriggerStateFlush<S>>) {
    for mut trigger in &mut state_query {
        trigger.0 = false;
    }
}

/// Add a local apply flush system for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalApplyFlushPlugin<S>`](crate::extra::app::LocalApplyFlushPlugin).
pub fn schedule_local_apply_flush<S: LocalState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (local_apply_flush::<S>, local_reset_trigger::<S>)
            .chain()
            .in_set(ApplyFlushSet),
    );
}
