use bevy_ecs::{
    schedule::Schedule,
    system::{Res, ResMut},
    world::{FromWorld, World},
};
use pyri_state_derive::RawState;

use crate::{
    buffer::NextState_,
    state::{RawState, State_},
};

// A state stack with the current state on top.
#[derive(RawState, PartialEq, Eq, Clone)]
pub struct StateStack<S: State_>(pub Vec<S>);

impl<S: State_ + FromWorld> FromWorld for StateStack<S> {
    fn from_world(world: &mut World) -> Self {
        Self::new(S::from_world(world))
    }
}

impl<S: State_> StateStack<S> {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn new(state: S) -> Self {
        Self(vec![state])
    }

    pub fn clear(mut state: ResMut<NextState_<Self>>) {
        if let Some(state) = state.get_mut() {
            state.0.clear();
        }
    }

    pub fn pop(mut state: ResMut<NextState_<Self>>) {
        if let Some(state) = state.get_mut() {
            state.0.pop();
        }
    }

    pub fn push(value: S) -> impl Fn(ResMut<NextState_<Self>>) {
        move |mut state| {
            if let Some(state) = state.get_mut() {
                state.0.push(value.clone());
            }
        }
    }

    pub fn clear_push(value: S) -> impl Fn(ResMut<NextState_<Self>>) {
        move |mut state| {
            if let Some(state) = state.get_mut() {
                state.0.clear();
                state.0.push(value.clone());
            }
        }
    }

    pub fn pop_push(value: S) -> impl Fn(ResMut<NextState_<Self>>) {
        move |mut state| {
            if let Some(state) = state.get_mut() {
                state.0.pop();
                state.0.push(value.clone());
            }
        }
    }
}

fn compute_state_from_stack<S: State_>(
    mut state: ResMut<NextState_<S>>,
    stack: Res<NextState_<StateStack<S>>>,
) {
    state.set_flush(true).inner = stack.get().and_then(|stack| stack.0.last().cloned());
}

pub fn schedule_state_stack<S: State_>(schedule: &mut Schedule) {
    schedule.add_systems(StateStack::<S>::on_flush(compute_state_from_stack::<S>));
}

#[cfg(feature = "bevy_app")]
mod app {
    use bevy_app::App;

    use crate::{
        app::{
            AddState, AppExtPyriState, ApplyFlushPlugin, DetectChangePlugin, ResolveStatePlugin,
        },
        buffer::{CurrentState, NextState_},
        schedule::StateFlush,
        state::State_,
    };

    use super::{schedule_state_stack, StateStack};

    impl<S: State_ + AddState> AddState for StateStack<S> {
        fn add_state(app: &mut App, value: Option<Self>) {
            // Replace `None` with `StateStack(vec![])`.
            let value = value.unwrap_or_else(Self::empty);

            app.add_state_::<S>()
                .init_resource::<CurrentState<Self>>()
                .insert_resource(NextState_::enabled(value))
                .add_plugins((
                    ResolveStatePlugin::<Self>::default().before::<S>(),
                    DetectChangePlugin::<Self>::default(),
                    ApplyFlushPlugin::<Self>::default(),
                ));

            schedule_state_stack::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }
}
