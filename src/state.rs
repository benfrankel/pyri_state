use bevy_ecs::system::{Res, ResMut, Resource, SystemParam};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

pub trait State: 'static + Send + Sync + Clone + PartialEq + Eq {}

// The immutable half of the double buffered state.
// This should not be accessed mutably unless you know what you're doing.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: State> {
    pub value: Option<S>,
}

impl<S: State> Default for CurrentState<S> {
    fn default() -> Self {
        Self::absent()
    }
}

impl<S: State> CurrentState<S> {
    pub fn new(value: S) -> Self {
        Self { value: Some(value) }
    }

    pub fn absent() -> Self {
        Self { value: None }
    }
}

// The mutable half of the double buffered state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct NextState<S: State> {
    pub value: Option<S>,
    pub flush: bool,
}

impl<S: State> Default for NextState<S> {
    fn default() -> Self {
        Self::absent()
    }
}

impl<S: State> NextState<S> {
    pub fn new(value: S) -> Self {
        Self {
            value: Some(value),
            flush: false,
        }
    }

    pub fn absent() -> Self {
        Self {
            value: None,
            flush: false,
        }
    }
}

#[derive(SystemParam)]
pub struct StateRef<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: Res<'w, NextState<S>>,
}

impl<'w, S: State> StateRef<'w, S> {
    pub fn get(&self) -> Option<&S> {
        self.current.value.as_ref()
    }

    pub fn get_next(&self) -> Option<&S> {
        self.next.value.as_ref()
    }

    pub fn unwrap(&self) -> (&S, &S) {
        (
            self.current.value.as_ref().unwrap(),
            self.next.value.as_ref().unwrap(),
        )
    }

    pub fn is_absent(&self) -> bool {
        self.current.value.is_none()
    }

    pub fn will_be_absent(&self) -> bool {
        self.next.value.is_none()
    }

    pub fn is_present(&self) -> bool {
        !self.is_absent()
    }

    pub fn will_be_present(&self) -> bool {
        !self.will_be_absent()
    }

    pub fn will_remain_absent(&self) -> bool {
        self.is_absent() && self.will_be_absent()
    }

    pub fn will_remain_present(&self) -> bool {
        self.is_present() && self.will_be_present()
    }

    pub fn will_be_inserted(&self) -> bool {
        self.is_absent() && self.will_be_present()
    }

    pub fn will_insert(&self, value: &S) -> bool {
        self.is_absent() && self.will_be_in(value)
    }

    pub fn will_be_removed(&self) -> bool {
        self.will_be_absent() && self.is_present()
    }

    pub fn will_remove(&self, value: &S) -> bool {
        self.will_be_absent() && self.is_in(value)
    }

    pub fn is_in(&self, value: &S) -> bool {
        self.get() == Some(value)
    }

    pub fn will_be_in(&self, value: &S) -> bool {
        self.get_next() == Some(value)
    }

    pub fn will_stay(&self) -> bool {
        self.is_present() && self.get() == self.get_next()
    }

    pub fn will_stay_in(&self, value: &S) -> bool {
        self.is_in(value) && self.will_be_in(value)
    }

    pub fn will_change(&self) -> bool {
        self.will_remain_present() && self.get() != self.get_next()
    }

    pub fn will_change_from(&self, from: &S) -> bool {
        self.will_be_present() && self.is_in(from) && !self.will_be_in(from)
    }

    pub fn will_change_to(&self, to: &S) -> bool {
        self.is_present() && !self.is_in(to) && self.will_be_in(to)
    }

    pub fn will_change_from_to(&self, from: &S, to: &S) -> bool {
        self.is_in(from) && self.will_be_in(to) && from != to
    }

    pub fn will_flush(&self) -> bool {
        self.next.flush
    }

    pub fn will_flush_from(&self, from: &S) -> bool {
        self.will_flush() && self.is_in(from)
    }

    pub fn will_flush_to(&self, to: &S) -> bool {
        self.will_flush() && self.will_be_in(to)
    }

    pub fn will_flush_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.is_in(from) && self.will_be_in(to)
    }

    pub fn will_refresh(&self) -> bool {
        self.will_flush() && self.will_stay()
    }

    pub fn will_refresh_in(&self, value: &S) -> bool {
        self.will_flush() && self.will_stay_in(value)
    }

    pub fn will_transition(&self) -> bool {
        self.will_flush() && self.will_remain_present()
    }
}

#[derive(SystemParam)]
pub struct StateMut<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: ResMut<'w, NextState<S>>,
}

impl<'w, S: State + Default> StateMut<'w, S> {
    pub fn set(&mut self) -> &mut S {
        self.next
            .value
            .get_or_insert_with(|| self.current.value.clone().unwrap_or_default())
    }

    pub fn change(&mut self) -> &mut S {
        self.next.flush = false;
        self.set()
    }

    pub fn flush(&mut self) -> &mut S {
        self.next.flush = true;
        self.set()
    }

    pub fn restart(&mut self) -> &mut S {
        self.insert(S::default())
    }

    // TODO: Not sure about the name for this and `hard_restart`.
    pub fn soft_restart(&mut self) -> &mut S {
        self.change_to(S::default())
    }

    pub fn hard_restart(&mut self) -> &mut S {
        self.flush_to(S::default())
    }
}

impl<'w, S: State> StateMut<'w, S> {
    pub fn get(&self) -> Option<&S> {
        self.current.value.as_ref()
    }

    pub fn get_next(&self) -> Option<&S> {
        self.next.value.as_ref()
    }

    pub fn get_next_mut(&mut self) -> Option<&mut S> {
        self.next.value.as_mut()
    }

    pub fn unwrap(&self) -> (&S, &S) {
        (
            self.current.value.as_ref().unwrap(),
            self.next.value.as_ref().unwrap(),
        )
    }

    pub fn unwrap_mut(&mut self) -> (&S, &mut S) {
        (
            self.current.value.as_ref().unwrap(),
            self.next.value.as_mut().unwrap(),
        )
    }

    pub fn remove(&mut self) {
        self.next.value = None;
    }

    pub fn insert(&mut self, value: S) -> &mut S {
        self.next.value.insert(value)
    }

    pub fn change_to(&mut self, value: S) -> &mut S {
        self.next.flush = false;
        self.insert(value)
    }

    pub fn flush_to(&mut self, value: S) -> &mut S {
        self.next.flush = true;
        self.insert(value)
    }

    // NOTE: This is a no-op if the current state is absent.
    pub fn refresh(&mut self) {
        if self.is_present() {
            self.next.flush = true;
            self.next.value = self.current.value.clone();
        }
    }

    pub fn is_absent(&self) -> bool {
        self.current.value.is_none()
    }

    pub fn will_be_absent(&self) -> bool {
        self.next.value.is_none()
    }

    pub fn is_present(&self) -> bool {
        !self.is_absent()
    }

    pub fn will_be_present(&self) -> bool {
        !self.will_be_absent()
    }

    pub fn will_remain_absent(&self) -> bool {
        self.is_absent() && self.will_be_absent()
    }

    pub fn will_remain_present(&self) -> bool {
        self.is_present() && self.will_be_present()
    }

    pub fn will_be_inserted(&self) -> bool {
        self.is_absent() && self.will_be_present()
    }

    pub fn will_insert(&self, value: &S) -> bool {
        self.is_absent() && self.will_be_in(value)
    }

    pub fn will_be_removed(&self) -> bool {
        self.will_be_absent() && self.is_present()
    }

    pub fn will_remove(&self, value: &S) -> bool {
        self.will_be_absent() && self.is_in(value)
    }

    pub fn is_in(&self, value: &S) -> bool {
        self.get() == Some(value)
    }

    pub fn will_be_in(&self, value: &S) -> bool {
        self.get_next() == Some(value)
    }

    pub fn will_stay(&self) -> bool {
        self.is_present() && self.get() == self.get_next()
    }

    pub fn will_stay_in(&self, value: &S) -> bool {
        self.is_in(value) && self.will_be_in(value)
    }

    pub fn will_change(&self) -> bool {
        self.will_remain_present() && self.get() != self.get_next()
    }

    pub fn will_change_from(&self, from: &S) -> bool {
        self.will_be_present() && self.is_in(from) && !self.will_be_in(from)
    }

    pub fn will_change_to(&self, to: &S) -> bool {
        self.is_present() && !self.is_in(to) && self.will_be_in(to)
    }

    pub fn will_change_from_to(&self, from: &S, to: &S) -> bool {
        self.is_in(from) && self.will_be_in(to) && from != to
    }

    pub fn will_flush(&self) -> bool {
        self.next.flush
    }

    pub fn will_flush_from(&self, from: &S) -> bool {
        self.will_flush() && self.is_in(from)
    }

    pub fn will_flush_to(&self, to: &S) -> bool {
        self.will_flush() && self.will_be_in(to)
    }

    pub fn will_flush_from_to(&self, from: &S, to: &S) -> bool {
        self.will_flush() && self.is_in(from) && self.will_be_in(to)
    }

    pub fn will_transition(&self) -> bool {
        self.will_flush() && self.will_remain_present()
    }

    pub fn will_refresh(&self) -> bool {
        self.will_flush() && self.will_stay()
    }

    pub fn will_refresh_in(&self, value: &S) -> bool {
        self.will_flush() && self.will_stay_in(value)
    }
}
