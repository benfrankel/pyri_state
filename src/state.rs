//! State traits and components.
//!
//! Provided [`NextState`] types:
//!
//! - [`NextStateBuffer`](crate::buffer::NextStateBuffer) (default)
//! - [`NextStateStack`](crate::extra::stack::NextStateStack)
//! - [`NextStateIndex`](crate::extra::sequence::NextStateIndex)

use std::{fmt::Debug, marker::PhantomData};

use bevy_ecs::{
    component::Component,
    system::{ReadOnlySystemParam, Res, ResMut, Resource, SystemParam, SystemParamItem},
};

use crate::{
    access::{CurrentRef, FlushMut, NextMut, NextRef},
    pattern::{AnyStatePattern, AnyStateTransPattern, FnStatePattern, FnStateTransPattern},
    prelude::FlushRef,
};

/// A [`Resource`] that can be used as a state.
///
/// This trait can be [derived](pyri_state_derive::State) or implemented manually:
///
/// ```rust
/// #[derive(Resource, State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// #[derive(Resource)]
/// enum MenuState { ... }
/// impl State for MenuState {
///     type Next = NextStateBuffer<Self>;
/// }
/// ```
///
/// The derive macro would also implement
/// [`RegisterState`](crate::extra::app::RegisterState) for `MenuState`.
///
/// See the following extension traits with additional bounds on `Self` and [`Self::Next`](State::Next):
///
/// - [`StateExtEq`]
/// - [`StateMut`]
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait State: Resource + Sized {
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

    /// A run condition that checks if this state type is triggered to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn is_triggered(trigger: Res<TriggerStateFlush<Self>>) -> bool {
        trigger.0
    }

    /// A system that triggers this state type to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.0 = true;
    }

    /// A system that resets the trigger for this state type to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn reset_trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.0 = false;
    }
}

/// An extention trait for [`State`] types that also implement [`Eq`].
pub trait StateExtEq: State + Eq {
    /// A run condition that checks if this state type will change if triggered.
    fn will_change(state: FlushRef<Self>) -> bool {
        state.will_change()
    }
}

impl<S: State + Eq> StateExtEq for S {}

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

impl<S: State<Next: NextStateMut>> StateMut for S {}

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

/// A marker trait for [`State`] types that can be stored as components on entities.
pub trait LocalState: State<Next: Component> + Component {}

impl<S: State<Next: Component> + Component> LocalState for S {}

/// A [`Resource`] / [`Component`] that determines whether the [`State`] type `S` will flush in the
/// [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(Resource, Component, Debug)]
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

/// A [`Resource`] that determines the next state for [`Self::State`].
///
/// Use [`NextRef`] or [`FlushRef`] in a system for read-only access to the next state.
///
/// See [`NextStateMut`] for mutable next state types.
///
/// # Example
///
/// The default `NextState` type is [`NextStateBuffer`](crate::buffer::NextStateBuffer).
/// You can set a different `NextState` type in the [derive macro](pyri_state_derive::State):
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// #[state(next(NextStateStack<Self>))]
/// enum MenuState { ... }
/// ```
pub trait NextState: Resource {
    /// The stored [`State`] type.
    type State: State;

    /// A [`ReadOnlySystemParam`] to help access the next state if needed.
    ///
    /// If the next state is stored within `Self`, this can be set to `()`.
    type Param: ReadOnlySystemParam;

    /// Create an empty next state instance.
    ///
    /// Used in [`AppExtState::add_state`](crate::extra::app::AppExtState::add_state).
    fn empty() -> Self;

    /// Get a read-only reference to the next state, or `None` if disabled.
    fn next_state<'s>(&'s self, param: &'s SystemParamItem<Self::Param>)
        -> Option<&'s Self::State>;
}

/// A [`NextState`] type that allows [`Self::State`](NextState::State) to be mutated directly.
///
/// Use [`NextMut`] or [`FlushMut`] in a system for mutable access to the next state.
pub trait NextStateMut: NextState {
    /// A [`SystemParam`] to help mutably access the next state if needed.
    ///
    /// If the next state is stored within `Self`, this can be set to `()`.
    type ParamMut: SystemParam;

    /// Get a reference to the next state, or `None` if disabled.
    fn next_state_from_mut<'s>(
        &'s self,
        param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s Self::State>;

    /// Get a mutable reference to the next state, or `None` if disabled.
    fn next_state_mut<'s>(
        &'s mut self,
        param: &'s mut SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut Self::State>;

    /// Set the next state to a new value, or `None` to disable.
    fn set_next_state(
        &mut self,
        param: &mut SystemParamItem<Self::ParamMut>,
        state: Option<Self::State>,
    );
}
