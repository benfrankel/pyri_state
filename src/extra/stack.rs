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
pub struct StateStack<S: RawState> {
    base: Option<S>,
    stack: Vec<Option<S>>,
}

impl<S: RawState> StateStorage for StateStack<S> {}

impl<S: RawState> GetStateStorage<S> for StateStack<S> {
    type Param = SRes<Self>;

    fn get_state<'a>(param: &'a SystemParamItem<Self::Param>) -> Option<&'a S> {
        param.get()
    }
}

impl<S: RawState> SetStateStorage<S> for StateStack<S> {
    type Param = SResMut<Self>;

    fn get_state_from_mut<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.get()
    }

    fn get_state_mut<'s>(param: &'s mut SystemParamItem<Self::Param>) -> Option<&'s mut S> {
        param.get_mut()
    }

    fn set_state(param: &mut SystemParamItem<Self::Param>, state: Option<S>) {
        param.set(state);
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
        Self {
            base: None,
            stack: Vec::new(),
        }
    }

    pub fn new(state: S) -> Self {
        Self {
            base: None,
            stack: vec![Some(state)],
        }
    }

    pub fn with_base(base: S) -> Self {
        Self {
            base: Some(base),
            stack: Vec::new(),
        }
    }

    pub fn get(&self) -> Option<&S> {
        self.stack.last().map_or(self.base.as_ref(), |x| x.as_ref())
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.stack
            .last_mut()
            .map_or(self.base.as_mut(), |x| x.as_mut())
    }

    pub fn set(&mut self, state: Option<S>) {
        *self.stack.last_mut().unwrap_or(&mut self.base) = state;
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn push(&mut self, state: S) {
        self.stack.push(Some(state));
    }

    pub fn clear_push(&mut self, state: S) {
        self.clear();
        self.push(state);
    }

    pub fn pop_push(&mut self, state: S) {
        self.pop();
        self.push(state);
    }
}

pub trait StateStackMut: RawState {
    fn clear(mut stack: ResMut<StateStack<Self>>) {
        stack.clear();
    }

    fn pop(mut stack: ResMut<StateStack<Self>>) {
        stack.pop();
    }
}

impl<S: RawState<Storage = StateStack<S>>> StateStackMut for S {}

pub trait StateStackMutExtClone: StateStackMut + Clone {
    fn push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| stack.push(self.clone())
    }

    fn clear_push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| stack.clear_push(self.clone())
    }

    fn pop_push(self) -> impl Fn(ResMut<StateStack<Self>>) {
        move |mut stack| stack.pop_push(self.clone())
    }
}

impl<S: StateStackMut + Clone> StateStackMutExtClone for S {}
