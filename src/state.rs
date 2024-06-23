//! State traits and resources.
//!
//! Provided [`StateStorage`] types:
//!
//! - [`StateBuffer`](crate::buffer::StateBuffer) (default)
//! - [`StateStack`](crate::extra::stack::StateStack)
//! - [`StateSequence`](crate::extra::sequence::StateSequence)

use std::{fmt::Debug, marker::PhantomData};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::system::{ReadOnlySystemParam, Res, ResMut, Resource, SystemParam, SystemParamItem};

use crate::{
    access::{NextStateMut, NextStateRef, StateFlushMut},
    pattern::{
        AnyStatePattern, AnyStateTransPattern, FnStatePattern, FnStateTransPattern, StatePattern,
    },
};

/// A data type that can be used as a state.
///
/// The current state will be stored in the [`CurrentState`] resource,
/// and the next state will be stored in the specified [`StateStorage`].
///
/// This trait can be [derived](pyri_state_derive::State) or implemented manually:
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// enum MenuState { ... }
/// impl State for MenuState {
///     type Storage = StateBuffer<Self>;
/// }
/// ```
///
/// The derive macro would also implement [`AddState`](crate::extra::app::AddState) for `MenuState`.
///
/// See the following extension traits with additional bounds on `Self` and [`Self::Storage`](State::Storage):
///
/// - [`StateMut`]
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait State: 'static + Send + Sync + Sized {
    /// The [`StateStorage`] type that describes how this state will be stored in the ECS world.
    type Storage: StateStorage<State = Self>;

    /// The [`AnyStatePattern`] for this state type.
    const ANY: AnyStatePattern<Self> = AnyStatePattern(PhantomData);

    /// The [`AnyStateTransPattern`] for this state type.
    const ANY_TO_ANY: AnyStateTransPattern<Self> = AnyStateTransPattern(PhantomData);

    /// Create a [`FnStatePattern`] from a callback.
    fn with<F>(f: F) -> FnStatePattern<Self, F>
    where
        F: 'static + Send + Sync + Fn(&Self) -> bool,
    {
        FnStatePattern::new(f)
    }

    /// Create a [`FnStateTransPattern`] from a callback.
    fn when<F>(f: F) -> FnStateTransPattern<Self, F>
    where
        F: 'static + Send + Sync + Fn(&Self, &Self) -> bool,
    {
        FnStateTransPattern::new(f)
    }

    /// A run condition that checks if the current state is disabled.
    fn is_disabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_disabled()
    }

    /// A run condition that checks if the current state is enabled.
    fn is_enabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_enabled()
    }

    /// A run condition that checks if the next state will be disabled.
    fn will_be_disabled(next: NextStateRef<Self>) -> bool {
        next.get().is_none()
    }

    /// A run condition that checks if the next state will be enabled.
    fn will_be_enabled(next: NextStateRef<Self>) -> bool {
        next.get().is_some()
    }

    /// A system that triggers this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.trigger();
    }

    /// A system that resets the trigger for this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn relax(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.relax();
    }
}

/// An extension trait for [`State`] types with [mutable storage](StateStorageMut).
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait StateMut: State<Storage: StateStorageMut> {
    /// A system that disables the next state.
    fn disable(mut state: NextStateMut<Self>) {
        state.set(None);
    }
}

/// An extension trait for [`StateMut`] types that also implement [`Clone`].
pub trait StateMutExtClone: StateMut + Clone {
    /// Build a system that enables the next state with a specific value if it's disabled.
    fn enable(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            }
        }
    }

    /// Build a system that toggles the next state between disabled and enabled with a specific value.
    fn toggle(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            } else {
                state.disable();
            }
        }
    }

    /// Build a system that enables the next state with a specific value.
    fn enter(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.set(Some(self.clone()));
        }
    }

    /// A system that resets the next state to the current state and relaxes the trigger to flush.
    fn reset(mut state: StateFlushMut<Self>) {
        state.reset();
    }

    /// A system that resets the next state to the current state and triggers a flush.
    fn refresh(mut state: StateFlushMut<Self>) {
        state.refresh();
    }
}

impl<S: StateMut + Clone> StateMutExtClone for S {}

/// An extension trait for [`StateMut`] types that also implement [`Default`].
pub trait StateMutExtDefault: StateMut + Default {
    /// A system that enables the next state with the default value if it's disabled.
    fn enable_default(mut state: NextStateMut<Self>) {
        state.enable_default();
    }

    /// A system that toggles the next state between disabled and enabled with the default value.
    fn toggle_default(mut state: NextStateMut<Self>) {
        state.toggle_default();
    }

    /// A system that enables the next state with the default value.
    fn enter_default(mut state: NextStateMut<Self>) {
        state.enter_default();
    }
}

impl<S: StateMut + Default> StateMutExtDefault for S {}

/// A resource that contains the current value of the [`State`] type `S`.
///
/// Use [`StateFlushRef`](crate::access::StateFlushRef) or [`StateFlushMut`] in a system to access
/// the next state alongside the current state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: State>(
    /// The current state, or `None` if disabled.
    pub Option<S>,
);

impl<S: State> Default for CurrentState<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: State> CurrentState<S> {
    /// Create a disabled `CurrentState`.
    pub fn disabled() -> Self {
        Self(None)
    }

    /// Create an enabled `CurrentState` with a specific value.
    pub fn enabled(value: S) -> Self {
        Self(Some(value))
    }

    /// Get a read-only reference to the current state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.0.as_ref()
    }

    /// Get a read-only reference to the current state, or panic if it's disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Check if the current state is disabled.
    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the current state is enabled.
    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the current state is enabled and matches a specific [`StatePattern`].
    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }
}

/// A resource that determines whether the [`State`] type `S` will flush in the
/// [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct TriggerStateFlush<S: State>(
    /// The flush flag. If true, `S` will flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub bool,
    PhantomData<S>,
);

impl<S: State> Default for TriggerStateFlush<S> {
    fn default() -> Self {
        Self(false, PhantomData)
    }
}

impl<S: State> TriggerStateFlush<S> {
    /// Trigger `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) {
        self.0 = true;
    }

    /// Reset the trigger for `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn relax(&mut self) {
        self.0 = false;
    }
}

/// A resource that determines the next state for the [`State`] type `S`.
///
/// Use [`NextStateRef`] or [`StateFlushRef`](crate::access::StateFlushRef)
/// in a system for read-only access to the next state.
///
/// See [`StateStorageMut`] for mutable storage.
///
/// # Example
///
/// The default storage type is [`StateBuffer`](crate::buffer::StateBuffer). You can
/// set a different storage type in the [derive macro](pyri_state_derive::State):
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// #[state(storage(StateStack<Self>))]
/// enum MenuState { ... }
/// ```
pub trait StateStorage: Resource {
    /// The stored [`State`] type.
    type State: State;

    /// A [`ReadOnlySystemParam`] with read-only access to the next state.
    type Param: ReadOnlySystemParam;

    /// Create an empty storage.
    ///
    /// Used in [`AppExtState::add_state`](crate::extra::app::AppExtState::add_state).
    fn empty() -> Self;

    /// Get a read-only reference to the next state, or `None` if disabled.
    fn get_state<'s>(&'s self, param: &'s SystemParamItem<Self::Param>) -> Option<&'s Self::State>;
}

/// A [`StateStorage`] type that allows `S` to be mutated directly as a [`StateMut`].
///
/// Use [`NextStateMut`] or [`StateFlushMut`] in a system for mutable access to the next state.
pub trait StateStorageMut: StateStorage {
    /// A [`SystemParam`] with mutable access to the next state.
    type ParamMut: SystemParam;

    /// Get a reference to the next state, or `None` if disabled.
    fn get_state_from_mut<'s>(
        &'s self,
        param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s Self::State>;

    /// Get a mutable reference to the next state, or `None` if disabled.
    fn get_state_mut<'s>(
        &'s mut self,
        param: &'s mut SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut Self::State>;

    /// Set the next state to a new value, or `None` to disable.
    fn set_state(
        &mut self,
        param: &mut SystemParamItem<Self::ParamMut>,
        state: Option<Self::State>,
    );
}

// A `State` is `StateMut` if its `StateStorage` is `StateStorageMut`
impl<S: State<Storage: StateStorageMut>> StateMut for S {}
