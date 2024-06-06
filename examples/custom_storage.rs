// Swap out or define your own state storage type.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_ecs::system::lifetimeless::{SRes, SResMut};
use pyri_state::{
    app::{AddState, AddStateStorage},
    debug::StateDebugSettings,
    extra::stack::*,
    prelude::*,
    storage::{StateStorage, StateStorageMut},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings::Enabled)
        .init_state_::<MyBufferedState>()
        .init_state_::<MyStackedState>()
        .insert_state_(StateSwap([
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
// The default storage is `StateBuffer<Self>` (newtyped `Option<Self>`).
//#[state(storage(StateBuffer<Self>))]
struct MyBufferedState;

#[derive(State, Clone, PartialEq, Eq, Debug, Default)]
// You can easily swap in a `StateStack<Self>` instead.
#[state(log_flush, storage(StateStack<Self>))]
enum MyStackedState {
    #[default]
    A,
    B,
}

// You can create your own fully custom state storage type:
#[derive(Resource)]
pub struct StateSwap<S: State_>([Option<S>; 2]);

// Impl `StateStorage<S>` to enable getting the next state from your storage type.
// This allows `NextStateRef<S>` and `StateRef<S>` to interface with your storage type,
// and attaches run conditions such as `S::will_be_enabled`.
impl<S: State_> StateStorage<S> for StateSwap<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s bevy_ecs::system::SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.0[0].as_ref()
    }
}

// Impl `StateStorageMut<S>` to enable setting the next state for your storage type.
// This allows `NextStateMut<S>` and `StateMut<S>` to interface with your storage type,
// and attaches systems such as `S::disable`.
impl<S: State_> StateStorageMut<S> for StateSwap<S> {
    type ParamMut = SResMut<Self>;

    fn get_state_from_mut<'s>(
        param: &'s bevy_ecs::system::SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s S> {
        param.0[0].as_ref()
    }

    fn get_state_mut<'s>(
        param: &'s mut bevy_ecs::system::SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut S> {
        param.0[0].as_mut()
    }

    fn set_state(param: &mut bevy_ecs::system::SystemParamItem<Self::ParamMut>, state: Option<S>) {
        param.0[0] = state;
    }
}

// Impl `AddStateStorage<S>` to enable `app.add_state_::<S>()`, etc.
impl<S: AddState<AddStorage = Self>> AddStateStorage for StateSwap<S> {
    type AddState = S;

    fn add_state_storage(app: &mut bevy_app::App, storage: Option<Self>) {
        app.insert_resource(storage.unwrap_or_else(|| StateSwap([None, None])));
    }
}

// Define a custom trait to associate extra systems and run conditions with any
// state using your storage.
pub trait StateSwapMut: State_ {
    fn swap(mut swap: ResMut<StateSwap<Self>>) {
        let [left, right] = &mut swap.0;
        std::mem::swap(left, right);
    }
}

// Blanket impl the trait.
impl<S: State_<Storage = StateSwap<S>>> StateSwapMut for S {}

#[derive(State, Clone, PartialEq, Eq, Debug)]
// Now you can use `StateSwap<Self>` as a first-class custom storage type!
#[state(log_flush, storage(StateSwap<Self>))]
enum MySwappedState {
    X,
    Y,
}
