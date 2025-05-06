//! Store the [`NextState`] as a [`NextStateStack`].
//!
//! Enable the `stack` feature flag to use this module.
//!
//! This can be used to implement a back button, for example.

use alloc::{vec, vec::Vec};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{Commands, ResMut, SystemParamItem},
    world::{FromWorld, World},
};
use tiny_bail::prelude::*;

use crate::{
    next_state::{NextState, NextStateMut},
    state::State,
};

/// A [`NextState`] type that stores the [`State`] type `S` in a stack with the next state on top.
///
/// Using this as [`State::Next`] unlocks the [`NextStateStackMut`] extension trait for `S`.
#[derive(Resource, Component, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct NextStateStack<S: State<Next = Self>> {
    stack: Vec<Option<S>>,
    bases: Vec<usize>,
}

impl<S: State<Next = Self>> NextState for NextStateStack<S> {
    type State = S;

    type Param = ();

    fn empty() -> Self {
        Self {
            stack: Vec::new(),
            bases: Vec::new(),
        }
    }

    fn next_state<'s>(
        &'s self,
        _param: &'s SystemParamItem<Self::Param>,
    ) -> Option<&'s Self::State> {
        self.get()
    }
}

impl<S: State<Next = Self>> NextStateMut for NextStateStack<S> {
    type ParamMut = ();

    fn next_state_from_mut<'s>(
        &'s self,
        _param: &'s SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s Self::State> {
        self.get()
    }

    fn next_state_mut<'s>(
        &'s mut self,
        _param: &'s mut SystemParamItem<Self::ParamMut>,
    ) -> Option<&'s mut Self::State> {
        self.get_mut()
    }

    fn set_next_state(
        &mut self,
        _param: &mut SystemParamItem<Self::ParamMut>,
        state: Option<Self::State>,
    ) {
        self.set(state);
    }
}

impl<S: State<Next = Self> + FromWorld> FromWorld for NextStateStack<S> {
    fn from_world(world: &mut World) -> Self {
        Self::new(S::from_world(world))
    }
}

impl<S: State<Next = Self>> NextStateStack<S> {
    /// Create a new `NextStateStack` with an initial state.
    pub fn new(state: S) -> Self {
        Self {
            stack: vec![Some(state)],
            bases: Vec::new(),
        }
    }

    /// Create a new `NextStateStack` with an initial base state.
    pub fn with_base(state: S) -> Self {
        Self {
            stack: vec![Some(state)],
            bases: vec![1],
        }
    }

    /// Get the top base state index of the stack.
    pub fn base(&self) -> usize {
        self.bases.last().copied().unwrap_or_default()
    }

    /// Push a new base state index to the stack.
    pub fn acquire(&mut self) -> &mut Self {
        self.bases.push(self.stack.len());
        self
    }

    /// Pop the top base state index of the stack.
    pub fn release(&mut self) -> &mut Self {
        self.bases.pop();
        self
    }

    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.stack.last().and_then(|x| x.as_ref())
    }

    /// Get a mutable reference to the next state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.stack.last_mut().and_then(|x| x.as_mut())
    }

    /// Set the next state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        if self.stack.is_empty() {
            self.stack.push(state);
        } else {
            *self.stack.last_mut().unwrap() = state;
        }
    }

    /// Clear the stack down to the base state.
    pub fn clear(&mut self) -> &mut Self {
        self.stack.drain(self.base()..);
        self
    }

    /// Pop the stack if it's above the base state.
    pub fn pop(&mut self) -> &mut Self {
        if self.stack.len() > self.base() {
            self.stack.pop();
        }
        self
    }

    /// Push a state to the top of the stack.
    pub fn push(&mut self, state: S) -> &mut Self {
        self.stack.push(Some(state));
        self
    }
}

/// An extension trait for [`State`] types with [`NextStateStack`] as their [`NextState`] type.
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`NextStateStackMutExtClone`]
pub trait NextStateStackMut: State<Next = NextStateStack<Self>> {
    /// A system that pushes a new base state index to the stack.
    fn acquire(mut stack: ResMut<NextStateStack<Self>>) {
        stack.acquire();
    }

    /// A system that pops the top base state index of the stack.
    fn release(mut stack: ResMut<NextStateStack<Self>>) {
        stack.release();
    }

    /// A system that clears the stack down to the base state.
    fn clear(mut stack: ResMut<NextStateStack<Self>>) {
        stack.clear();
    }

    /// A system that pops the stack if it's above the base state.
    fn pop(mut stack: ResMut<NextStateStack<Self>>) {
        stack.pop();
    }
}

impl<S: State<Next = NextStateStack<S>>> NextStateStackMut for S {}

/// An extension trait for [`NextStateStackMut`] types that are also [`Clone`].
pub trait NextStateStackMutExtClone: NextStateStackMut + Clone {
    /// A system that pushes a state to the top of the stack.
    fn push(self) -> impl Fn(ResMut<NextStateStack<Self>>) {
        move |mut stack| {
            stack.push(self.clone());
        }
    }

    /// A system that clears and then pushes a state to the top of the stack.
    fn clear_push(self) -> impl Fn(ResMut<NextStateStack<Self>>) {
        move |mut stack| {
            stack.clear().push(self.clone());
        }
    }

    /// A system that pops and then pushes a state to the top of the stack.
    fn pop_push(self) -> impl Fn(ResMut<NextStateStack<Self>>) {
        move |mut stack| {
            stack.pop().push(self.clone());
        }
    }
}

impl<S: NextStateStackMut + Clone> NextStateStackMutExtClone for S {}

/// An extension trait for [`Commands`] that provides methods for operating on states with
/// [`NextStateStack`] as their `Next` type.
pub trait NextStateStackCommandsExt {
    /// Queues a [`Command`](bevy_ecs::system::Command) to push a new base state index to the stack.
    fn state_stack_acquire<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to pop the top base state index of the stack.
    fn state_stack_release<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to clear the stack down to the base state.
    fn state_stack_clear<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to pop the stack if it's above the base
    /// state.
    fn state_stack_pop<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to push a state to the top of the stack.
    fn state_stack_push<S: State<Next = NextStateStack<S>>>(&mut self, state: S) -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to clear and then push a state to the top of
    /// the stack.
    fn state_stack_clear_push<S: State<Next = NextStateStack<S>>>(&mut self, state: S)
    -> &mut Self;

    /// Queues a [`Command`](bevy_ecs::system::Command) to pop and then push a state to the top of
    /// the stack.
    fn state_stack_pop_push<S: State<Next = NextStateStack<S>>>(&mut self, state: S) -> &mut Self;
}

impl NextStateStackCommandsExt for Commands<'_, '_> {
    fn state_stack_acquire<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>()).acquire();
        });
        self
    }

    fn state_stack_release<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>()).release();
        });
        self
    }

    fn state_stack_clear<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>()).clear();
        });
        self
    }

    fn state_stack_pop<S: State<Next = NextStateStack<S>>>(&mut self) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>()).pop();
        });
        self
    }

    fn state_stack_push<S: State<Next = NextStateStack<S>>>(&mut self, state: S) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>()).push(state);
        });
        self
    }

    fn state_stack_clear_push<S: State<Next = NextStateStack<S>>>(
        &mut self,
        state: S,
    ) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>())
                .clear()
                .push(state);
        });
        self
    }

    fn state_stack_pop_push<S: State<Next = NextStateStack<S>>>(&mut self, state: S) -> &mut Self {
        self.queue(move |world: &mut World| {
            r!(world.get_resource_mut::<NextStateStack<S>>())
                .pop()
                .push(state);
        });
        self
    }
}
