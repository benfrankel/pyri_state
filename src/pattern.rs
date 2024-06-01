use std::marker::PhantomData;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::Res,
};

use crate::{
    schedule::StateFlushSet,
    state::{CurrentState, GetState, NextStateRef, RawState, StateFlushRef},
};

pub trait StatePattern<S: RawState>: 'static + Send + Sync + Sized {
    fn matches(&self, state: &S) -> bool;

    // Equivalent to `will_exit`.
    fn will_update(self) -> impl 'static + Send + Sync + Fn(Res<CurrentState<S>>) -> bool {
        self.will_exit()
    }

    fn on_update<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(self.will_update())
    }

    fn will_exit(self) -> impl 'static + Send + Sync + Fn(Res<CurrentState<S>>) -> bool {
        move |state| state.is_in(&self)
    }

    fn on_exit<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_exit())
            .in_set(StateFlushSet::<S>::Exit)
    }
}

pub trait StatePatternExtGet<S: GetState>: StatePattern<S> {
    fn will_disable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| matches!(state.get(), (Some(x), None) if self.matches(x))
    }

    fn on_disable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_disable())
            .in_set(StateFlushSet::<S>::Exit)
    }

    fn will_enter(self) -> impl 'static + Send + Sync + Fn(NextStateRef<S>) -> bool {
        move |state| matches!(state.get(), Some(x) if self.matches(x))
    }

    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(StateFlushSet::<S>::Enter)
    }

    fn will_enable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| matches!(state.get(), (None, Some(x)) if self.matches(x))
    }

    fn on_enable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enable())
            .in_set(StateFlushSet::<S>::Enter)
    }
}

impl<S: GetState, T: StatePattern<S>> StatePatternExtGet<S> for T {}

pub trait StatePatternExtGetAndEq<S: GetState + Eq>: StatePattern<S> {
    fn will_refresh(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| {
            matches!(
                state.get(),
                (Some(x), Some(y)) if x == y && self.matches(y),
            )
        }
    }

    fn on_refresh<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_refresh())
            .in_set(StateFlushSet::<S>::Transition)
    }
}

impl<S: GetState + Eq, T: StatePattern<S>> StatePatternExtGetAndEq<S> for T {}

impl<S: RawState + Eq> StatePattern<S> for S {
    fn matches(&self, state: &S) -> bool {
        self == state
    }
}

pub struct FnStatePattern<S: RawState, F: Fn(&S) -> bool>(pub(crate) F, pub(crate) PhantomData<S>);

impl<S: RawState, F> StatePattern<S> for FnStatePattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S) -> bool,
{
    fn matches(&self, state: &S) -> bool {
        self.0(state)
    }
}

impl<S: RawState, F> FnStatePattern<S, F>
where
    F: Fn(&S) -> bool,
{
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

// Helper macro for building a pattern matching state set.
#[macro_export]
macro_rules! state {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        pyri_state::pattern::FnStatePattern::new(
            |state| matches!(*state, $pattern $(if $guard)?),
        )
    };
}

pub struct AnyStatePattern<S: RawState>(pub(crate) PhantomData<S>);

impl<S: RawState> StatePattern<S> for AnyStatePattern<S> {
    fn matches(&self, _state: &S) -> bool {
        true
    }
}
