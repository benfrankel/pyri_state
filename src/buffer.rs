use std::{fmt::Debug, hash::Hash};

use bevy_ecs::{
    schedule::States,
    system::{Res, ResMut, Resource, SystemParam},
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::state::{ContainsState, RawState, State_};

// Wrapper for compatibility with bevy states
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State_ + Hash + Debug>(pub Option<S>);

impl<S: State_ + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}

// The immutable half of the double-buffered state.
// This should never be accessed mutably during normal usage.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: RawState> {
    pub inner: Option<S>,
}

impl<S: RawState> Default for CurrentState<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: RawState> CurrentState<S> {
    pub fn new(inner: Option<S>) -> Self {
        Self { inner }
    }

    pub fn disabled() -> Self {
        Self::new(None)
    }

    pub fn enabled(value: S) -> Self {
        Self::new(Some(value))
    }

    pub fn get(&self) -> Option<&S> {
        self.inner.as_ref()
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn is_disabled(&self) -> bool {
        self.inner.is_none()
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.is_some()
    }

    pub fn is_in<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), Some(x) if set.contains_state(x))
    }
}

// The mutable half of the double-buffered state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct NextState_<S: RawState> {
    pub inner: Option<S>,
    pub flush: bool,
}

impl<S: RawState> Default for NextState_<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: RawState + Default> NextState_<S> {
    // Set the next state to the default state from disabled.
    pub fn enable(&mut self) -> &mut S {
        self.inner.get_or_insert_with(|| S::default())
    }

    // Toggle between the default state and disabled.
    pub fn toggle(&mut self) {
        if self.will_be_enabled() {
            self.disable();
        } else {
            self.restart();
        }
    }

    // Set the next state to the default state and enable flush.
    pub fn restart(&mut self) -> &mut S {
        self.set_flush(true).enter(S::default())
    }
}

impl<S: RawState> NextState_<S> {
    pub fn new(inner: Option<S>) -> Self {
        Self {
            inner,
            flush: false,
        }
    }

    pub fn disabled() -> Self {
        Self::new(None)
    }

    pub fn enabled(value: S) -> Self {
        Self::new(Some(value))
    }

    pub fn get(&self) -> Option<&S> {
        self.inner.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.inner.as_mut()
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    // TODO: Consider renaming to is_disabled etc.
    pub fn will_be_disabled(&self) -> bool {
        self.inner.is_none()
    }

    pub fn will_be_enabled(&self) -> bool {
        self.inner.is_some()
    }

    pub fn will_be_in<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), Some(x) if set.contains_state(x))
    }

    pub fn set_flush(&mut self, flush: bool) -> &mut Self {
        self.flush = flush;
        self
    }

    pub fn disable(&mut self) {
        self.inner = None;
    }

    pub fn enable_as(&mut self, value: S) -> &mut S {
        self.inner.get_or_insert(value)
    }

    // Toggle between the given state and disabled.
    pub fn toggle_as(&mut self, value: S) {
        if self.will_be_enabled() {
            self.disable();
        } else {
            self.enter(value);
        }
    }

    pub fn enter(&mut self, value: S) -> &mut S {
        self.inner.insert(value)
    }
}

#[derive(SystemParam)]
pub struct StateRef<'w, S: RawState> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: Res<'w, NextState_<S>>,
}

impl<'w, S: RawState + Eq> StateRef<'w, S> {
    pub fn will_refresh<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && set.contains_state(y),
        )
    }
}

impl<'w, S: RawState> StateRef<'w, S> {
    pub fn get(&self) -> (Option<&S>, Option<&S>) {
        (self.current.get(), self.next.get())
    }

    pub fn unwrap(&self) -> (&S, &S) {
        (
            self.current.inner.as_ref().unwrap(),
            self.next.inner.as_ref().unwrap(),
        )
    }

    pub fn will_exit<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (Some(x), _) if set.contains_state(x))
    }

    pub fn will_disable<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (Some(x), None) if set.contains_state(x))
    }

    pub fn will_enter<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (_, Some(y)) if set.contains_state(y))
    }

    pub fn will_enable<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (None, Some(y)) if set.contains_state(y))
    }
}

#[derive(SystemParam)]
pub struct StateMut<'w, S: RawState> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: ResMut<'w, NextState_<S>>,
}

impl<'w, S: RawState + Default> StateMut<'w, S> {
    // Set the next state to the default state from disabled.
    pub fn enable(&mut self) -> &mut S {
        self.next.enable()
    }

    // Toggle between the default state and disabled.
    pub fn toggle(&mut self) {
        self.next.toggle();
    }

    // Set the next state to the default state and enable flush.
    pub fn restart(&mut self) -> &mut S {
        self.next.restart()
    }
}

impl<'w, S: RawState + Clone> StateMut<'w, S> {
    // Set the next state to the current state and disable flush.
    pub fn reset(&mut self) {
        self.next
            .set_flush(false)
            .inner
            .clone_from(&self.current.inner);
    }

    // Set the next state to the current state and enable flush.
    pub fn refresh(&mut self) {
        self.next
            .set_flush(true)
            .inner
            .clone_from(&self.current.inner);
    }
}

impl<'w, S: RawState + Eq> StateMut<'w, S> {
    pub fn will_refresh<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && set.contains_state(y),
        )
    }
}

impl<'w, S: RawState> StateMut<'w, S> {
    pub fn get(&self) -> (Option<&S>, Option<&S>) {
        (self.current.get(), self.next.get())
    }

    pub fn get_mut(&mut self) -> (Option<&S>, Option<&mut S>) {
        (self.current.get(), self.next.get_mut())
    }

    pub fn unwrap(&self) -> (&S, &S) {
        (self.current.unwrap(), self.next.unwrap())
    }

    pub fn unwrap_mut(&mut self) -> (&S, &mut S) {
        (
            self.current.inner.as_ref().unwrap(),
            self.next.inner.as_mut().unwrap(),
        )
    }

    pub fn will_exit<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (Some(x), _) if set.contains_state(x))
    }

    pub fn will_disable<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (Some(x), None) if set.contains_state(x))
    }

    pub fn will_enter<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (_, Some(y)) if set.contains_state(y))
    }

    pub fn will_enable<C: ContainsState<S>>(&self, set: &C) -> bool {
        matches!(self.get(), (None, Some(y)) if set.contains_state(y))
    }

    pub fn disable(&mut self) {
        self.next.disable();
    }

    pub fn enable_as(&mut self, value: S) -> &mut S {
        self.next.enable_as(value)
    }

    // Toggles between the given state and disabled.
    pub fn toggle_as(&mut self, value: S) {
        self.next.toggle_as(value);
    }

    pub fn enter(&mut self, value: S) -> &mut S {
        self.next.enter(value)
    }

    pub fn set_flush(&mut self, flush: bool) -> &mut Self {
        self.next.flush = flush;
        self
    }
}
