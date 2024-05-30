// Swap out or define your own state storage type.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_ecs::system::lifetimeless::{SRes, SResMut};
use pyri_state::{
    app::AddStateStorage,
    prelude::*,
    storage::{
        stack::{StateStack, StateStackMut, StateStackMutExtClone},
        GetStateStorage, SetStateStorage, StateStorage,
    },
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .init_state_::<MySlottedState>()
        .init_state_::<MyStackedState>()
        .init_state_::<MySwappedState>()
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
// The default storage is `StateSlot<Self>` (newtyped `Option<Self>`).
//#[state(storage(StateSlot<Self>))]
struct MySlottedState;

#[derive(State, Clone, PartialEq, Eq, Default)]
// You can easily swap in a `StateStack<Self>` instead (newtyped `Vec<Self>`).
#[state(storage(StateStack<Self>))]
enum MyStackedState {
    #[default]
    A,
    B,
}

// You can create your own fully custom state storage type:
#[derive(Resource)]
pub struct StateSwap<S: RawState>([Option<S>; 2]);

// Impl `StateStorage<S>` to mark your type as a valid state storage type.
impl<S: RawState> StateStorage<S> for StateSwap<S> {}

// Impl `GetStateStorage<S>` to enable getting the next state from your storage type.
// This allows `NextStateRef<S>` and `StateRef<S>` to interface with your storage type,
// and attaches run conditions such as `S::will_be_enabled`.
impl<S: RawState> GetStateStorage<S> for StateSwap<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s bevy_ecs::system::SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.0[0].as_ref()
    }
}

// Impl `SetStateStorage<S>` to enable setting the next state for your storage type.
// This allows `NextStateMut<S>` and `StateMut<S>` to interface with your storage type,
// and attaches systems such as `S::disable`.
impl<S: RawState> SetStateStorage<S> for StateSwap<S> {
    type Param = SResMut<Self>;

    fn get_state_from_mut<'s>(
        param: &'s bevy_ecs::system::SystemParamItem<Self::Param>,
    ) -> Option<&'s S> {
        param.0[0].as_ref()
    }

    fn get_state_mut<'s>(
        param: &'s mut bevy_ecs::system::SystemParamItem<Self::Param>,
    ) -> Option<&'s mut S> {
        param.0[0].as_mut()
    }

    fn set_state(param: &mut bevy_ecs::system::SystemParamItem<Self::Param>, state: Option<S>) {
        param.0[0] = state;
    }
}

// Impl `AddStateStorage<S>` to enable `app.add_state_::<S>()`, etc.
impl<S: RawState> AddStateStorage<S> for StateSwap<S> {
    fn add_state_storage(app: &mut bevy_app::App, state: Option<S>) {
        app.insert_resource(StateSwap([state, None]));
    }
}

// Define a custom trait to associate extra systems and run conditions with any
// state using your storage.
pub trait StateSwapMut: RawState {
    fn swap(mut swap: ResMut<StateSwap<Self>>) {
        let [left, right] = &mut swap.0;
        std::mem::swap(left, right);
    }
}

// Blanket impl the trait.
impl<S: RawState<Storage = StateSwap<S>>> StateSwapMut for S {}

#[derive(State, Clone, PartialEq, Eq, Default)]
// Now you can use `StateSwap<Self>` as a first-class custom storage type!
#[state(storage(StateSwap<Self>))]
struct MySwappedState;
