use std::{fmt::Debug, hash::Hash};

use bevy_ecs::{
    schedule::{IntoSystemConfigs, States, SystemConfigs},
    system::{Res, ResMut},
};

use crate::{
    buffer::{CurrentState, NextState_, StateMut, StateRef},
    schedule::StateFlushSet,
};

pub trait State_: RawState + Clone + PartialEq + Eq {}

impl<T: RawState + Clone + PartialEq + Eq> State_ for T {}

// Wrapper for compatibility with bevy states
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State_ + Hash + Debug>(pub Option<S>);

impl<S: State_ + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}

pub trait RawState: 'static + Send + Sync + Sized {
    fn is_absent(state: Res<CurrentState<Self>>) -> bool {
        state.is_absent()
    }

    fn is_present(state: Res<CurrentState<Self>>) -> bool {
        state.is_present()
    }

    fn is_present_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.is_present_and(&test)
    }

    fn on_any_update<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(Self::is_present)
    }

    fn on_update_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems.run_if(Self::is_present_and(test))
    }

    fn on_any_flush<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Flush)
    }

    fn will_flush_and(
        test: impl Fn(Option<&Self>, Option<&Self>) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_flush_and(&test)
    }

    fn on_flush_and<M>(
        test: impl Fn(Option<&Self>, Option<&Self>) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_flush_and(test))
            .in_set(StateFlushSet::<Self>::Flush)
    }

    // Equivalent to `is_present`.
    fn will_any_exit(state: Res<CurrentState<Self>>) -> bool {
        state.is_present()
    }

    fn on_any_exit<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Exit)
    }

    fn will_exit_and(
        test: impl Fn(&Self, Option<&Self>) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_exit_and(&test)
    }

    fn on_exit_and<M>(
        test: impl Fn(&Self, Option<&Self>) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_exit_and(test))
            .in_set(StateFlushSet::<Self>::Exit)
    }

    fn will_any_enter(state: Res<NextState_<Self>>) -> bool {
        state.will_be_present()
    }

    fn on_any_enter<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Enter)
    }

    fn will_enter_and(
        test: impl Fn(Option<&Self>, &Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_enter_and(&test)
    }

    fn on_enter_and<M>(
        test: impl Fn(Option<&Self>, &Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_enter_and(test))
            .in_set(StateFlushSet::<Self>::Enter)
    }

    fn will_any_transition(state: StateRef<Self>) -> bool {
        state.will_any_transition()
    }

    fn on_any_transition<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.in_set(StateFlushSet::<Self>::Transition)
    }

    fn will_transition_and(
        test: impl Fn(&Self, &Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_transition_and(&test)
    }

    fn on_transition_and<M>(
        test: impl Fn(&Self, &Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_transition_and(test))
            .in_set(StateFlushSet::<Self>::Transition)
    }

    fn will_any_remove(state: StateRef<Self>) -> bool {
        state.will_any_remove()
    }

    fn on_any_remove<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_remove)
            .in_set(StateFlushSet::<Self>::Exit)
    }

    fn will_remove_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_remove_and(&test)
    }

    fn on_remove_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_remove_and(test))
            .in_set(StateFlushSet::<Self>::Exit)
    }

    fn will_any_insert(state: StateRef<Self>) -> bool {
        state.will_any_insert()
    }

    fn on_any_insert<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_insert)
            .in_set(StateFlushSet::<Self>::Enter)
    }

    fn will_insert_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_insert_and(&test)
    }

    fn on_insert_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_insert_and(test))
            .in_set(StateFlushSet::<Self>::Enter)
    }

    fn remove(mut state: ResMut<NextState_<Self>>) {
        state.remove();
    }

    fn set_flush(flush: bool) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.set_flush(flush);
        }
    }
}

pub trait RawStateExtClone: RawState + Clone {
    fn insert(self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.insert(self.clone());
        }
    }

    // Alias for `insert`.
    fn set(value: Self) -> impl Fn(ResMut<NextState_<Self>>) + 'static + Send + Sync {
        Self::insert(value)
    }

    fn stay(mut state: StateMut<Self>) {
        state.stay();
    }

    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }
}

impl<S: RawState + Clone> RawStateExtClone for S {}

pub trait RawStateExtEq: RawState + Eq {
    // Equivalent to `will_exit`.
    fn will_update(self) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.is_in(&self)
    }

    fn on_update<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(self.will_update())
    }

    // Equivalent to `will_update`.
    fn will_exit(self) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.is_in(&self)
    }

    fn on_exit<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_exit())
            .in_set(StateFlushSet::<Self>::Exit)
    }

    fn will_enter(self) -> impl Fn(Res<NextState_<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.will_be_in(&self)
    }

    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(StateFlushSet::<Self>::Enter)
    }

    fn will_transition(
        before: Self,
        after: Self,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_transition(&before, &after)
    }

    fn on_transition<M>(
        before: Self,
        after: Self,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_transition(before, after))
            .in_set(StateFlushSet::<Self>::Transition)
    }

    fn will_any_change(state: StateRef<Self>) -> bool {
        state.will_any_change()
    }

    fn on_any_change<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_change)
            .in_set(StateFlushSet::<Self>::Flush)
    }

    fn will_change_and(
        test: impl Fn(Option<&Self>, Option<&Self>) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_change_and(&test)
    }

    fn on_change_and<M>(
        test: impl Fn(Option<&Self>, Option<&Self>) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_change_and(test))
            .in_set(StateFlushSet::<Self>::Flush)
    }

    fn will_any_refresh(state: StateRef<Self>) -> bool {
        state.will_any_refresh()
    }

    fn on_any_refresh<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_refresh)
            .in_set(StateFlushSet::<Self>::Transition)
    }

    fn will_refresh(self) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_refresh(&self)
    }

    fn on_refresh<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_refresh())
            .in_set(StateFlushSet::<Self>::Transition)
    }

    fn will_refresh_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_refresh_and(&test)
    }

    fn on_refresh_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_refresh_and(test))
            .in_set(StateFlushSet::<Self>::Transition)
    }
}

impl<T: RawState + Eq> RawStateExtEq for T {}

pub trait RawStateExtDefault: RawState + Default {
    fn init(mut state: ResMut<NextState_<Self>>) {
        state.init();
    }

    fn restart(mut state: ResMut<NextState_<Self>>) {
        state.restart();
    }
}

impl<T: RawState + Default> RawStateExtDefault for T {}
