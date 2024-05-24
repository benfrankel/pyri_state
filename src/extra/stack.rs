use bevy_ecs::{
    schedule::Schedule,
    system::{Res, ResMut},
    world::{FromWorld, World},
};

use crate::{
    buffer::NextState_,
    state::{RawState, State_},
};

// TODO: StateStack::<S>::pop, etc. as systems
// A state stack with the current state on top
#[derive(PartialEq, Eq, Clone)]
pub struct StateStack<S: State_>(pub Vec<S>);

impl<S: State_> RawState for StateStack<S> {}

impl<S: State_ + FromWorld> FromWorld for StateStack<S> {
    fn from_world(world: &mut World) -> Self {
        Self(vec![S::from_world(world)])
    }
}

impl<S: State_> StateStack<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

fn compute_state_from_stack<S: State_>(
    mut state: ResMut<NextState_<S>>,
    stack: Res<NextState_<StateStack<S>>>,
) {
    state.set_flush(true).inner = stack.get().and_then(|stack| stack.0.last().cloned());
}

pub fn schedule_stack<S: State_>(schedule: &mut Schedule) {
    schedule.add_systems(StateStack::<S>::on_any_flush(compute_state_from_stack::<S>));
}

#[cfg(feature = "bevy_app")]
mod app {
    use std::marker::PhantomData;

    use bevy_app::App;
    use bevy_ecs::schedule::SystemSet;

    use crate::{
        app::{
            AppExtState, ConfigureState, GetStateConfig, StateConfigApplyFlush,
            StateConfigDetectChange, StateConfigResolveState,
        },
        buffer::NextState_,
        schedule::{StateFlush, StateFlushSet},
        state::{RawState, State_},
    };

    use super::{schedule_stack, StateStack};

    impl<S: State_ + GetStateConfig> GetStateConfig for StateStack<S> {
        fn get_config() -> impl ConfigureState {
            (
                StateConfigResolveState::<Self>::new(
                    vec![],
                    vec![StateFlushSet::<S>::Resolve.intern()],
                ),
                StateConfigDetectChange::<Self>::new(),
                StateConfigApplyFlush::<Self>::new(),
                StateConfigStack::<Self>::new(),
            )
        }
    }

    struct StateConfigStack<S: RawState>(PhantomData<S>);

    impl<S: State_ + GetStateConfig> ConfigureState for StateConfigStack<StateStack<S>> {
        fn configure(self, app: &mut App) {
            app.add_state_::<S>()
                .insert_resource(NextState_::enabled(StateStack::<S>::new()));
            schedule_stack::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: RawState> StateConfigStack<S> {
        fn new() -> Self {
            Self(PhantomData)
        }
    }
}
