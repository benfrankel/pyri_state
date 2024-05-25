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
    use bevy_ecs::schedule::SystemSet;

    use crate::{
        app::{
            AppExtState, ApplyFlushPlugin, ConfigureState, DetectChangePlugin, ResolveStatePlugin,
        },
        buffer::NextState_,
        schedule::{StateFlush, StateFlushSet},
        state::State_,
    };

    use super::{schedule_state_stack, StateStack};

    impl<S: State_ + ConfigureState> ConfigureState for StateStack<S> {
        fn configure(app: &mut App) {
            app.add_state_::<S>().add_plugins((
                ResolveStatePlugin::<StateStack<S>>::new(
                    vec![],
                    vec![StateFlushSet::<S>::Resolve.intern()],
                ),
                DetectChangePlugin::<StateStack<S>>::default(),
                ApplyFlushPlugin::<StateStack<S>>::default(),
            ));

            // Replace `None` with `StateStack(vec![])`.
            if app
                .world
                .get_resource::<NextState_<StateStack<S>>>()
                .is_some_and(|x| x.will_be_disabled())
            {
                app.insert_resource(NextState_::enabled(StateStack::<S>::empty()));
            }

            schedule_state_stack::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }
}
