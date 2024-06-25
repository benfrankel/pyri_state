//! State traits and components.
//!
//! Provided [`NextState`] types:
//!
//! - [`StateBuffer`](crate::buffer::StateBuffer) (default)
//! - [`StateStack`](crate::extra::stack::StateStack)
//! - [`StateSequence`](crate::extra::sequence::StateSequence)

use std::{fmt::Debug, marker::PhantomData};

use bevy_ecs::{
    component::Component,
    query::With,
    system::{Query, ReadOnlySystemParam, SystemParam, SystemParamItem},
};

use crate::{
    access::{CurrentRef, FlushMut, GlobalStates, NextMut, NextRef},
    pattern::{
        AnyStatePattern, AnyStateTransPattern, FnStatePattern, FnStateTransPattern, StatePattern,
    },
};

/// A data type that can be used as a state.
///
/// The current state will be stored in the [`CurrentState`] component,
/// and the next state will be stored in the specified [`NextState`] component.
///
/// This trait can be [derived](pyri_state_derive::State) or implemented manually:
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// enum MenuState { ... }
/// impl State for MenuState {
///     type Next = StateBuffer<Self>;
/// }
/// ```
///
/// The derive macro would also implement [`AddState`](crate::extra::app::AddState) for `MenuState`.
///
/// See the following extension traits with additional bounds on `Self` and [`Self::Next`](State::Next):
///
/// - [`StateMut`]
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait State: 'static + Send + Sync + Sized {
    /// The [`NextState`] type that determines the next state for this state type.
    type Next: NextState<State = Self>;

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
    fn is_disabled(state: CurrentRef<Self>) -> bool {
        state.is_disabled()
    }

    /// A run condition that checks if the current state is enabled.
    fn is_enabled(state: CurrentRef<Self>) -> bool {
        state.is_enabled()
    }

    /// A run condition that checks if the next state will be disabled.
    fn will_be_disabled(next: NextRef<Self>) -> bool {
        next.get().is_none()
    }

    /// A run condition that checks if the next state will be enabled.
    fn will_be_enabled(next: NextRef<Self>) -> bool {
        next.get().is_some()
    }

    /// A system that triggers this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn trigger(mut trigger: Query<&mut TriggerStateFlush<Self>, With<GlobalStates>>) {
        trigger.single_mut().trigger();
    }

    /// A system that resets the trigger for this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn relax(mut trigger: Query<&mut TriggerStateFlush<Self>, With<GlobalStates>>) {
        trigger.single_mut().relax();
    }
}

/// An extension trait for [`State`] types with [mutable `NextState`](NextStateMut).
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait StateMut: State<Next: NextStateMut> {
    /// A system that disables the next state.
    fn disable(mut state: NextMut<Self>) {
        state.set(None);
    }
}

/// An extension trait for [`StateMut`] types that also implement [`Clone`].
pub trait StateMutExtClone: StateMut + Clone {
    /// Build a system that enables the next state with a specific value if it's disabled.
    fn enable(self) -> impl Fn(NextMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            }
        }
    }

    /// Build a system that toggles the next state between disabled and enabled with a specific value.
    fn toggle(self) -> impl Fn(NextMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            } else {
                state.disable();
            }
        }
    }

    /// Build a system that enables the next state with a specific value.
    fn enter(self) -> impl Fn(NextMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.set(Some(self.clone()));
        }
    }

    /// A system that resets the next state to the current state and relaxes the trigger to flush.
    fn reset(mut state: FlushMut<Self>) {
        state.reset();
    }

    /// A system that resets the next state to the current state and triggers a flush.
    fn refresh(mut state: FlushMut<Self>) {
        state.refresh();
    }
}

impl<S: StateMut + Clone> StateMutExtClone for S {}

/// An extension trait for [`StateMut`] types that also implement [`Default`].
pub trait StateMutExtDefault: StateMut + Default {
    /// A system that enables the next state with the default value if it's disabled.
    fn enable_default(mut state: NextMut<Self>) {
        state.enable_default();
    }

    /// A system that toggles the next state between disabled and enabled with the default value.
    fn toggle_default(mut state: NextMut<Self>) {
        state.toggle_default();
    }

    /// A system that enables the next state with the default value.
    fn enter_default(mut state: NextMut<Self>) {
        state.enter_default();
    }
}

impl<S: StateMut + Default> StateMutExtDefault for S {}

/// A [`Component`] that contains the current value of the [`State`] type `S`.
///
/// Use [`FlushRef`](crate::access::FlushRef) or [`FlushMut`] in a system to access
/// the next state alongside the current state.
#[derive(Component, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
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

    /// Get a mutable reference to the current state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.0.as_mut()
    }

    /// Set the current state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        self.0 = state;
    }

    /// Get a read-only reference to the current state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Get a mutable reference to the current state, or panic if disabled.
    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
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

/// A [`Component`] that determines whether the [`State`] type `S` will flush in the
/// [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(Component, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
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

/// A [`Component`] that determines the next state for the [`State`] type `S`.
///
/// Use [`NextRef`] or [`FlushRef`](crate::access::FlushRef)
/// in a system for read-only access to the next state.
///
/// See [`NextStateMut`] for mutable next state types.
///
/// # Example
///
/// The default next state type is [`StateBuffer`](crate::buffer::StateBuffer). You can
/// set a different next state type in the [derive macro](pyri_state_derive::State):
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// #[state(next(StateStack<Self>))]
/// enum MenuState { ... }
/// ```
pub trait NextState: Component {
    /// The stored [`State`] type.
    type State: State;

    /// A [`ReadOnlySystemParam`] to help access the next state if needed.
    ///
    /// If the next state is stored within `Self`, this can be set to `()`.
    type Param: ReadOnlySystemParam;

    /// Create an empty next state component.
    ///
    /// Used in [`AppExtState::add_state`](crate::extra::app::AppExtState::add_state).
    fn empty() -> Self;

    /// Get a read-only reference to the next state, or `None` if disabled.
    fn get_state<'s>(&'s self, param: &'s SystemParamItem<Self::Param>) -> Option<&'s Self::State>;
}

/// A [`NextState`] type that allows `S` to be mutated directly as a [`StateMut`].
///
/// Use [`NextMut`] or [`FlushMut`] in a system for mutable access to the next state.
pub trait NextStateMut: NextState {
    /// A [`SystemParam`] to help mutably access the next state if needed.
    ///
    /// If the next state is stored within `Self`, this can be set to `()`.
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

// A `State` is `StateMut` if its `NextState` is `NextStateMut`
impl<S: State<Next: NextStateMut>> StateMut for S {}
