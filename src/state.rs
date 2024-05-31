use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    schedule::{IntoSystemConfigs, States, SystemConfigs},
    system::{Res, ResMut, Resource, StaticSystemParam, SystemParam},
};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::{AnyStatePattern, FnStatePattern, StatePattern},
    schedule::StateFlushSet,
    storage::{GetStateStorage, SetStateStorage, StateStorage},
};

pub trait RawState: 'static + Send + Sync + Sized {
    type Storage: StateStorage;

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

    fn on_flush<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Flush)
    }

    fn on_transition<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Transition)
    }

    fn set_flush(value: bool) -> impl 'static + Send + Sync + Fn(ResMut<FlushState<Self>>) {
        move |mut flush| {
            flush.0 = value;
        }
    }
}

pub trait State_: RawState + Clone + PartialEq + Eq {}

impl<T: RawState + Clone + PartialEq + Eq> State_ for T {}

pub trait GetState: RawState {
    type GetStorage: GetStateStorage<Self>;

    fn will_be_disabled(next: NextStateRef<Self>) -> bool {
        next.get().is_none()
    }

    fn will_be_enabled(next: NextStateRef<Self>) -> bool {
        next.get().is_some()
    }
}

pub trait SetState: RawState {
    type SetStorage: SetStateStorage<Self>;

    fn disable(mut state: NextStateMut<Self>) {
        state.set(None);
    }
}

pub trait SetStateExtClone: SetState + Clone {
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

    fn reset(mut state: StateMut<Self>) {
        state.reset();
    }

    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }
}

impl<S: SetState + Clone> SetStateExtClone for S {}

pub trait SetStateExtDefault: SetState + Default {
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

impl<S: SetState + Default> SetStateExtDefault for S {}

// The immutable half of the double-buffered state.
// This should not be accessed mutably during normal usage.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: RawState>(pub Option<S>);

impl<S: RawState> Default for CurrentState<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: RawState> CurrentState<S> {
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
pub struct FlushState<S: RawState>(pub bool, PhantomData<S>);

impl<S: RawState> Default for FlushState<S> {
    fn default() -> Self {
        Self(false, PhantomData)
    }
}

#[derive(SystemParam)]
pub struct NextStateRef<'w, 's, S: GetState>(
    StaticSystemParam<'w, 's, <<S as GetState>::GetStorage as GetStateStorage<S>>::Param>,
);

impl<'w, 's, S: GetState> NextStateRef<'w, 's, S> {
    pub fn get(&self) -> Option<&S> {
        S::GetStorage::get_state(&self.0)
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
pub struct NextStateMut<'w, 's, S: SetState> {
    next: StaticSystemParam<'w, 's, <<S as SetState>::SetStorage as SetStateStorage<S>>::Param>,
    flush: ResMut<'w, FlushState<S>>,
}

impl<'w, 's, S: SetState + Default> NextStateMut<'w, 's, S> {
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

impl<'w, 's, S: SetState> NextStateMut<'w, 's, S> {
    pub fn get(&self) -> Option<&S> {
        S::SetStorage::get_state_from_mut(&self.next)
    }

    pub fn get_mut(&mut self) -> Option<&mut S> {
        S::SetStorage::get_state_mut(&mut self.next)
    }

    pub fn set(&mut self, state: Option<S>) {
        S::SetStorage::set_state(&mut self.next, state)
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

    pub fn set_flush(&mut self, value: bool) -> &mut Self {
        self.flush.0 = value;
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
pub struct StateRef<'w, 's, S: GetState> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: NextStateRef<'w, 's, S>,
}

impl<'w, 's, S: GetState + Eq> StateRef<'w, 's, S> {
    pub fn will_refresh<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: GetState> StateRef<'w, 's, S> {
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
}

// Helper macro for building a pattern matching flush run condition.
#[macro_export]
macro_rules! will_flush {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        {
            |state: pyri_state::state::StateRef<_>| {
                matches!(state.get(), $pattern $(if $guard)?)
            }
        }
    };
}

#[derive(SystemParam)]
pub struct StateMut<'w, 's, S: SetState> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: NextStateMut<'w, 's, S>,
}

impl<'w, 's, S: SetState + Clone> StateMut<'w, 's, S> {
    // Set the next state to the current state and disable flush.
    pub fn reset(&mut self) {
        self.next.set_flush(false).set(self.current.0.clone());
    }

    // Set the next state to the current state and enable flush.
    pub fn refresh(&mut self) {
        self.next.set_flush(true).set(self.current.0.clone());
    }
}

impl<'w, 's, S: SetState + Eq> StateMut<'w, 's, S> {
    pub fn will_refresh<P: StatePattern<S>>(&mut self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: SetState> StateMut<'w, 's, S> {
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

    pub fn set_flush(&mut self, value: bool) -> &mut Self {
        self.next.set_flush(value);
        self
    }
}

// A wrapper for compatibility with bevy states.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State_ + Hash + Debug>(pub Option<S>);

impl<S: State_ + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}
