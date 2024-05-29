use bevy_ecs::system::{
    lifetimeless::{SRes, SResMut},
    ResMut, Resource, SystemParamItem,
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    app::AddStateStorage,
    state::RawState,
    storage::{GetStateStorage, SetStateStorage, StateStorage},
};

// A state stack with the current state on top.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateStack<S: RawState>(pub Vec<S>);

impl<S: RawState> StateStorage<S> for StateStack<S> {}

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
impl<S: RawState> AddStateStorage<S> for StateStack<S> {
    fn add_state_storage(app: &mut bevy_app::App, state: Option<S>) {
        app.insert_resource(match state {
            Some(value) => StateStack::new(value),
            None => StateStack::empty(),
        });
    }
}

impl<S: RawState> Default for StateStack<S> {
    fn default() -> Self {
        Self::empty()
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
