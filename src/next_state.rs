//! [`NextState`] trait, extension traits and types.
//!
//! Provided `NextState` types:
//!
//! - [`NextStateBuffer`](buffer::NextStateBuffer) (default)
//! - [`NextStateStack`](stack::NextStateStack)
//! - [`NextStateIndex`](sequence::NextStateIndex)

use core::marker::PhantomData;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ReadOnlySystemParam, SystemParam, SystemParamItem},
};

use crate::state::State;

pub mod buffer;
#[cfg(feature = "sequence")]
pub mod sequence;
#[cfg(feature = "stack")]
pub mod stack;

/// A [`Resource`] / [`Component`] that determines whether the [`State`] type `S` will flush in the
/// [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(Resource, Component, Debug)]
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

/// A [`Resource`] that determines the next state for [`Self::State`].
///
/// Use [`NextRef`](crate::access::NextRef) or [`FlushRef`](crate::access::FlushRef)
/// in a system for read-only access to the next state.
///
/// See [`NextStateMut`] for mutable next state types.
///
/// # Example
///
/// The default `NextState` type is [`NextStateBuffer`](buffer::NextStateBuffer).
/// You can set a different `NextState` type in the [derive macro](pyri_state_derive::State):
///
/// ```
/// # use pyri_state::prelude::*;
/// #
/// #[derive(State, Clone, PartialEq, Eq)]
/// #[state(next(NextStateStack<Self>))]
/// enum Menu { /* ... */ }
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
    /// Used in [`AppExtState::add_state`](crate::setup::AppExtState::add_state).
    fn empty() -> Self;

    /// Get a read-only reference to the next state, or `None` if disabled.
    fn next_state<'s>(&'s self, param: &'s SystemParamItem<Self::Param>)
    -> Option<&'s Self::State>;
}

/// A [`NextState`] type that allows [`Self::State`](NextState::State) to be mutated directly.
///
/// Use [`NextMut`](crate::access::NextMut) or [`FlushMut`](crate::access::FlushMut)
/// in a system for mutable access to the next state.
pub trait NextStateMut: NextState {
    /// A [`SystemParam`] to help mutably access the next state if needed.
    ///
    /// If the next state is stored within `Self`, this can be set to `()`.
    type ParamMut: SystemParam;

    /// Get a reference to the next state if enabled.
    fn next_state_from_mut<'s>(
        &'s self,
        param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s Self::State>;

    /// Get a mutable reference to the next state if enabled.
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
