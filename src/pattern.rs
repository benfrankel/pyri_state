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

/// A type that can match a subset of states for the [`State_`] type `S`.
///
/// See the following extension traits with additional bounds on `S`:
///
/// - [`StatePatternExtClone`]
/// - [`StatePatternExtEq`]
pub trait StatePattern<S: State_>: 'static + Send + Sync + Sized {
    /// Check if the pattern matches a particular state.
    fn matches(&self, state: &S) -> bool;

    /// A run condition that checks if `S` is in a matching state.
    fn will_update(self) -> impl 'static + Send + Sync + Fn(Res<CurrentState<S>>) -> bool {
        self.will_exit()
    }

    /// Configure systems to run if `S` is in a matching state.
    fn on_update<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(self.will_update())
    }

    /// A run condition that checks if `S` will exit a matching state if triggered.
    fn will_exit(self) -> impl 'static + Send + Sync + Fn(Res<CurrentState<S>>) -> bool {
        move |state| state.is_in(&self)
    }

    /// Configure systems to run when `S` exits a matching state.
    fn on_exit<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_exit())
            .in_set(StateFlushSet::<S>::Exit)
    }

    /// A run condition that checks if `S` will become disabled from a matching state if triggered.
    fn will_disable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_disable(&self)
    }

    /// Configure systems to run when `S` is disabled from a matching state.
    fn on_disable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_disable())
            .in_set(StateFlushSet::<S>::Exit)
    }

    /// A run condition that checks if `S` will enter into a matching state if triggered.
    fn will_enter(self) -> impl 'static + Send + Sync + Fn(NextStateRef<S>) -> bool {
        move |state| state.will_be_in(&self)
    }

    /// Configure systems to run when `S` enters a matching state.
    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(StateFlushSet::<S>::Enter)
    }

    /// A run condition that checks if `S` will become enabled in a matching state if triggered.
    fn will_enable(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_enable(&self)
    }

    /// Configure systems to run when `S` becomes enabled in a matching state.
    fn on_enable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enable())
            .in_set(StateFlushSet::<S>::Enter)
    }
}

/// An extension trait for [`StatePattern<S>`] when `S` also implements `Clone`.
pub trait StatePatternExtClone<S: State_>: StatePattern<S> + Clone {
    /// Helper method for configuring [`on_exit`](StatePattern::on_exit) and
    /// [`on_enter`](StatePattern::on_enter) systems for the same `StatePattern`.
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

/// An extension trait for [`StatePattern<S>`] when `S` also implements `Eq`.
pub trait StatePatternExtEq<S: State_ + Eq>: StatePattern<S> {
    /// A run condition that checks if `S` will refresh in a matching state if triggered.
    fn will_refresh(self) -> impl 'static + Send + Sync + Fn(StateFlushRef<S>) -> bool {
        move |state| state.will_refresh(&self)
    }

    /// Configure systems to run when `S` refreshes in a matching state.
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

/// A [`StatePattern`] that matches any value of the [`State_`] type `S`.
///
/// The usual way to use `AnyStatePattern` is through the associated constant [`State_::ANY`]:
///
/// ```rust
/// Level::ANY.on_enter(reset_timer)
/// ```
///
#[derive(Clone)]
pub struct AnyStatePattern<S: State_>(pub(crate) PhantomData<S>);

impl<S: State_> StatePattern<S> for AnyStatePattern<S> {
    fn matches(&self, _state: &S) -> bool {
        true
    }
}

/// A [`StatePattern`] that runs a callback to determine which values of the [`State_`]
/// type `S` should match.
///
/// The usual way to construct a `FnStatePattern` is by using [`State_::with`]:
///
/// ```rust
/// Level::with(|x| x.0 < 4).on_refresh(my_systems)
/// ```
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

/// A helper macro for building a simple [`StatePattern`].
///
/// # Example
///
/// ```rust
/// state!(Level(4 | 7 | 10)).on_enter(save_checkpoint)
/// ```
#[macro_export]
macro_rules! state {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        pyri_state::pattern::FnStatePattern::new(
            |state| matches!(*state, $pattern $(if $guard)?),
        )
    };
}

/// A type that can match a subset of transitions for the [`State_`] type `S`.
pub trait StateTransitionPattern<S: State_>: 'static + Send + Sync + Sized {
    /// Check if the pattern matches a particular pair of states.
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
