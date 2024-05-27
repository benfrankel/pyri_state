use bevy_ecs::system::{Res, ResMut, Resource, SystemParam};

use crate::{
    buffer::CurrentState,
    state::{ContainsState, RawState},
};

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

// TODO: Do we still want this? Can we make this generic instead of relying on NextState?
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

// SYSTEMS AND RUN CONDITIONS
// TODO: I need to impl some kind of additional trait for the RawState itself,
// that is tied to NextState_ specifically.

pub trait RawStateExtClone: RawState + Clone {
    // TODO: Tied to NextState
    fn enable_as(value: Self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.enable_as(value.clone());
        }
    }

    // TODO: Tied to NextState
    fn toggle_as(value: Self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| state.toggle_as(value.clone())
    }

    // TODO: Tied to NextState
    fn enter(self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.enter(self.clone());
        }
    }

    // TODO: Tied to NextState
    fn reset(mut state: StateMut<Self>) {
        state.reset();
    }

    // TODO: Tied to NextState
    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }
}

impl<S: RawState + Clone> RawStateExtClone for S {}

pub trait RawStateExtDefault: RawState + Default {
    // TODO: Tied to NextState
    fn enable(mut state: ResMut<NextState_<Self>>) {
        state.enable();
    }

    // TODO: Tied to NextState
    fn toggle(mut state: ResMut<NextState_<Self>>) {
        state.toggle();
    }

    // TODO: Tied to NextState
    fn restart(mut state: ResMut<NextState_<Self>>) {
        state.restart();
    }
}

impl<T: RawState + Default> RawStateExtDefault for T {}
