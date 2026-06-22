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
}

use core::{any::type_name, fmt::Debug};

use bevy_diagnostic::FrameCount;
use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, SystemCondition},
    system::Res,
};
use bevy_log::info;

use crate::{
    access::{CurrentRef, FlushRef, NextRef},
    debug::StateDebugSettings,
    pattern::{StatePattern, StateTransPattern},
    schedule::ResolveStateSystems,
    state::State,
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
                    .and_then(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_flush)),
            ),
        log_state_exit::<S>
            .in_set(ResolveStateSystems::<S>::Flush)
            .before(ResolveStateSystems::<S>::Exit)
            .run_if(
                S::is_triggered
                    .and_then(S::ANY.will_exit())
                    .and_then(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_exit)),
            ),
        log_state_trans::<S>
            .after(ResolveStateSystems::<S>::Exit)
            .before(ResolveStateSystems::<S>::Trans)
            .run_if(
                S::is_triggered
                    .and_then(S::ANY_TO_ANY.will_trans())
                    .and_then(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_trans)),
            ),
        log_state_enter::<S>
            .after(ResolveStateSystems::<S>::Trans)
            .before(ResolveStateSystems::<S>::Enter)
            .run_if(
                S::is_triggered
                    .and_then(S::ANY.will_enter())
                    .and_then(|x: Option<Res<StateDebugSettings>>| x.is_some_and(|x| x.log_enter)),
            ),
    ));
}
