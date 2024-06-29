//! State debugging tools.
//!
//! Enable the `debug` feature flag to use this module.
//!
//! Insert the [`StateDebugSettings`] resource to enable debug tools.

use std::{any::type_name, fmt::Debug};

use bevy_core::FrameCount;
#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::{
    entity::Entity,
    schedule::{common_conditions::resource_exists, Condition, IntoSystemConfigs, Schedule},
    system::{Query, Res, Resource, StaticSystemParam},
};
use bevy_log::info;

use crate::{
    access::{CurrentRef, FlushRef, NextRef},
    pattern::{StatePattern, StateTransPattern},
    schedule::StateHook,
    state::{LocalState, NextState, State, TriggerStateFlush},
};

/// A resource that controls the behavior of [state debugging tools](crate::extra::debug).
#[derive(Resource, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateDebugSettings {
    /// Enable on-flush logs.
    pub log_flush: bool,
    /// Enable on-exit logs.
    pub log_exit: bool,
    /// Enable on-transition logs.
    pub log_trans: bool,
    /// Enable on-enter logs.
    pub log_enter: bool,
    /// Enable logging for local states.
    pub log_local: bool,
}

/// A plugin that adds on-flush logging systems for the [`State`] type `S`.
///
/// Calls [`schedule_log_flush<S>`].
#[cfg(feature = "bevy_app")]
pub struct LogFlushPlugin<S: State + Debug>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: State + Debug> bevy_app::Plugin for LogFlushPlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        schedule_log_flush::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: State + Debug> Default for LogFlushPlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

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
    schedule.add_systems(
        (
            log_state_flush::<S>
                .after(StateHook::<S>::Trigger)
                .before(StateHook::<S>::Flush)
                .run_if(S::is_triggered.and_then(|x: Res<StateDebugSettings>| x.log_flush)),
            log_state_exit::<S>
                .in_set(StateHook::<S>::Flush)
                .before(StateHook::<S>::Exit)
                .run_if(
                    S::is_triggered
                        .and_then(S::ANY.will_exit())
                        .and_then(|x: Res<StateDebugSettings>| x.log_exit),
                ),
            log_state_trans::<S>
                .after(StateHook::<S>::Exit)
                .before(StateHook::<S>::Trans)
                .run_if(
                    S::is_triggered
                        .and_then(S::ANY_TO_ANY.will_trans())
                        .and_then(|x: Res<StateDebugSettings>| x.log_trans),
                ),
            log_state_enter::<S>
                .after(StateHook::<S>::Trans)
                .before(StateHook::<S>::Enter)
                .run_if(
                    S::is_triggered
                        .and_then(S::ANY.will_enter())
                        .and_then(|x: Res<StateDebugSettings>| x.log_enter),
                ),
        )
            .run_if(resource_exists::<StateDebugSettings>),
    );
}

/// A plugin that adds local on-flush logging systems for the [`State`] type `S`.
///
/// Calls [`schedule_local_log_flush<S>`].
#[cfg(feature = "bevy_app")]
pub struct LocalLogFlushPlugin<S: LocalState + Debug>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: LocalState + Debug> bevy_app::Plugin for LocalLogFlushPlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        schedule_log_flush::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: LocalState + Debug> Default for LocalLogFlushPlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
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
    schedule.add_systems(
        (
            log_local_state_flush::<S>
                .after(StateHook::<S>::Trigger)
                .before(StateHook::<S>::Flush)
                .run_if(|x: Res<StateDebugSettings>| x.log_local && x.log_flush),
            log_local_state_exit::<S>
                .in_set(StateHook::<S>::Flush)
                .before(StateHook::<S>::Exit)
                .run_if(|x: Res<StateDebugSettings>| x.log_local && x.log_exit),
            log_local_state_trans::<S>
                .after(StateHook::<S>::Exit)
                .before(StateHook::<S>::Trans)
                .run_if(|x: Res<StateDebugSettings>| x.log_local && x.log_trans),
            log_local_state_enter::<S>
                .after(StateHook::<S>::Trans)
                .before(StateHook::<S>::Enter)
                .run_if(|x: Res<StateDebugSettings>| x.log_local && x.log_enter),
        )
            .run_if(resource_exists::<StateDebugSettings>),
    );
}
