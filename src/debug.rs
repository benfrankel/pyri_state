//! TODO: Module-level documentation

use std::{any::type_name, fmt::Debug, marker::PhantomData};

use bevy_app::{App, Plugin};
use bevy_core::FrameCount;
use bevy_ecs::{
    schedule::{common_conditions::resource_exists_and_equals, IntoSystemConfigs, Schedule},
    system::{Res, Resource},
};
use bevy_log::info;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::{StatePattern, StateTransitionPattern},
    schedule::StateFlush,
    state::{CurrentState, NextStateRef, StateFlushRef, State_},
};

#[derive(Resource, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub enum StateDebugSettings {
    #[default]
    Disabled,
    Enabled,
}

pub struct LogFlushPlugin<S: State_ + Debug>(PhantomData<S>);

impl<S: State_ + Debug> Plugin for LogFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_log_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: State_ + Debug> Default for LogFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn log_state_exit<S: State_ + Debug>(frame: Res<FrameCount>, old: Res<CurrentState<S>>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let old = old.unwrap();
    info!("[Frame {frame}] {ty} exit:  {old:?}");
}

fn log_state_transition<S: State_ + Debug>(frame: Res<FrameCount>, state: StateFlushRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let (old, new) = state.unwrap();
    info!("[Frame {frame}] {ty} trans: {old:?} -> {new:?}");
}

fn log_state_enter<S: State_ + Debug>(frame: Res<FrameCount>, new: NextStateRef<S>) {
    let frame = frame.0;
    let ty = type_name::<S>();
    let new = new.unwrap();
    info!("[Frame {frame}] {ty} enter: {new:?}");
}

pub fn schedule_log_flush<S: State_ + Debug>(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            S::ANY.on_exit(log_state_exit::<S>),
            (S::ANY, S::ANY).on_transition(log_state_transition::<S>),
            S::ANY.on_enter(log_state_enter::<S>),
        )
            .run_if(resource_exists_and_equals(StateDebugSettings::Enabled)),
    );
}
