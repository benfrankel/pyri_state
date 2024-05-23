use bevy_ecs::system::{Res, ResMut, Resource, SystemParam};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::state::RawState;

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

impl<S: RawState + Eq> CurrentState<S> {
    pub fn is_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
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

    pub fn is_enabled_and(&self, test: impl Fn(&S) -> bool) -> bool {
        self.get().is_some_and(test)
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
    // Sets the next state to the default state unless there's already a next state.
    pub fn enable(&mut self) -> &mut S {
        self.inner.get_or_insert_with(|| S::default())
    }

    pub fn toggle(&mut self) {
        if self.will_be_enabled() {
            self.disable();
        } else {
            self.restart();
        }
    }

    // Sets the next state to the default state and enables flush.
    pub fn restart(&mut self) -> &mut S {
        self.set_flush(true).enter(S::default())
    }
}

impl<S: RawState + Eq> NextState_<S> {
    pub fn will_be_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
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

    pub fn will_be_disabled(&self) -> bool {
        self.inner.is_none()
    }

    pub fn will_be_enabled(&self) -> bool {
        self.inner.is_some()
    }

    pub fn will_be_enabled_and(&self, test: impl Fn(&S) -> bool) -> bool {
        self.get().is_some_and(test)
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
    pub fn will_exit(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), _) if value == x)
    }

    pub fn will_enter(&self, value: &S) -> bool {
        matches!(self.get(), (_, Some(y)) if y == value)
    }

    pub fn will_transition(&self, before: &S, after: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if before == x && y == after)
    }

    pub fn will_any_change(&self) -> bool {
        matches!(self.get(), (x, y) if x != y)
    }

    pub fn will_change_and(&self, test: impl Fn(Option<&S>, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (x, y) if x != y && test(x, y))
    }

    pub fn will_any_refresh(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y)
    }

    pub fn will_refresh(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn will_refresh_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y && test(y))
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

    pub fn will_flush_and(&self, test: impl Fn(Option<&S>, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (x, y) if test(x, y))
    }

    pub fn will_any_exit(&self) -> bool {
        matches!(self.get(), (Some(_), _))
    }

    pub fn will_exit_and(&self, test: impl Fn(&S, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (Some(x), y) if test(x, y))
    }

    pub fn will_any_enter(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn will_enter_and(&self, test: impl Fn(Option<&S>, &S) -> bool) -> bool {
        matches!(self.get(), (x, Some(y)) if test(x, y))
    }

    pub fn will_any_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn will_transition_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if test(x, y))
    }

    pub fn will_any_disable(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn will_disable_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), None) if test(x))
    }

    pub fn will_any_enable(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn will_enable_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (None, Some(y)) if test(y))
    }
}

#[derive(SystemParam)]
pub struct StateMut<'w, S: RawState> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: ResMut<'w, NextState_<S>>,
}

impl<'w, S: RawState + Default> StateMut<'w, S> {
    // Sets the next state to the default state unless there's already a next state.
    pub fn enable(&mut self) -> &mut S {
        self.next.enable()
    }

    pub fn toggle(&mut self) {
        self.next.toggle();
    }

    // Sets the next state to the default state and enables flush.
    pub fn restart(&mut self) -> &mut S {
        self.next.restart()
    }
}

impl<'w, S: RawState + Clone> StateMut<'w, S> {
    // Sets the next state to the current state and disables flush.
    pub fn reset(&mut self) {
        self.next
            .set_flush(false)
            .inner
            .clone_from(&self.current.inner);
    }

    // Sets the next state to the current state and enables flush.
    pub fn refresh(&mut self) {
        self.next
            .set_flush(true)
            .inner
            .clone_from(&self.current.inner);
    }
}

impl<'w, S: RawState + Eq> StateMut<'w, S> {
    pub fn will_exit(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), _) if value == x)
    }

    pub fn will_enter(&self, value: &S) -> bool {
        matches!(self.get(), (_, Some(y)) if y == value)
    }

    pub fn will_transition(&self, before: &S, after: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if before == x && y == after)
    }

    pub fn will_any_change(&self) -> bool {
        matches!(self.get(), (x, y) if x != y)
    }

    pub fn will_change_and(&self, test: impl Fn(Option<&S>, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (x, y) if x != y && test(x, y))
    }

    pub fn will_any_refresh(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y)
    }

    pub fn will_refresh(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn will_refresh_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y && test(y))
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

    pub fn will_flush_and(&self, test: impl Fn(Option<&S>, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (x, y) if test(x, y))
    }

    pub fn will_any_exit(&self) -> bool {
        matches!(self.get(), (Some(_), _))
    }

    pub fn will_exit_and(&self, test: impl Fn(&S, Option<&S>) -> bool) -> bool {
        matches!(self.get(), (Some(x), y) if test(x, y))
    }

    pub fn will_any_enter(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn will_enter_and(&self, test: impl Fn(Option<&S>, &S) -> bool) -> bool {
        matches!(self.get(), (x, Some(y)) if test(x, y))
    }

    pub fn will_any_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn will_transition_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if test(x, y))
    }

    pub fn will_any_disable(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn will_disable_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), None) if test(x))
    }

    pub fn will_any_enable(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn will_enable_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (None, Some(y)) if test(y))
    }

    pub fn disable(&mut self) {
        self.next.disable();
    }

    pub fn enable_as(&mut self, value: S) -> &mut S {
        self.next.enable_as(value)
    }

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
