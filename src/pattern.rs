//! TODO: Module-level documentation

use std::marker::PhantomData;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::Res,
};

use crate::{
    schedule::StateFlushSet,
    state::{CurrentState, NextStateRef, StateFlushRef, State_},
};

pub trait StatePattern<S: State_>: 'static + Send + Sync + Sized {
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

    fn will_disable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_disable(&self)
    }

    fn on_disable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_disable())
            .in_set(StateFlushSet::<S>::Exit)
    }

    fn will_enter(self) -> impl 'static + Send + Sync + Fn(NextStateRef<S>) -> bool {
        move |state| state.will_be_in(&self)
    }

    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(StateFlushSet::<S>::Enter)
    }

    fn will_enable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_enable(&self)
    }

    fn on_enable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enable())
            .in_set(StateFlushSet::<S>::Enter)
    }
}

pub trait StatePatternExtClone<S: State_>: StatePattern<S> + Clone {
    fn on_edge<M1, M2>(
        self,
        exit_systems: impl IntoSystemConfigs<M1>,
        enter_systems: impl IntoSystemConfigs<M2>,
    ) -> SystemConfigs {
        (
            self.clone().on_exit(exit_systems),
            self.on_enter(enter_systems),
        )
            .into_configs()
    }
}

impl<S: State_, P: StatePattern<S> + Clone> StatePatternExtClone<S> for P {}

pub trait StatePatternExtEq<S: State_ + Eq>: StatePattern<S> {
    fn will_refresh(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_refresh(&self)
    }

    fn on_refresh<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_refresh())
            .in_set(StateFlushSet::<S>::Transition)
    }
}

impl<S: State_ + Eq, P: StatePattern<S>> StatePatternExtEq<S> for P {}

impl<S: State_ + Eq> StatePattern<S> for S {
    fn matches(&self, state: &S) -> bool {
        self == state
    }
}

#[derive(Clone)]
pub struct FnStatePattern<S: State_, F>(pub(crate) F, pub(crate) PhantomData<S>)
where
    F: 'static + Send + Sync + Fn(&S) -> bool;

impl<S: State_, F> StatePattern<S> for FnStatePattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S) -> bool,
{
    fn matches(&self, state: &S) -> bool {
        self.0(state)
    }
}

impl<S: State_, F> FnStatePattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S) -> bool,
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

#[derive(Clone)]
pub struct AnyStatePattern<S: State_>(pub(crate) PhantomData<S>);

impl<S: State_> StatePattern<S> for AnyStatePattern<S> {
    fn matches(&self, _state: &S) -> bool {
        true
    }
}

pub trait StateTransitionPattern<S: State_>: 'static + Send + Sync + Sized {
    fn matches(&self, old: &S, new: &S) -> bool;

    fn will_transition(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_transition(&self)
    }

    fn on_transition<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_transition())
            .in_set(StateFlushSet::<S>::Transition)
    }
}

impl<S: State_, P1: StatePattern<S>, P2: StatePattern<S>> StateTransitionPattern<S> for (P1, P2) {
    fn matches(&self, old: &S, new: &S) -> bool {
        self.0.matches(old) && self.1.matches(new)
    }
}
