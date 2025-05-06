//! Swap out or define your own next state type.

use bevy::{
    ecs::system::SystemParamItem, input::common_conditions::input_just_pressed, prelude::*,
};
use pyri_state::{
    next_state::{NextState, NextStateMut},
    prelude::*,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<MyBufferedState>()
        .init_state::<MyStackedState>()
        .insert_state(NextStatePair([
            Some(MyPairedState::X),
            Some(MyPairedState::Y),
        ]))
        .add_systems(
            Update,
            (
                MyStackedState::A
                    .push()
                    .run_if(input_just_pressed(KeyCode::KeyA)),
                MyStackedState::B
                    .push()
                    .run_if(input_just_pressed(KeyCode::KeyB)),
                MyStackedState::pop.run_if(input_just_pressed(KeyCode::Escape)),
                MyPairedState::swap.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Default)]
// The default `NextState` type is `NextStateBuffer<Self>`, which is a newtyped `Option<Self>`.
//#[state(next(NextStateBuffer<Self>))]
#[reflect(Resource)]
struct MyBufferedState;

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug, Default)]
// You can easily swap in a `NextStateStack<Self>` instead, for example.
#[state(log_flush, next(NextStateStack<Self>))]
#[reflect(Resource)]
enum MyStackedState {
    #[default]
    A,
    B,
}

// You can define your own fully custom `NextState` type:
#[derive(Resource, Component, Reflect)]
#[reflect(Resource, Component)]
struct NextStatePair<S: State>([Option<S>; 2]);

impl<S: State> NextState for NextStatePair<S> {
    type State = S;

    type Param = ();

    // This is used in `app.add_state`.
    fn empty() -> Self {
        Self([None, None])
    }

    // This allows `NextRef<S>` and `FlushRef<S>` to interface with your `NextState` type.
    fn next_state<'s>(&'s self, _param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        self.0[0].as_ref()
    }
}

impl<S: State> NextStateMut for NextStatePair<S> {
    type ParamMut = ();

    fn next_state_from_mut<'s>(
        &'s self,
        _param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s S> {
        self.0[0].as_ref()
    }

    // This allows `NextMut<S>` and `FlushMut<S>` to interface with your `NextState` type,
    fn next_state_mut<'s>(
        &'s mut self,
        _param: &'s mut SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut S> {
        self.0[0].as_mut()
    }

    fn set_next_state(&mut self, _param: &mut SystemParamItem<Self::ParamMut>, state: Option<S>) {
        self.0[0] = state;
    }
}

// Define a custom extension trait to attach extra systems and run conditions to
// `State` types using your `NextState` type.
trait NextStatePairMut: State {
    fn swap(mut swap: ResMut<NextStatePair<Self>>) {
        let [left, right] = &mut swap.0;
        core::mem::swap(left, right);
    }
}

// Blanket impl the trait.
impl<S: State<Next = NextStatePair<S>>> NextStatePairMut for S {}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug)]
// Now you can use `NextStatePair<Self>` as a custom first-class `NextState` type!
#[state(log_flush, next(NextStatePair<Self>))]
#[reflect(Resource)]
enum MyPairedState {
    X,
    Y,
}
