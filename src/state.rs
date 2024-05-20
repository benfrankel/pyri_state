use bevy_ecs::system::{BoxedSystem, IntoSystem, Res, ResMut, Resource, SystemParam};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::schedule::OnState;

pub trait State: 'static + Send + Sync + Clone {
    fn on_flush() -> OnState<Self> {
        OnState::Flush
    }

    fn on_exit() -> OnState<Self> {
        OnState::Exit
    }

    fn on_enter() -> OnState<Self> {
        OnState::Enter
    }

    fn on_transition() -> OnState<Self> {
        OnState::Transition
    }

    // TODO: This doesn't check `flush`. Could be confusing.
    fn will_flush_and(
        test: impl Fn(Option<&Self>, Option<&Self>) -> bool,
    ) -> impl Fn(StateRef<Self>) -> bool {
        move |state| matches!(state.get(), (x, y) if test(x, y))
    }

    fn will_exit_and(test: impl Fn(&Self) -> bool) -> impl Fn(Res<CurrentState<Self>>) -> bool {
        move |state| matches!(state.get(), Some(x) if test(x))
    }

    fn will_enter_and(test: impl Fn(&Self) -> bool) -> impl Fn(Res<NextState<Self>>) -> bool {
        move |state| matches!(state.get(), Some(y) if test(y))
    }

    fn will_transition_and(test: impl Fn(&Self, &Self) -> bool) -> impl Fn(StateRef<Self>) -> bool {
        move |state| matches!(state.get(), (Some(x), Some(y)) if test(x, y))
    }

    // TODO: BoxedSystem is a workaround for https://github.com/bevyengine/bevy/issues/13436.
    fn flush(flush: bool) -> BoxedSystem {
        Box::new(IntoSystem::into_system(
            move |mut state: ResMut<NextState<Self>>| {
                state.flush(flush);
            },
        ))
    }

    fn remove(mut state: ResMut<NextState<Self>>) {
        state.remove();
    }

    // TODO: BoxedSystem is a workaround for https://github.com/bevyengine/bevy/issues/13436.
    fn insert(value: Self) -> BoxedSystem {
        Box::new(IntoSystem::into_system(
            move |mut state: ResMut<NextState<Self>>| {
                state.insert(value.clone());
            },
        ))
    }

    // TODO: BoxedSystem is a workaround for https://github.com/bevyengine/bevy/issues/13436.
    // Alias for `insert`.
    fn set(value: Self) -> BoxedSystem {
        Self::insert(value)
    }

    fn stay(mut state: StateMut<Self>) {
        state.stay();
    }

    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }
}

pub trait StateExtEq: State + Eq {
    fn will_exit(before: Self) -> impl Fn(Res<CurrentState<Self>>) -> bool {
        move |state| state.is_in(&before)
    }

    fn will_enter(after: Self) -> impl Fn(Res<NextState<Self>>) -> bool {
        move |state| state.will_be_in(&after)
    }

    fn will_transition(before: Self, after: Self) -> impl Fn(StateRef<Self>) -> bool {
        move |state| state.will_transition(&before, &after)
    }

    fn will_change(state: StateRef<Self>) -> bool {
        state.will_change()
    }

    fn will_stay(state: StateRef<Self>) -> bool {
        state.will_stay()
    }

    fn will_stay_as(value: Option<Self>) -> impl Fn(StateRef<Self>) -> bool {
        move |state| state.will_stay_as(value.as_ref())
    }

    fn will_stay_and(test: impl Fn(Option<&Self>) -> bool) -> impl Fn(StateRef<Self>) -> bool {
        move |state| matches!(state.get(), (x, y) if x == y && test(x))
    }

    fn will_refresh(state: StateRef<Self>) -> bool {
        state.will_refresh()
    }

    fn will_refresh_as(value: Self) -> impl Fn(StateRef<Self>) -> bool {
        move |state| state.will_refresh_as(&value)
    }

    fn will_refresh_and(test: impl Fn(&Self) -> bool) -> impl Fn(StateRef<Self>) -> bool {
        move |state| matches!(state.get(), (Some(x), Some(y)) if x == y && test(x))
    }
}

impl<T: State + Eq> StateExtEq for T {}

pub trait StateExtDefault: State + Default {
    fn init(mut state: ResMut<NextState<Self>>) {
        state.init();
    }

    fn restart(mut state: ResMut<NextState<Self>>) {
        state.restart();
    }
}

impl<T: State + Default> StateExtDefault for T {}

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
}

impl<S: State + Eq> CurrentState<S> {
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
    pub fn will_be_in(&self, value: &S) -> bool {
        self.inner.as_ref() == Some(value)
    }
}

impl<S: State> NextState<S> {
    pub fn new(value: S) -> Self {
        Self {
            inner: Some(value),
            flush: false,
        }
    }

    pub fn absent() -> Self {
        Self {
            inner: None,
            flush: false,
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

    pub fn will_be_absent(&self) -> bool {
        self.inner.is_none()
    }

    pub fn will_be_present(&self) -> bool {
        self.inner.is_some()
    }

    pub fn flush(&mut self, flush: bool) -> &mut Self {
        self.flush = flush;
        self
    }

    pub fn remove(&mut self) {
        self.inner = None;
    }

    pub fn insert(&mut self, value: S) -> &mut S {
        self.set(value)
    }

    // Alias for `insert`.
    pub fn set(&mut self, value: S) -> &mut S {
        self.inner.insert(value)
    }
}

#[derive(SystemParam)]
pub struct StateRef<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: Res<'w, NextState<S>>,
}

impl<'w, S: State + Eq> StateRef<'w, S> {
    pub fn will_exit(&self, before: &S) -> bool {
        self.current.is_in(before)
    }

    pub fn will_enter(&self, after: &S) -> bool {
        self.next.will_be_in(after)
    }

    pub fn will_transition(&self, before: &S, after: &S) -> bool {
        self.will_exit(before) && self.will_enter(after)
    }

    pub fn will_change(&self) -> bool {
        self.current.inner != self.next.inner
    }

    pub fn will_stay(&self) -> bool {
        self.current.inner == self.next.inner
    }

    pub fn will_stay_as(&self, value: Option<&S>) -> bool {
        self.current.get() == value && self.next.get() == value
    }

    pub fn will_refresh(&self) -> bool {
        self.current.is_present() && self.will_stay()
    }

    pub fn will_refresh_as(&self, value: &S) -> bool {
        self.current.is_in(value) && self.next.will_be_in(value)
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
}

#[derive(SystemParam)]
pub struct StateMut<'w, S: State> {
    pub current: Res<'w, CurrentState<S>>,
    pub next: ResMut<'w, NextState<S>>,
}

impl<'w, S: State + Eq> StateMut<'w, S> {
    pub fn will_exit(&self, before: &S) -> bool {
        self.current.is_in(before)
    }

    pub fn will_enter(&self, after: &S) -> bool {
        self.next.will_be_in(after)
    }

    pub fn will_transition(&self, before: &S, after: &S) -> bool {
        self.will_exit(before) && self.will_enter(after)
    }

    pub fn will_change(&self) -> bool {
        self.current.inner != self.next.inner
    }

    pub fn will_stay(&self) -> bool {
        self.current.inner == self.next.inner
    }

    pub fn will_stay_as(&self, value: Option<&S>) -> bool {
        self.current.get() == value && self.next.get() == value
    }

    pub fn will_refresh(&self) -> bool {
        self.current.is_present() && self.will_stay()
    }

    pub fn will_refresh_as(&self, value: &S) -> bool {
        self.current.is_in(value) && self.next.will_be_in(value)
    }
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

    pub fn flush(&mut self, flush: bool) -> &mut Self {
        self.next.flush = flush;
        self
    }

    pub fn remove(&mut self) {
        self.next.remove();
    }

    pub fn insert(&mut self, value: S) -> &mut S {
        self.set(value)
    }

    // Alias for `insert`.
    pub fn set(&mut self, value: S) -> &mut S {
        self.next.set(value)
    }

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
