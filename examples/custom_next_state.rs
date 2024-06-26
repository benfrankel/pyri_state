// Swap out or define your own next state type.

use bevy::{
    ecs::system::SystemParamItem, input::common_conditions::input_just_pressed, prelude::*,
};
use pyri_state::{
    prelude::*,
    state::{NextState, NextStateMut},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<MyBufferedState>()
        .init_state::<MyStackedState>()
        .insert_state(StateSwap([
            Some(MySwappedState::X),
            Some(MySwappedState::Y),
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
                MySwappedState::swap.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .run();
}

#[derive(State, Component, Clone, PartialEq, Eq, Default)]
// The default `NextState` type is `StateBuffer<Self>`, which is a newtyped `Option<Self>`.
//#[state(next(StateBuffer<Self>))]
struct MyBufferedState;

#[derive(State, Component, Clone, PartialEq, Eq, Debug, Default)]
// You can easily swap in a `StateStack<Self>` instead, for example.
#[state(log_flush, next(StateStack<Self>))]
enum MyStackedState {
    #[default]
    A,
    B,
}

// You can define your own fully custom next state type:
#[derive(Component)]
pub struct StateSwap<S: State>([Option<S>; 2]);

impl<S: State> NextState for StateSwap<S> {
    type State = S;

    type Param = ();

    // This is used in `app.add_state`.
    fn empty() -> Self {
        Self([None, None])
    }

    // This allows `NextRef<S>` and `FlushRef<S>` to interface with your `NextState` type.
    fn get_state<'s>(&'s self, _param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        self.0[0].as_ref()
    }
}

impl<S: State> NextStateMut for StateSwap<S> {
    type ParamMut = ();

    fn get_state_from_mut<'s>(
        &'s self,
        _param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s S> {
        self.0[0].as_ref()
    }

    // This allows `NextMut<S>` and `FlushMut<S>` to interface with your `NextState` type,
    fn get_state_mut<'s>(
        &'s mut self,
        _param: &'s mut SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut S> {
        self.0[0].as_mut()
    }

    fn set_state(&mut self, _param: &mut SystemParamItem<Self::ParamMut>, state: Option<S>) {
        self.0[0] = state;
    }
}

// Define a custom extension trait to attach extra systems and run conditions to
// `State` types using your `NextState` type.
pub trait StateSwapMut: State {
    fn swap(mut swap: Query<&mut StateSwap<Self>, With<GlobalStates>>) {
        let [left, right] = &mut swap.single_mut().0;
        std::mem::swap(left, right);
    }
}

// Blanket impl the trait.
impl<S: State<Next = StateSwap<S>>> StateSwapMut for S {}

#[derive(State, Component, Clone, PartialEq, Eq, Debug)]
// Now you can use `StateSwap<Self>` as a first-class custom next state type!
#[state(log_flush, next(StateSwap<Self>))]
enum MySwappedState {
    X,
    Y,
}
