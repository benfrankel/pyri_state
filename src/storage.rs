//! State storage types and traits.
//!
//! Provided [`StateStorage`] types:
//!
//! - [`StateBuffer`] (default)
//! - [`StateStack`](crate::extra::stack::StateStack)
//! - [`StateSequence`](crate::extra::sequence::StateSequence)

use bevy_ecs::{
    system::{
        lifetimeless::{SRes, SResMut},
        ReadOnlySystemParam, Resource, SystemParam, SystemParamItem,
    },
    world::{FromWorld, World},
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::StatePattern,
    state::{State, StateMut},
};

/// A type that describes how the [`State`] type `S` will be stored in the ECS world.
///
/// Use [`NextStateRef`](crate::state::NextStateRef) or [`StateFlushRef`](crate::state::StateFlushRef)
/// in a system for read-only access to the next state.
///
/// See [`StateStorageMut`] for mutable storage.
///
/// # Example
///
/// The default storage type is [`StateBuffer`]. You can set a different storage type in the
/// [derive macro](pyri_state_derive::State):
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// #[state(storage(StateStack<Self>))]
/// enum MenuState { ... }
/// ```
pub trait StateStorage<S: State> {
    /// A [`ReadOnlySystemParam`] with read-only access to the next state.
    type Param: ReadOnlySystemParam;

    /// Get a read-only reference to the next state, or `None` if disabled.
    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S>;
}

/// A [`StateStorage`] type that allows `S` to be mutated directly as a [`StateMut`].
///
/// Use [`NextStateMut`](crate::state::NextStateMut) or [`StateFlushMut`](crate::state::StateFlushMut)
/// in a system for mutable access to the next state.
pub trait StateStorageMut<S: State>: StateStorage<S> {
    /// A [`SystemParam`] with mutable access to the next state.
    type ParamMut: SystemParam;

    /// Get a reference to the next state, or `None` if disabled.
    fn get_state_from_mut<'s>(param: &'s SystemParamItem<Self::ParamMut>) -> Option<&'s S>;

    /// Get a mutable reference to the next state, or `None` if disabled.
    fn get_state_mut<'s>(param: &'s mut SystemParamItem<Self::ParamMut>) -> Option<&'s mut S>;

    /// Set the next state to a new value, or `None` to disable.
    fn set_state(param: &mut SystemParamItem<Self::ParamMut>, state: Option<S>);
}

impl<S: State> StateMut for S
where
    S::Storage: StateStorageMut<S>,
{
    type StorageMut = S::Storage;
}

/// The default [`StateStorage`] type, storing the next state in a resource.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateBuffer<S: State>(
    /// The next state, or `None` if disabled.
    pub Option<S>,
);

impl<S: State> StateStorage<S> for StateBuffer<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.get()
    }
}

impl<S: State> StateStorageMut<S> for StateBuffer<S> {
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
impl<S: crate::app::AddState<AddStorage = Self>> crate::app::AddStateStorage for StateBuffer<S> {
    type AddState = S;

    fn add_state_storage(app: &mut bevy_app::App, storage: Option<Self>) {
        app.insert_resource(storage.unwrap_or_else(StateBuffer::disabled));
    }
}

impl<S: State + FromWorld> FromWorld for StateBuffer<S> {
    fn from_world(world: &mut World) -> Self {
        Self::enabled(S::from_world(world))
    }
}

impl<S: State> StateBuffer<S> {
    /// Create a disabled `StateBuffer`.
    pub fn disabled() -> Self {
        Self(None)
    }

    /// Create an enabled `StateBuffer` with a specific value.
    pub fn enabled(state: S) -> Self {
        Self(Some(state))
    }

    /// Get a reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.0.as_ref()
    }

    /// Get a mutable reference to the next state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.0.as_mut()
    }

    /// Set the next state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        self.0 = state;
    }

    /// Get a reference to the next state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Get a mutable reference to the next state, or panic if disabled.
    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    /// Check if the next state is disabled.
    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the next state is enabled.
    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the next state is enabled and matches a specific [`StatePattern`].
    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }

    /// Disable the next state.
    pub fn disable(&mut self) {
        self.0 = None;
    }

    /// Enable the next state with a specific value if it's disabled, and
    /// return a mutable reference to the next state.
    pub fn enable(&mut self, state: S) -> &mut S {
        self.0.get_or_insert(state)
    }

    /// Toggle between disabled and enabled with a specific value.
    pub fn toggle(&mut self, state: S) {
        if self.is_enabled() {
            self.disable();
        } else {
            self.enter(state);
        }
    }

    /// Enable the next state with a specific value, and
    /// return a mutable reference to the next state.
    pub fn enter(&mut self, value: S) -> &mut S {
        self.0.insert(value)
    }
}
