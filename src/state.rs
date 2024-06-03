use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    schedule::States,
    system::{Res, ResMut, Resource, StaticSystemParam, SystemParam},
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::{AnyStatePattern, FnStatePattern, StatePattern},
    prelude::StateTransitionPattern,
    storage::{StateStorage, StateStorageMut},
};

pub trait State_: 'static + Send + Sync + Sized {
    type Storage: StateStorage<Self>;

    const ANY: AnyStatePattern<Self> = AnyStatePattern(PhantomData);

    fn with<F>(f: F) -> FnStatePattern<Self, F>
    where
        F: 'static + Send + Sync + Fn(&Self) -> bool,
    {
        FnStatePattern(f, PhantomData)
    }

    fn is_disabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_disabled()
    }

    fn is_enabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_enabled()
    }

    fn will_be_disabled(next: NextStateRef<Self>) -> bool {
        next.get().is_none()
    }

    fn will_be_enabled(next: NextStateRef<Self>) -> bool {
        next.get().is_some()
    }

    fn trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.trigger();
    }

    fn relax(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.relax();
    }
}

pub trait StateMut: State_ {
    type StorageMut: StateStorageMut<Self>;

    fn disable(mut state: NextStateMut<Self>) {
        state.set(None);
    }
}

pub trait StateMutExtClone: StateMut + Clone {
    fn enable(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            }
        }
    }

    fn toggle(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            } else {
                state.disable();
            }
        }
    }

    fn enter(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.set(Some(self.clone()));
        }
    }

    fn reset(mut state: StateFlushMut<Self>) {
        state.reset();
    }

    fn refresh(mut state: StateFlushMut<Self>) {
        state.refresh();
    }
}

impl<S: StateMut + Clone> StateMutExtClone for S {}

pub trait StateMutExtDefault: StateMut + Default {
    fn enable_default(mut state: NextStateMut<Self>) {
        state.enable_default();
    }

    fn toggle_default(mut state: NextStateMut<Self>) {
        state.toggle_default();
    }

    fn enter_default(mut state: NextStateMut<Self>) {
        state.enter_default();
    }
}

impl<S: StateMut + Default> StateMutExtDefault for S {}

// The immutable half of the double-buffered state.
// This should not be accessed mutably during normal usage.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: State_>(pub Option<S>);

impl<S: State_> Default for CurrentState<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: State_> CurrentState<S> {
    pub fn new(value: Option<S>) -> Self {
        Self(value)
    }

    pub fn disabled() -> Self {
        Self::new(None)
    }

    pub fn enabled(value: S) -> Self {
        Self::new(Some(value))
    }

    pub fn get(&self) -> Option<&S> {
        self.0.as_ref()
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }
}

// The flag that determines whether a state type will flush.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct TriggerStateFlush<S: State_>(pub bool, PhantomData<S>);

impl<S: State_> Default for TriggerStateFlush<S> {
    fn default() -> Self {
        Self(false, PhantomData)
    }
}

impl<S: State_> TriggerStateFlush<S> {
    pub fn trigger(&mut self) {
        self.0 = true;
    }

    pub fn relax(&mut self) {
        self.0 = false;
    }
}

#[derive(SystemParam)]
pub struct NextStateRef<'w, 's, S: State_>(
    StaticSystemParam<'w, 's, <<S as State_>::Storage as StateStorage<S>>::Param>,
);

impl<'w, 's, S: State_> NextStateRef<'w, 's, S> {
    pub fn get(&self) -> Option<&S> {
        S::Storage::get_state(&self.0)
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn will_be_disabled(&self) -> bool {
        self.get().is_none()
    }

    pub fn will_be_enabled(&self) -> bool {
        self.get().is_some()
    }

    pub fn will_be_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }
}

#[derive(SystemParam)]
pub struct NextStateMut<'w, 's, S: StateMut> {
    next: StaticSystemParam<'w, 's, <<S as StateMut>::StorageMut as StateStorageMut<S>>::Param>,
    trigger: ResMut<'w, TriggerStateFlush<S>>,
}

impl<'w, 's, S: StateMut + Default> NextStateMut<'w, 's, S> {
    // Enter the default state if disabled.
    pub fn enable_default(&mut self) {
        if self.will_be_disabled() {
            self.enter(S::default())
        }
    }

    // Toggle between the default state and disabled.
    pub fn toggle_default(&mut self) {
        if self.will_be_disabled() {
            self.enable_default();
        } else {
            self.disable();
        }
    }

    // Enter the default state.
    pub fn enter_default(&mut self) {
        self.enter(S::default());
    }
}

impl<'w, 's, S: StateMut> NextStateMut<'w, 's, S> {
    pub fn get(&self) -> Option<&S> {
        S::StorageMut::get_state_from_mut(&self.next)
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        S::StorageMut::get_state_mut(&mut self.next)
    }

    pub fn set(&mut self, state: Option<S>) {
        S::StorageMut::set_state(&mut self.next, state)
    }

    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    pub fn will_be_disabled(&self) -> bool {
        self.get().is_none()
    }

    pub fn will_be_enabled(&self) -> bool {
        self.get().is_some()
    }

    pub fn will_be_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }

    pub fn trigger(&mut self) -> &mut Self {
        self.trigger.trigger();
        self
    }

    pub fn relax(&mut self) -> &mut Self {
        self.trigger.relax();
        self
    }

    pub fn disable(&mut self) {
        self.set(None);
    }

    // Enter the given state if disabled.
    pub fn enable(&mut self, value: S) {
        if self.will_be_disabled() {
            self.enter(value);
        }
    }

    // Toggle between the given state and disabled.
    pub fn toggle(&mut self, value: S) {
        if self.will_be_enabled() {
            self.disable();
        } else {
            self.enter(value);
        }
    }

    pub fn enter(&mut self, value: S) {
        self.set(Some(value));
    }
}

#[derive(SystemParam)]
pub struct StateFlushRef<'w, 's, S: State_> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: NextStateRef<'w, 's, S>,
}

impl<'w, 's, S: State_ + Eq> StateFlushRef<'w, 's, S> {
    pub fn will_refresh<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: State_> StateFlushRef<'w, 's, S> {
    pub fn get(&self) -> (Option<&S>, Option<&S>) {
        (self.current.get(), self.next.get())
    }

    pub fn unwrap(&self) -> (&S, &S) {
        let (current, next) = self.get();
        (current.unwrap(), next.unwrap())
    }

    pub fn will_exit<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), _) if pattern.matches(x))
    }

    pub fn will_disable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), None) if pattern.matches(x))
    }

    pub fn will_enter<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (_, Some(y)) if pattern.matches(y))
    }

    pub fn will_enable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (None, Some(y)) if pattern.matches(y))
    }

    pub fn will_transition<P: StateTransitionPattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if pattern.matches(x, y))
    }
}

// Helper macro for building a pattern matching flush run condition.
#[macro_export]
macro_rules! will_flush {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        {
            |state: pyri_state::state::StateFlushRef<_>| {
                matches!(state.get(), $pattern $(if $guard)?)
            }
        }
    };
}

#[derive(SystemParam)]
pub struct StateFlushMut<'w, 's, S: StateMut> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: NextStateMut<'w, 's, S>,
}

impl<'w, 's, S: StateMut + Clone> StateFlushMut<'w, 's, S> {
    // Set the next state to the current state and relax flush.
    pub fn reset(&mut self) {
        self.next.relax().set(self.current.0.clone());
    }

    // Set the next state to the current state and trigger flush.
    pub fn refresh(&mut self) {
        self.next.trigger().set(self.current.0.clone());
    }
}

impl<'w, 's, S: StateMut + Eq> StateFlushMut<'w, 's, S> {
    pub fn will_refresh<P: StatePattern<S>>(&mut self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: StateMut> StateFlushMut<'w, 's, S> {
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
        (self.current.unwrap(), self.next.unwrap_mut())
    }

    pub fn will_exit<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), _) if pattern.matches(x))
    }

    pub fn will_disable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), None) if pattern.matches(x))
    }

    pub fn will_enter<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (_, Some(y)) if pattern.matches(y))
    }

    pub fn will_enable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (None, Some(y)) if pattern.matches(y))
    }

    pub fn will_transition<P: StateTransitionPattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if pattern.matches(x, y))
    }

    pub fn disable(&mut self) {
        self.next.disable();
    }

    // Enter the given state if disabled.
    pub fn enable(&mut self, value: S) {
        self.next.enable(value);
    }

    // Toggle between the given state and disabled.
    pub fn toggle(&mut self, value: S) {
        self.next.toggle(value);
    }

    // Enter the default state.
    pub fn enter(&mut self, value: S) {
        self.next.set(Some(value));
    }

    pub fn trigger(&mut self) -> &mut Self {
        self.next.trigger();
        self
    }

    pub fn relax(&mut self) -> &mut Self {
        self.next.relax();
        self
    }
}

// A wrapper for compatibility with bevy states.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State_ + Clone + PartialEq + Eq + Hash + Debug>(pub Option<S>);

impl<S: State_ + Clone + PartialEq + Eq + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}
