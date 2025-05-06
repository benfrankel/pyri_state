//! State flush logging tools.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use core::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that adds on-flush logging systems for the [`State`] type `S`.
    ///
    /// Calls [`schedule_log_flush<S>`].
    pub struct LogFlushPlugin<S: State + Debug>(PhantomData<S>);

    impl<S: State + Debug> Plugin for LogFlushPlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_log_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State + Debug> Default for LogFlushPlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    /// A plugin that adds local on-flush logging systems for the [`State`] type `S`.
    ///
    /// Calls [`schedule_local_log_flush<S>`].
    pub struct LocalLogFlushPlugin<S: LocalState + Debug>(PhantomData<S>);

    impl<S: LocalState + Debug> Plugin for LocalLogFlushPlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_local_log_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: LocalState + Debug> Default for LocalLogFlushPlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use core::{any::type_name, fmt::Debug};

use bevy_diagnostic::FrameCount;
use bevy_ecs::{
    entity::Entity,
    schedule::{Condition, IntoScheduleConfigs, Schedule},
    system::{Query, Res, StaticSystemParam},
};
use bevy_log::info;

use crate::{
    access::{CurrentRef, FlushRef, NextRef},
    debug::StateDebugSettings,
    next_state::{NextState, TriggerStateFlush},
    pattern::{StatePattern, StateTransPattern},
    schedule::ResolveStateSystems,
    state::{LocalState, State},
};

fn log_state_flush<S: State + Debug>(frame: Res<FrameCount>, state: FlushRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let (old, new) = state.get();
    info!("[Frame {frame}] {ty} flush: {old:?} -> {new:?}");
}

fn log_state_exit<S: State + Debug>(frame: Res<FrameCount>, old: CurrentRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let old = old.unwrap();
    info!("[Frame {frame}] {ty} exit:  {old:?}");
}

fn log_state_trans<S: State + Debug>(frame: Res<FrameCount>, state: FlushRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let (old, new) = state.unwrap();
    info!("[Frame {frame}] {ty} trans: {old:?} -> {new:?}");
}

fn log_state_enter<S: State + Debug>(frame: Res<FrameCount>, new: NextRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let new = new.unwrap();
    info!("[Frame {frame}] {ty} enter: {new:?}");
}

/// Add on-flush logging systems for the [`State`] type `S` to a schedule.
///
/// Used in [`LogFlushPlugin<S>`].
pub fn schedule_log_flush<S: State + Debug>(schedule: &mut Schedule) {
    schedule.add_systems((
        log_state_flush::<S>
            .after(ResolveStateSystems::<S>::Trigger)
            .before(ResolveStateSystems::<S>::Flush)
            .run_if(
                S::is_triggered
                    .and(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_flush)),
            ),
        log_state_exit::<S>
            .in_set(ResolveStateSystems::<S>::Flush)
            .before(ResolveStateSystems::<S>::Exit)
            .run_if(
                S::is_triggered
                    .and(S::ANY.will_exit())
                    .and(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_exit)),
            ),
        log_state_trans::<S>
            .after(ResolveStateSystems::<S>::Exit)
            .before(ResolveStateSystems::<S>::Trans)
            .run_if(
                S::is_triggered
                    .and(S::ANY_TO_ANY.will_trans())
                    .and(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_trans)),
            ),
        log_state_enter::<S>
            .after(ResolveStateSystems::<S>::Trans)
            .before(ResolveStateSystems::<S>::Enter)
            .run_if(
                S::is_triggered
                    .and(S::ANY.will_enter())
                    .and(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_enter)),
            ),
    ));
}

fn log_local_state_flush<S: LocalState + Debug>(
    frame: Res<FrameCount>,
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, Option<&S>, &S::Next, &TriggerStateFlush<S>)>,
) {
    let frame = frame.0;
    let ty = type_name::<S>();
    for (entity, old, new, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }

        let new = new.next_state(&next_param);
        info!("[Frame {frame}] {ty} flush ({entity}): {old:?} -> {new:?}");
    }
}

fn log_local_state_exit<S: LocalState + Debug>(
    frame: Res<FrameCount>,
    state_query: Query<(Entity, &S, &TriggerStateFlush<S>)>,
) {
    let frame = frame.0;
    let ty = type_name::<S>();
    for (entity, old, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }

        info!("[Frame {frame}] {ty} exit ({entity}): {old:?}");
    }
}

fn log_local_state_trans<S: LocalState + Debug>(
    frame: Res<FrameCount>,
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, &S, &S::Next, &TriggerStateFlush<S>)>,
) {
    let frame = frame.0;
    let ty = type_name::<S>();
    for (entity, old, new, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }
        let Some(new) = new.next_state(&next_param) else {
            continue;
        };

        info!("[Frame {frame}] {ty} trans ({entity}): {old:?} -> {new:?}");
    }
}

fn log_local_state_enter<S: LocalState + Debug>(
    frame: Res<FrameCount>,
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, &S::Next, &TriggerStateFlush<S>)>,
) {
    let frame = frame.0;
    let ty = type_name::<S>();
    for (entity, new, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }
        let Some(new) = new.next_state(&next_param) else {
            continue;
        };

        info!("[Frame {frame}] {ty} enter ({entity}): {new:?}");
    }
}

/// Add local on-flush logging systems for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalLogFlushPlugin<S>`].
pub fn schedule_local_log_flush<S: LocalState + Debug>(schedule: &mut Schedule) {
    schedule.add_systems((
        log_local_state_flush::<S>
            .after(ResolveStateSystems::<S>::Trigger)
            .before(ResolveStateSystems::<S>::Flush)
            .run_if(|x: Option<Res<StateDebugSettings>>| {
                x.is_some_and(|x| x.log_local && x.log_flush)
            }),
        log_local_state_exit::<S>
            .in_set(ResolveStateSystems::<S>::Flush)
            .before(ResolveStateSystems::<S>::Exit)
            .run_if(|x: Option<Res<StateDebugSettings>>| {
                x.is_some_and(|x| x.log_local && x.log_exit)
            }),
        log_local_state_trans::<S>
            .after(ResolveStateSystems::<S>::Exit)
            .before(ResolveStateSystems::<S>::Trans)
            .run_if(|x: Option<Res<StateDebugSettings>>| {
                x.is_some_and(|x| x.log_local && x.log_trans)
            }),
        log_local_state_enter::<S>
            .after(ResolveStateSystems::<S>::Trans)
            .before(ResolveStateSystems::<S>::Enter)
            .run_if(|x: Option<Res<StateDebugSettings>>| {
                x.is_some_and(|x| x.log_local && x.log_enter)
            }),
    ));
}
