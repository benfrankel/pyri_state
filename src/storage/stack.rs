use bevy_ecs::{
    system::{
        lifetimeless::{SRes, SResMut},
        ResMut, Resource, SystemParamItem,
    },
    world::{FromWorld, World},
};

use crate::{
    state::RawState,
    storage::{GetStateStorage, SetStateStorage, StateStorage},
};

// A state stack with the current state on top.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    // TODO: In bevy 0.14 this will be possible.
    //reflect(Resource)
)]
pub struct StateStack<S: RawState>(pub Vec<S>);

impl<S: RawState> StateStorage for StateStack<S> {}

impl<S: RawState> GetStateStorage<S> for StateStack<S> {
    type Param = SRes<Self>;

    fn get_state<'a>(param: &'a SystemParamItem<Self::Param>) -> Option<&'a S> {
        param.0.last()
    }
}

impl<S: RawState> SetStateStorage<S> for StateStack<S> {
    type Param = SResMut<Self>;

    fn get_state_from_mut<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.0.last()
    }

    fn get_state_mut<'s>(param: &'s mut SystemParamItem<Self::Param>) -> Option<&'s mut S> {
        param.0.last_mut()
    }

    fn set_state(param: &mut SystemParamItem<Self::Param>, state: Option<S>) {
        match state {
            Some(value) => {
                param.0.pop();
                param.0.push(value);
            }
            None => param.0.clear(),
        }
    }
}

#[cfg(feature = "bevy_app")]
impl<S: crate::app::AddState<AddStorage = Self>> crate::app::AddStateStorage for StateStack<S> {
    type AddState = S;

    fn add_state_storage(app: &mut bevy_app::App, storage: Option<Self>) {
        app.insert_resource(storage.unwrap_or_else(StateStack::empty));
    }
}

impl<S: RawState + FromWorld> FromWorld for StateStack<S> {
    fn from_world(world: &mut World) -> Self {
        Self::new(S::from_world(world))
    }
}

impl<S: RawState> StateStack<S> {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn new(state: S) -> Self {
        Self(vec![state])
    }
}

pub trait StateStackMut: RawState {
    fn clear(mut stack: ResMut<StateStack<Self>>) {
        stack.0.clear();
    }

    fn pop(mut stack: ResMut<StateStack<Self>>) {
        stack.0.pop();
    }
}

impl<S: RawState<Storage = StateStack<S>>> StateStackMut for S {}

pub trait StateStackMutExtClone: StateStackMut + Clone {
    fn push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| {
            stack.0.push(self.clone());
        }
    }

    fn clear_push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| {
            stack.0.clear();
            stack.0.push(self.clone());
        }
    }

    fn pop_push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| {
            stack.0.pop();
            stack.0.push(self.clone());
        }
    }
}

impl<S: StateStackMut + Clone> StateStackMutExtClone for S {}
