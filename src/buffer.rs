use bevy_ecs::system::{Res, ResMut, Resource, SystemParam};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::state::State;

// The immutable half of the double-buffered state.
// This should never be accessed mutably during normal usage.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: State> {
    pub inner: Option<S>,
}

impl<S: State> Default for CurrentState<S> {
    fn default() -> Self {
        Self::absent()
    }
}

impl<S: State + Eq> CurrentState<S> {
    pub fn is_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
    }

    // Alias for `is_in`.
    pub fn will_exit(&self, value: &S) -> bool {
        self.is_in(value)
    }
}

impl<S: State> CurrentState<S> {
    pub fn new(inner: Option<S>) -> Self {
        Self { inner }
    }

    pub fn absent() -> Self {
        Self::new(None)
    }

    pub fn present(value: S) -> Self {
        Self::new(Some(value))
    }

    pub fn get(&self) -> Option<&S> {
        self.inner.as_ref()
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn is_absent(&self) -> bool {
        self.inner.is_none()
    }

    // Equivalent to `will_any_exit`.
    pub fn is_present(&self) -> bool {
        self.inner.is_some()
    }

    // Equivalent to `is_present`.
    pub fn will_any_exit(&self) -> bool {
        self.is_present()
    }

    pub fn will_exit_and(&self, test: impl Fn(&S) -> bool) -> bool {
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
pub struct NextState<S: State> {
    pub inner: Option<S>,
    pub flush: bool,
}

impl<S: State> Default for NextState<S> {
    fn default() -> Self {
        Self::absent()
    }
}

impl<S: State + Default> NextState<S> {
    pub fn init(&mut self) -> &mut S {
        self.inner.get_or_insert_with(|| S::default())
    }

    pub fn restart(&mut self) -> &mut S {
        self.insert(S::default())
    }
}

impl<S: State + Eq> NextState<S> {
    // Equivalent to `will_enter`.
    pub fn will_be_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
    }

    // Equivalent to `will_be_in`.
    pub fn will_enter(&self, value: &S) -> bool {
        self.will_be_in(value)
    }
}

impl<S: State> NextState<S> {
    pub fn new(inner: Option<S>) -> Self {
        Self {
            inner,
            flush: false,
        }
    }

    pub fn absent() -> Self {
        Self::new(None)
    }

    pub fn present(value: S) -> Self {
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

    pub fn will_be_absent(&self) -> bool {
        self.inner.is_none()
    }

    // Equivalent to `will_any_enter`.
    pub fn will_be_present(&self) -> bool {
        self.inner.is_some()
    }

    // Equivalent to `will_be_present`.
    pub fn will_any_enter(&self) -> bool {
        self.will_be_present()
    }

    pub fn will_enter_and(&self, test: impl Fn(&S) -> bool) -> bool {
        self.get().is_some_and(test)
    }

    pub fn set_flush(&mut self, flush: bool) -> &mut Self {
        self.flush = flush;
        self
    }

    pub fn remove(&mut self) {
        self.inner = None;
    }

    pub fn insert(&mut self, value: S) -> &mut S {
        self.inner.insert(value)
    }

    // Alias for `insert`.
    pub fn set(&mut self, value: S) -> &mut S {
        self.insert(value)
    }
}

#[derive(SystemParam)]
pub struct StateRef<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: Res<'w, NextState<S>>,
}

impl<'w, S: State + Eq> StateRef<'w, S> {
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
        matches!(self.get(), (Some(x), Some(y)) if x != y)
    }

    pub fn will_change_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y && test(x, y))
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

impl<'w, S: State> StateRef<'w, S> {
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

    pub fn will_exit_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), _) if test(x))
    }

    pub fn will_any_enter(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn will_enter_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (_, Some(y)) if test(y))
    }

    pub fn will_any_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn will_transition_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if test(x, y))
    }

    pub fn will_any_remove(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn will_remove_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), None) if test(x))
    }

    pub fn will_any_insert(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn will_insert_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (None, Some(y)) if test(y))
    }
}

#[derive(SystemParam)]
pub struct StateMut<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: ResMut<'w, NextState<S>>,
}

impl<'w, S: State + Default> StateMut<'w, S> {
    // Sets the next state to the default state unless there's already a next state.
    pub fn init(&mut self) -> &mut S {
        self.next.init()
    }

    // Sets the next state to the default state.
    pub fn restart(&mut self) -> &mut S {
        self.next.restart()
    }
}

impl<'w, S: State + Clone> StateMut<'w, S> {
    // TODO: Rename to `reset`? Or would that be confusing alongside `restart`, `refresh`, and `remove`?
    pub fn stay(&mut self) {
        self.next.inner.clone_from(&self.current.inner);
    }

    pub fn refresh(&mut self) {
        self.stay();
        if self.next.will_be_present() {
            self.next.flush = true;
        }
    }
}

impl<'w, S: State + Eq> StateMut<'w, S> {
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
        matches!(self.get(), (Some(x), Some(y)) if x != y)
    }

    pub fn will_change_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y && test(x, y))
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

impl<'w, S: State> StateMut<'w, S> {
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

    pub fn will_exit_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), _) if test(x))
    }

    pub fn will_any_enter(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn will_enter_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (_, Some(y)) if test(y))
    }

    pub fn will_any_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn will_transition_and(&self, test: impl Fn(&S, &S) -> bool) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if test(x, y))
    }

    pub fn will_any_remove(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn will_remove_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (Some(x), None) if test(x))
    }

    pub fn will_any_insert(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn will_insert_and(&self, test: impl Fn(&S) -> bool) -> bool {
        matches!(self.get(), (None, Some(y)) if test(y))
    }

    pub fn remove(&mut self) {
        self.next.remove();
    }

    pub fn insert(&mut self, value: S) -> &mut S {
        self.next.insert(value)
    }

    // Alias for `insert`.
    pub fn set(&mut self, value: S) -> &mut S {
        self.insert(value)
    }

    pub fn set_flush(&mut self, flush: bool) -> &mut Self {
        self.next.flush = flush;
        self
    }
}
