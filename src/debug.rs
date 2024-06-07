//! TODO: Module-level documentation

use std::{any::type_name, fmt::Debug, marker::PhantomData};

use bevy_app::{App, Plugin};
use bevy_core::FrameCount;
use bevy_ecs::{
    schedule::{common_conditions::resource_exists, Condition, IntoSystemConfigs, Schedule},
    system::{Res, Resource},
};
use bevy_log::info;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::{StatePattern, StateTransPattern},
    schedule::{was_triggered, StateFlush, StateFlushSet},
    state::{CurrentState, NextStateRef, State, StateFlushRef},
};

/// A resource that controls state-related debug behavior.
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
}

/// A plugin that schedules flush logging for the [`State`] type `S`.
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

fn log_state_flush<S: State + Debug>(frame: Res<FrameCount>, state: StateFlushRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let (old, new) = state.get();
    info!("[Frame {frame}] {ty} flush: {old:?} -> {new:?}");
}

fn log_state_exit<S: State + Debug>(frame: Res<FrameCount>, old: Res<CurrentState<S>>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let old = old.unwrap();
    info!("[Frame {frame}] {ty} exit:  {old:?}");
}

fn log_state_trans<S: State + Debug>(frame: Res<FrameCount>, state: StateFlushRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let (old, new) = state.unwrap();
    info!("[Frame {frame}] {ty} trans: {old:?} -> {new:?}");
}

fn log_state_enter<S: State + Debug>(frame: Res<FrameCount>, new: NextStateRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let new = new.unwrap();
    info!("[Frame {frame}] {ty} enter: {new:?}");
}

/// Add flush logging systems for the [`State`] type `S` to a schedule.
///
/// Used in [`LogFlushPlugin<S>`].
pub fn schedule_log_flush<S: State + Debug>(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            log_state_flush::<S>
                .after(StateFlushSet::<S>::Trigger)
                .before(StateFlushSet::<S>::Flush)
                .run_if(was_triggered::<S>.and_then(|x: Res<StateDebugSettings>| x.log_flush)),
            log_state_exit::<S>
                .in_set(StateFlushSet::<S>::Flush)
                .before(StateFlushSet::<S>::Exit)
                .run_if(
                    S::ANY
                        .will_exit()
                        .and_then(|x: Res<StateDebugSettings>| x.log_exit),
                ),
            log_state_trans::<S>
                .after(StateFlushSet::<S>::Exit)
                .before(StateFlushSet::<S>::Trans)
                .run_if(
                    S::ANY_TO_ANY
                        .will_trans()
                        .and_then(|x: Res<StateDebugSettings>| x.log_trans),
                ),
            log_state_enter::<S>
                .after(StateFlushSet::<S>::Trans)
                .before(StateFlushSet::<S>::Enter)
                .run_if(
                    S::ANY
                        .will_enter()
                        .and_then(|x: Res<StateDebugSettings>| x.log_enter),
                ),
        )
            .run_if(resource_exists::<StateDebugSettings>),
    );
}
