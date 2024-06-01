use bevy_ecs::{
    system::{
        lifetimeless::{SRes, SResMut},
        ReadOnlySystemParam, Resource, SystemParam, SystemParamItem,
    },
    world::{FromWorld, World},
};

use crate::{
    pattern::StatePattern,
    state::{StateMut, State_},
};

// Trait for types that can be used as a state's storage.
pub trait StateStorage<S: State_> {
    type Param: ReadOnlySystemParam;

    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S>;
}

pub trait StateStorageMut<S: State_> {
    type Param: SystemParam;

    fn get_state_from_mut<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S>;

    fn get_state_mut<'s>(param: &'s mut SystemParamItem<Self::Param>) -> Option<&'s mut S>;

    fn set_state(param: &mut SystemParamItem<Self::Param>, state: Option<S>);
}

// A state is `StateMut` if its storage is `StateStorageMut`.
impl<S: State_> StateMut for S
where
    S::Storage: StateStorageMut<S>,
{
    type StorageMut = S::Storage;
}

// TODO: Update this comment and the other one.
// The mutable half of the double-buffered state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    // TODO: In bevy 0.14 this will be possible.
    //reflect(Resource)
)]
pub struct StateBuffer<S: State_>(pub Option<S>);

impl<S: State_> StateStorage<S> for StateBuffer<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.get()
    }
}

impl<S: State_> StateStorageMut<S> for StateBuffer<S> {
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
impl<S: crate::app::AddState<AddStorage = Self>> crate::app::AddStateStorage for StateBuffer<S> {
    type AddState = S;

    fn add_state_storage(app: &mut bevy_app::App, storage: Option<Self>) {
        app.insert_resource(storage.unwrap_or_else(StateBuffer::disabled));
    }
}

impl<S: State_ + FromWorld> FromWorld for StateBuffer<S> {
    fn from_world(world: &mut World) -> Self {
        Self::enabled(S::from_world(world))
    }
}

impl<S: State_> StateBuffer<S> {
    pub fn disabled() -> Self {
        Self(None)
    }

    pub fn enabled(value: S) -> Self {
        Self(Some(value))
    }

    pub fn get(&self) -> Option<&S> {
        self.0.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.0.as_mut()
    }

    pub fn set(&mut self, state: Option<S>) {
        self.0 = state;
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }

    pub fn disable(&mut self) {
        self.0 = None;
    }

    // Enter the given state if disabled.
    pub fn enable(&mut self, value: S) -> &mut S {
        self.0.get_or_insert(value)
    }

    // Toggle between the given state and disabled.
    pub fn toggle(&mut self, value: S) {
        if self.is_enabled() {
            self.disable();
        } else {
            self.enter(value);
        }
    }

    pub fn enter(&mut self, value: S) -> &mut S {
        self.0.insert(value)
    }
}
