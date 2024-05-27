use std::marker::PhantomData;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Res, ResMut},
};

use crate::{
    buffer::{CurrentState, NextState_, StateMut, StateRef},
    schedule::StateFlushSet,
};

pub trait State_: RawState + Clone + PartialEq + Eq {}

impl<T: RawState + Clone + PartialEq + Eq> State_ for T {}

pub trait RawState: 'static + Send + Sync + Sized {
    const ANY: UniversalStateSet<Self> = UniversalStateSet(PhantomData);

    fn with<F: Fn(&Self) -> bool + 'static + Send + Sync>(f: F) -> FunctionalStateSet<Self, F> {
        FunctionalStateSet(f, PhantomData)
    }

    fn is_disabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_disabled()
    }

    fn will_be_disabled(state: Res<NextState_<Self>>) -> bool {
        state.will_be_disabled()
    }

    fn is_enabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_enabled()
    }

    fn will_be_enabled(state: Res<NextState_<Self>>) -> bool {
        state.will_be_enabled()
    }

    fn on_flush<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Flush)
    }

    fn on_transition<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Transition)
    }

    fn disable(mut state: ResMut<NextState_<Self>>) {
        state.disable();
    }

    fn set_flush(flush: bool) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.set_flush(flush);
        }
    }
}

pub trait RawStateExtClone: RawState + Clone {
    fn enable_as(value: Self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.enable_as(value.clone());
        }
    }

    fn toggle_as(value: Self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| state.toggle_as(value.clone())
    }

    fn enter(self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.enter(self.clone());
        }
    }

    fn reset(mut state: StateMut<Self>) {
        state.reset();
    }

    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }
}

impl<S: RawState + Clone> RawStateExtClone for S {}

pub trait RawStateExtDefault: RawState + Default {
    fn enable(mut state: ResMut<NextState_<Self>>) {
        state.enable();
    }

    fn toggle(mut state: ResMut<NextState_<Self>>) {
        state.toggle();
    }

    fn restart(mut state: ResMut<NextState_<Self>>) {
        state.restart();
    }
}

impl<T: RawState + Default> RawStateExtDefault for T {}

pub trait ContainsState<S: RawState>: 'static + Send + Sync + Sized {
    fn contains_state(&self, state: &S) -> bool;

    // Equivalent to `will_exit`.
    fn will_update(self) -> impl Fn(Res<CurrentState<S>>) -> bool + 'static + Send + Sync {
        self.will_exit()
    }

    fn on_update<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(self.will_update())
    }

    fn will_exit(self) -> impl Fn(Res<CurrentState<S>>) -> bool + 'static + Send + Sync {
        move |state| state.is_in(&self)
    }

    fn on_exit<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_exit())
            .in_set(StateFlushSet::<S>::Exit)
    }

    fn will_disable(self) -> impl Fn(StateRef<S>) -> bool + 'static + Send + Sync {
        move |state| matches!(state.get(), (Some(x), None) if self.contains_state(x))
    }

    fn on_disable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_disable())
            .in_set(StateFlushSet::<S>::Exit)
    }

    fn will_enter(self) -> impl Fn(Res<NextState_<S>>) -> bool + 'static + Send + Sync {
        move |state| matches!(state.get(), Some(x) if self.contains_state(x))
    }

    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(StateFlushSet::<S>::Enter)
    }

    fn will_enable(self) -> impl Fn(StateRef<S>) -> bool + 'static + Send + Sync {
        move |state| matches!(state.get(), (None, Some(x)) if self.contains_state(x))
    }

    fn on_enable<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enable())
            .in_set(StateFlushSet::<S>::Enter)
    }
}

pub trait ContainsStateExtEq<S: RawState + Eq>: ContainsState<S> {
    fn will_refresh(self) -> impl Fn(StateRef<S>) -> bool + 'static + Send + Sync {
        move |state| {
            matches!(
                state.get(),
                (Some(x), Some(y)) if x == y && self.contains_state(y),
            )
        }
    }

    fn on_refresh<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_refresh())
            .in_set(StateFlushSet::<S>::Transition)
    }
}

impl<S: RawState + Eq, T: ContainsState<S>> ContainsStateExtEq<S> for T {}

impl<S: RawState + Eq> ContainsState<S> for S {
    fn contains_state(&self, state: &S) -> bool {
        self == state
    }
}

pub struct FunctionalStateSet<S: RawState, F: Fn(&S) -> bool>(F, PhantomData<S>);

impl<S: RawState, F: Fn(&S) -> bool + 'static + Send + Sync> ContainsState<S>
    for FunctionalStateSet<S, F>
{
    fn contains_state(&self, state: &S) -> bool {
        self.0(state)
    }
}

impl<S: RawState, F: Fn(&S) -> bool> FunctionalStateSet<S, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

// Helper macro for building a pattern matching state set.
#[macro_export]
macro_rules! state {
    ($pattern:pat $(if $guard:expr)? $(,)?) => {
        pyri_state::state::FunctionalStateSet::new(
            |state| matches!(*state, $pattern $(if $guard)?),
        )
    };
}

pub struct UniversalStateSet<S: RawState>(PhantomData<S>);

impl<S: RawState> ContainsState<S> for UniversalStateSet<S> {
    fn contains_state(&self, _state: &S) -> bool {
        true
    }
}
