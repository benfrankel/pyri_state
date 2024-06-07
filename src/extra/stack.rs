//! TODO: Module-level documentation

use bevy_ecs::{
    system::{
        lifetimeless::{SRes, SResMut},
        ResMut, Resource, SystemParamItem,
    },
    world::{FromWorld, World},
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    state::State,
    storage::{StateStorage, StateStorageMut},
};

/// A [`StateStorage`] type that stores `S` in a stack with the next state on top.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateStack<S: State> {
    stack: Vec<Option<S>>,
    bases: Vec<usize>,
}

impl<S: State> StateStorage<S> for StateStack<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.get()
    }
}

impl<S: State> StateStorageMut<S> for StateStack<S> {
    type ParamMut = SResMut<Self>;

    fn get_state_from_mut<'s>(param: &'s SystemParamItem<Self::ParamMut>) -> Option<&'s S> {
        param.get()
    }

    fn get_state_mut<'s>(param: &'s mut SystemParamItem<Self::ParamMut>) -> Option<&'s mut S> {
        param.get_mut()
    }

    fn set_state(param: &mut SystemParamItem<Self::ParamMut>, state: Option<S>) {
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

impl<S: State + FromWorld> FromWorld for StateStack<S> {
    fn from_world(world: &mut World) -> Self {
        Self::new(S::from_world(world))
    }
}

impl<S: State> StateStack<S> {
    pub fn empty() -> Self {
        Self {
            stack: Vec::new(),
            bases: Vec::new(),
        }
    }

    pub fn new(state: S) -> Self {
        Self {
            stack: vec![Some(state)],
            bases: Vec::new(),
        }
    }

    pub fn with_base(state: S) -> Self {
        Self {
            stack: vec![Some(state)],
            bases: vec![1],
        }
    }

    pub fn base(&self) -> usize {
        self.bases.last().copied().unwrap_or_default()
    }

    pub fn acquire(&mut self) -> &mut Self {
        self.bases.push(self.stack.len());
        self
    }

    pub fn release(&mut self) -> &mut Self {
        self.bases.pop();
        self
    }

    pub fn get(&self) -> Option<&S> {
        self.stack.last().and_then(|x| x.as_ref())
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.stack.last_mut().and_then(|x| x.as_mut())
    }

    pub fn set(&mut self, state: Option<S>) {
        if self.stack.is_empty() {
            self.stack.push(state);
        } else {
            *self.stack.last_mut().unwrap() = state;
        }
    }

    pub fn clear(&mut self) {
        self.stack.drain(self.base()..);
    }

    pub fn pop(&mut self) {
        if self.stack.len() > self.base() {
            self.stack.pop();
        }
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

pub trait StateStackMut: State {
    fn acquire(mut stack: ResMut<StateStack<Self>>) {
        stack.acquire();
    }

    fn release(mut stack: ResMut<StateStack<Self>>) {
        stack.release();
    }

    fn clear(mut stack: ResMut<StateStack<Self>>) {
        stack.clear();
    }

    fn pop(mut stack: ResMut<StateStack<Self>>) {
        stack.pop();
    }
}

impl<S: State<Storage = StateStack<S>>> StateStackMut for S {}

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
