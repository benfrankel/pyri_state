// Swap out or define your own state storage type.

use bevy::{
    ecs::system::SystemParamItem, input::common_conditions::input_just_pressed, prelude::*,
};
use pyri_state::{
    prelude::*,
    state::{StateStorage, StateStorageMut},
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

#[derive(State, Clone, PartialEq, Eq, Default)]
// The default storage is `StateBuffer<Self>`, which is a newtyped `Option<Self>`.
//#[state(storage(StateBuffer<Self>))]
struct MyBufferedState;

#[derive(State, Clone, PartialEq, Eq, Debug, Default)]
// You can easily swap in a `StateStack<Self>` instead, for example.
#[state(log_flush, storage(StateStack<Self>))]
enum MyStackedState {
    #[default]
    A,
    B,
}

// You can define your own fully custom storage type:
#[derive(Resource)]
pub struct StateSwap<S: State>([Option<S>; 2]);

// Impl `StateStorage` to mark your type as a valid state storage type.
impl<S: State> StateStorage for StateSwap<S> {
    type State = S;

    type Param = ();

    // This is used in `app.add_state`.
    fn empty() -> Self {
        Self([None, None])
    }

    // This allows `NextStateRef<S>` and `StateRef<S>` to interface with your storage type.
    fn get_state<'s>(&'s self, _param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        self.0[0].as_ref()
    }
}

// Impl `StateStorageMut` to support setting the next state through your storage type.
impl<S: State> StateStorageMut for StateSwap<S> {
    type ParamMut = ();

    fn get_state_from_mut<'s>(
        &'s self,
        _param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s S> {
        self.0[0].as_ref()
    }

    // This allows `NextStateMut<S>` and `StateMut<S>` to interface with your storage type,
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
// state types using your storage type.
pub trait StateSwapMut: State {
    fn swap(mut swap: ResMut<StateSwap<Self>>) {
        let [left, right] = &mut swap.0;
        std::mem::swap(left, right);
    }
}

// Blanket impl the trait.
impl<S: State<Storage = StateSwap<S>>> StateSwapMut for S {}

#[derive(State, Clone, PartialEq, Eq, Debug)]
// Now you can use `StateSwap<Self>` as a first-class custom storage type!
#[state(log_flush, storage(StateSwap<Self>))]
enum MySwappedState {
    X,
    Y,
}
