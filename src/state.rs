use bevy_ecs::system::{Res, ResMut, Resource, SystemParam};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

pub trait State: 'static + Send + Sync + Clone + PartialEq + Eq {}

// The immutable half of the double-buffered state.
// This should not be accessed mutably unless you know what you're doing.
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

impl<S: State> CurrentState<S> {
    pub fn new(value: S) -> Self {
        Self { inner: Some(value) }
    }

    pub fn absent() -> Self {
        Self { inner: None }
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

    pub fn is_present(&self) -> bool {
        self.inner.is_some()
    }

    pub fn is_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
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
    pub will_flush: bool,
}

impl<S: State> Default for NextState<S> {
    fn default() -> Self {
        Self::absent()
    }
}

impl<S: State + Default> NextState<S> {
    // Sets the next state to the default state unless there's already a next state.
    pub fn init(&mut self) -> &mut S {
        self.inner.get_or_insert_with(|| S::default())
    }

    // Sets the next state to the default state.
    pub fn restart(&mut self) -> &mut S {
        self.insert(S::default())
    }
}

impl<S: State> NextState<S> {
    pub fn new(value: S) -> Self {
        Self {
            inner: Some(value),
            will_flush: false,
        }
    }

    pub fn absent() -> Self {
        Self {
            inner: None,
            will_flush: false,
        }
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

    pub fn flush(&mut self) -> &mut Self {
        self.will_flush = true;
        self
    }

    pub fn clear_flush(&mut self) -> &mut Self {
        self.will_flush = false;
        self
    }

    // Sets the next state to the given value and returns a mutable reference to it.
    pub fn set(&mut self, value: S) -> &mut S {
        self.inner.insert(value)
    }

    // Alias for `set`.
    pub fn insert(&mut self, value: S) -> &mut S {
        self.set(value)
    }

    pub fn remove(&mut self) {
        self.inner = None;
    }

    pub fn would_be_absent(&self) -> bool {
        self.inner.is_none()
    }

    pub fn would_be_present(&self) -> bool {
        self.inner.is_some()
    }

    pub fn would_be_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
    }

    pub fn will_flush(&self) -> bool {
        self.will_flush
    }

    pub fn will_be_absent(&self) -> bool {
        self.will_flush() && self.would_be_absent()
    }

    pub fn will_be_present(&self) -> bool {
        self.will_flush() && self.would_be_present()
    }

    pub fn will_be_in(&self, value: &S) -> bool {
        self.will_flush() && self.would_be_in(value)
    }
}

#[derive(SystemParam)]
pub struct StateRef<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: Res<'w, NextState<S>>,
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

    pub fn would_have_been_absent(&self) -> bool {
        matches!(self.get(), (None, _))
    }

    pub fn would_have_been_present(&self) -> bool {
        matches!(self.get(), (Some(_), _))
    }

    pub fn would_have_been_in(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), _) if x == value)
    }

    pub fn would_be_absent(&self) -> bool {
        matches!(self.get(), (_, None))
    }

    pub fn would_be_present(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn would_be_in(&self, value: &S) -> bool {
        matches!(self.get(), (_, Some(y)) if y == value)
    }

    // Alias for `would_have_been_absent`.
    pub fn is_absent(&self) -> bool {
        self.would_have_been_absent()
    }

    // Alias for `would_have_been_present`.
    pub fn is_present(&self) -> bool {
        self.would_have_been_present()
    }

    // Alias for `would_have_been_in`.
    pub fn is_in(&self, value: &S) -> bool {
        self.would_have_been_in(value)
    }

    // Alias for `would_have_been_present`.
    pub fn would_be_exiting(&self) -> bool {
        self.would_have_been_present()
    }

    // Alias for `would_have_been_in`.
    pub fn would_exit(&self, from: &S) -> bool {
        self.would_have_been_in(from)
    }

    // Alias for `would_be_present`.
    pub fn would_be_entering(&self) -> bool {
        self.would_be_present()
    }

    // Alias for `would_be_in`.
    pub fn would_enter(&self, to: &S) -> bool {
        self.would_be_in(to)
    }

    pub fn would_be_inserted(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn would_insert(&self, value: &S) -> bool {
        matches!(self.get(), (None, Some(y)) if y == value)
    }

    pub fn would_be_removed(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn would_remove(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), None) if x == value)
    }

    pub fn would_mutate(&self) -> bool {
        matches!(self.get(), (x, y) if x != y)
    }

    pub fn would_mutate_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), y) if from == x && Some(x) != y)
    }

    pub fn would_mutate_to(&self, to: &S) -> bool {
        matches!(self.get(), (x, Some(y)) if x != Some(y) && y == to)
    }

    // Equivalent to `would_change_from_to`.
    pub fn would_mutate_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y && y == to)
    }

    // Equivalent to `would_stay_present`.
    pub fn would_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn would_transition_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), Some(_)) if from == x)
    }

    pub fn would_transition_to(&self, to: &S) -> bool {
        matches!(self.get(), (Some(_), Some(y)) if y == to)
    }

    pub fn would_transition_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && y == to)
    }

    pub fn would_change(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y)
    }

    pub fn would_change_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y)
    }

    pub fn would_change_to(&self, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y && y == to)
    }

    // Equivalent to `would_mutate_from_to`.
    pub fn would_change_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y && y == to)
    }

    pub fn would_stay(&self) -> bool {
        matches!(self.get(), (x, y) if x == y)
    }

    pub fn would_stay_absent(&self) -> bool {
        matches!(self.get(), (None, None))
    }

    // Does not imply the state would not change; only that it would be present both before and after.
    // Equivalent to `would_transition`.
    pub fn would_stay_present(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    // Equivalent to `would_refresh_as`.
    pub fn would_stay_as(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn would_refresh(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y)
    }

    // Equivalent to `would_stay_as`.
    pub fn would_refresh_as(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn will_flush(&self) -> bool {
        self.next.will_flush
    }

    pub fn will_have_been_absent(&self) -> bool {
        self.will_flush() && self.would_have_been_absent()
    }

    pub fn will_have_been_present(&self) -> bool {
        self.will_flush() && self.would_have_been_present()
    }

    pub fn will_have_been_in(&self, value: &S) -> bool {
        self.will_flush() && self.would_have_been_in(value)
    }

    pub fn will_be_absent(&self) -> bool {
        self.will_flush() && self.would_be_absent()
    }

    pub fn will_be_present(&self) -> bool {
        self.will_flush() && self.would_be_present()
    }

    pub fn will_be_in(&self, value: &S) -> bool {
        self.will_flush() && self.would_be_in(value)
    }

    // Alias for `will_have_been_present`.
    pub fn will_be_exiting(&self) -> bool {
        self.will_have_been_present()
    }

    // Alias for `will_have_been_in`.
    pub fn will_exit(&self, from: &S) -> bool {
        self.will_have_been_in(from)
    }

    // Alias for `will_be_present`.
    pub fn will_be_entering(&self) -> bool {
        self.will_be_present()
    }

    // Alias for `will_be_in`.
    pub fn will_enter(&self, to: &S) -> bool {
        self.will_be_in(to)
    }

    pub fn will_be_inserted(&self) -> bool {
        self.will_flush() && self.would_be_inserted()
    }

    pub fn will_insert(&self, value: &S) -> bool {
        self.will_flush() && self.would_insert(value)
    }

    pub fn will_be_removed(&self) -> bool {
        self.will_flush() && self.would_be_removed()
    }

    pub fn will_remove(&self, value: &S) -> bool {
        self.will_flush() && self.would_remove(value)
    }

    pub fn will_mutate(&self) -> bool {
        self.will_flush() && self.would_mutate()
    }

    pub fn will_mutate_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_mutate_from(from)
    }

    pub fn will_mutate_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_mutate_to(to)
    }

    // Equivalent to `will_change_from_to`.
    pub fn will_mutate_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_mutate_from_to(from, to)
    }

    // Equivalent to `will_stay_present`.
    pub fn will_transition(&self) -> bool {
        self.will_flush() && self.would_transition()
    }

    pub fn will_transition_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_transition_from(from)
    }

    pub fn will_transition_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_transition_to(to)
    }

    pub fn will_transition_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_transition_from_to(from, to)
    }

    pub fn will_change(&self) -> bool {
        self.will_flush() && self.would_change()
    }

    pub fn will_change_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_change_from(from)
    }

    pub fn will_change_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_change_to(to)
    }

    // Equivalent to `would_mutate_from_to`.
    pub fn will_change_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_change_from_to(from, to)
    }

    pub fn will_stay(&self) -> bool {
        self.will_flush() && self.would_stay()
    }

    pub fn will_stay_absent(&self) -> bool {
        self.will_flush() && self.would_stay_absent()
    }

    // Does not imply the state will not change; only that it will be present both before and after.
    // Equivalent to `will_transition`.
    pub fn will_stay_present(&self) -> bool {
        self.will_flush() && self.would_stay_present()
    }

    // Equivalent to `will_refresh_as`.
    pub fn will_stay_as(&self, value: &S) -> bool {
        self.will_flush() && self.would_stay_as(value)
    }

    pub fn will_refresh(&self) -> bool {
        self.will_flush() && self.would_refresh()
    }

    // Equivalent to `will_stay_as`.
    pub fn will_refresh_as(&self, value: &S) -> bool {
        self.will_flush() && self.would_refresh_as(value)
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

    pub fn flush(&mut self) -> &mut Self {
        self.next.will_flush = true;
        self
    }

    pub fn clear_flush(&mut self) -> &mut Self {
        self.next.will_flush = false;
        self
    }

    // Sets the next state to the given value and returns a mutable reference to it.
    pub fn set(&mut self, value: S) -> &mut S {
        self.next.set(value)
    }

    // Alias for `set`.
    pub fn insert(&mut self, value: S) -> &mut S {
        self.set(value)
    }

    pub fn remove(&mut self) {
        self.next.remove();
    }

    pub fn refresh(&mut self) {
        self.next.inner.clone_from(&self.current.inner);
    }

    pub fn would_have_been_absent(&self) -> bool {
        matches!(self.get(), (None, _))
    }

    pub fn would_have_been_present(&self) -> bool {
        matches!(self.get(), (Some(_), _))
    }

    pub fn would_have_been_in(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), _) if x == value)
    }

    pub fn would_be_absent(&self) -> bool {
        matches!(self.get(), (_, None))
    }

    pub fn would_be_present(&self) -> bool {
        matches!(self.get(), (_, Some(_)))
    }

    pub fn would_be_in(&self, value: &S) -> bool {
        matches!(self.get(), (_, Some(y)) if y == value)
    }

    // Alias for `would_have_been_absent`.
    pub fn is_absent(&self) -> bool {
        self.would_have_been_absent()
    }

    // Alias for `would_have_been_present`.
    pub fn is_present(&self) -> bool {
        self.would_have_been_present()
    }

    // Alias for `would_have_been_in`.
    pub fn is_in(&self, value: &S) -> bool {
        self.would_have_been_in(value)
    }

    // Alias for `would_have_been_present`.
    pub fn would_be_exiting(&self) -> bool {
        self.would_have_been_present()
    }

    // Alias for `would_have_been_in`.
    pub fn would_exit(&self, from: &S) -> bool {
        self.would_have_been_in(from)
    }

    // Alias for `would_be_present`.
    pub fn would_be_entering(&self) -> bool {
        self.would_be_present()
    }

    // Alias for `would_be_in`.
    pub fn would_enter(&self, to: &S) -> bool {
        self.would_be_in(to)
    }

    pub fn would_be_inserted(&self) -> bool {
        matches!(self.get(), (None, Some(_)))
    }

    pub fn would_insert(&self, value: &S) -> bool {
        matches!(self.get(), (None, Some(y)) if y == value)
    }

    pub fn would_be_removed(&self) -> bool {
        matches!(self.get(), (Some(_), None))
    }

    pub fn would_remove(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), None) if x == value)
    }

    pub fn would_mutate(&self) -> bool {
        matches!(self.get(), (x, y) if x != y)
    }

    pub fn would_mutate_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), y) if from == x && Some(x) != y)
    }

    pub fn would_mutate_to(&self, to: &S) -> bool {
        matches!(self.get(), (x, Some(y)) if x != Some(y) && y == to)
    }

    // Equivalent to `would_change_from_to`.
    pub fn would_mutate_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y && y == to)
    }

    // Equivalent to `would_stay_present`.
    pub fn would_transition(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    pub fn would_transition_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), Some(_)) if from == x)
    }

    pub fn would_transition_to(&self, to: &S) -> bool {
        matches!(self.get(), (Some(_), Some(y)) if y == to)
    }

    pub fn would_transition_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && y == to)
    }

    pub fn would_change(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y)
    }

    pub fn would_change_from(&self, from: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y)
    }

    pub fn would_change_to(&self, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x != y && y == to)
    }

    // Equivalent to `would_mutate_from_to`.
    pub fn would_change_from_to(&self, from: &S, to: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if from == x && x != y && y == to)
    }

    pub fn would_stay(&self) -> bool {
        matches!(self.get(), (x, y) if x == y)
    }

    pub fn would_stay_absent(&self) -> bool {
        matches!(self.get(), (None, None))
    }

    // Does not imply the state would not change; only that it would be present both before and after.
    // Equivalent to `would_transition`.
    pub fn would_stay_present(&self) -> bool {
        matches!(self.get(), (Some(_), Some(_)))
    }

    // Equivalent to `would_refresh_as`.
    pub fn would_stay_as(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn would_refresh(&self) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if x == y)
    }

    // Equivalent to `would_stay_as`.
    pub fn would_refresh_as(&self, value: &S) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if value == x && x == y)
    }

    pub fn will_flush(&self) -> bool {
        self.next.will_flush
    }

    pub fn will_have_been_absent(&self) -> bool {
        self.will_flush() && self.would_have_been_absent()
    }

    pub fn will_have_been_present(&self) -> bool {
        self.will_flush() && self.would_have_been_present()
    }

    pub fn will_have_been_in(&self, value: &S) -> bool {
        self.will_flush() && self.would_have_been_in(value)
    }

    pub fn will_be_absent(&self) -> bool {
        self.will_flush() && self.would_be_absent()
    }

    pub fn will_be_present(&self) -> bool {
        self.will_flush() && self.would_be_present()
    }

    pub fn will_be_in(&self, value: &S) -> bool {
        self.will_flush() && self.would_be_in(value)
    }

    // Alias for `will_have_been_present`.
    pub fn will_be_exiting(&self) -> bool {
        self.will_have_been_present()
    }

    // Alias for `will_have_been_in`.
    pub fn will_exit(&self, from: &S) -> bool {
        self.will_have_been_in(from)
    }

    // Alias for `will_be_present`.
    pub fn will_be_entering(&self) -> bool {
        self.will_be_present()
    }

    // Alias for `will_be_in`.
    pub fn will_enter(&self, to: &S) -> bool {
        self.will_be_in(to)
    }

    pub fn will_be_inserted(&self) -> bool {
        self.will_flush() && self.would_be_inserted()
    }

    pub fn will_insert(&self, value: &S) -> bool {
        self.will_flush() && self.would_insert(value)
    }

    pub fn will_be_removed(&self) -> bool {
        self.will_flush() && self.would_be_removed()
    }

    pub fn will_remove(&self, value: &S) -> bool {
        self.will_flush() && self.would_remove(value)
    }

    pub fn will_mutate(&self) -> bool {
        self.will_flush() && self.would_mutate()
    }

    pub fn will_mutate_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_mutate_from(from)
    }

    pub fn will_mutate_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_mutate_to(to)
    }

    // Equivalent to `will_change_from_to`.
    pub fn will_mutate_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_mutate_from_to(from, to)
    }

    // Equivalent to `will_stay_present`.
    pub fn will_transition(&self) -> bool {
        self.will_flush() && self.would_transition()
    }

    pub fn will_transition_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_transition_from(from)
    }

    pub fn will_transition_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_transition_to(to)
    }

    pub fn will_transition_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_transition_from_to(from, to)
    }

    pub fn will_change(&self) -> bool {
        self.will_flush() && self.would_change()
    }

    pub fn will_change_from(&self, from: &S) -> bool {
        self.will_flush() && self.would_change_from(from)
    }

    pub fn will_change_to(&self, to: &S) -> bool {
        self.will_flush() && self.would_change_to(to)
    }

    // Equivalent to `would_mutate_from_to`.
    pub fn will_change_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.would_change_from_to(from, to)
    }

    pub fn will_stay(&self) -> bool {
        self.will_flush() && self.would_stay()
    }

    pub fn will_stay_absent(&self) -> bool {
        self.will_flush() && self.would_stay_absent()
    }

    // Does not imply the state will not change; only that it will be present both before and after.
    // Equivalent to `will_transition`.
    pub fn will_stay_present(&self) -> bool {
        self.will_flush() && self.would_stay_present()
    }

    // Equivalent to `will_refresh_as`.
    pub fn will_stay_as(&self, value: &S) -> bool {
        self.will_flush() && self.would_stay_as(value)
    }

    pub fn will_refresh(&self) -> bool {
        self.will_flush() && self.would_refresh()
    }

    // Equivalent to `will_stay_as`.
    pub fn will_refresh_as(&self, value: &S) -> bool {
        self.will_flush() && self.would_refresh_as(value)
    }
}
