use bevy_ecs::{
    event::EventWriter,
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Res, ResMut},
};

use crate::{
    buffer::{CurrentState, NextState, StateMut, StateRef},
    schedule::{OnState, StateFlushEvent},
};

pub trait State: 'static + Send + Sync + Clone {
    fn is_absent(state: Res<CurrentState<Self>>) -> bool {
        state.is_absent()
    }

    // Equivalent to `will_any_exit`.
    fn is_present(state: Res<CurrentState<Self>>) -> bool {
        state.is_present()
    }

    fn on_any_update<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(Self::is_present)
    }

    // NOTE: This is the only `will_xyz` condition that actually checks that the state is set to flush.
    //       On the other hand, every `on_xyz` does require that the state is set to flush.
    fn will_any_flush(state: Res<NextState<Self>>) -> bool {
        state.flush
    }

    fn on_any_flush<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        // `will_any_flush` is already implied by the system set `OnState::<Self>::Flush`
        systems.in_set(OnState::<Self>::Flush)
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
            .in_set(OnState::<Self>::Flush)
    }

    // Equivalent to `is_present`.
    fn will_any_exit(state: Res<CurrentState<Self>>) -> bool {
        state.will_any_exit()
    }

    fn on_any_exit<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_exit)
            .in_set(OnState::<Self>::Exit)
    }

    fn will_exit_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.will_exit_and(&test)
    }

    fn on_exit_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_exit_and(test))
            .in_set(OnState::<Self>::Exit)
    }

    fn will_any_enter(state: Res<NextState<Self>>) -> bool {
        state.will_any_enter()
    }

    fn on_any_enter<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_enter)
            .in_set(OnState::<Self>::Enter)
    }

    fn will_enter_and(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(Res<NextState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.will_enter_and(&test)
    }

    fn on_enter_and<M>(
        test: impl Fn(&Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_enter_and(test))
            .in_set(OnState::<Self>::Enter)
    }

    fn will_any_transition(state: StateRef<Self>) -> bool {
        state.will_any_transition()
    }

    fn on_any_transition<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_transition)
            .in_set(OnState::<Self>::Flush)
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
            .in_set(OnState::<Self>::Flush)
    }

    fn will_any_remove(state: StateRef<Self>) -> bool {
        state.will_any_remove()
    }

    fn on_any_remove<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_remove)
            .in_set(OnState::<Self>::Exit)
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
            .in_set(OnState::<Self>::Exit)
    }

    fn will_any_insert(state: StateRef<Self>) -> bool {
        state.will_any_insert()
    }

    fn on_any_insert<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_insert)
            .in_set(OnState::<Self>::Enter)
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
            .in_set(OnState::<Self>::Enter)
    }

    fn remove(mut state: ResMut<NextState<Self>>) {
        state.remove();
    }

    fn insert(self) -> impl Fn(ResMut<NextState<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.insert(self.clone());
        }
    }

    // Alias for `insert`.
    fn set(value: Self) -> impl Fn(ResMut<NextState<Self>>) + 'static + Send + Sync {
        Self::insert(value)
    }

    fn stay(mut state: StateMut<Self>) {
        state.stay();
    }

    fn refresh(mut state: StateMut<Self>) {
        state.refresh();
    }

    fn set_flush(flush: bool) -> impl Fn(ResMut<NextState<Self>>) + 'static + Send + Sync {
        move |mut state| {
            state.set_flush(flush);
        }
    }

    // Shouldn't be necessary during normal usage.
    fn send_flush_event(state: StateRef<Self>, mut events: EventWriter<StateFlushEvent<Self>>) {
        events.send(StateFlushEvent {
            before: state.current.inner.clone(),
            after: state.next.inner.clone(),
        });
    }

    // Shouldn't be necessary during normal usage.
    fn apply_flush(mut current: ResMut<CurrentState<Self>>, next: Res<NextState<Self>>) {
        current.inner.clone_from(&next.inner);
    }
}

pub trait StateExtEq: State + Eq {
    // Equivalent to `will_exit`.
    fn is_in(self) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.is_in(&self)
    }

    fn on_update<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems.run_if(self.is_in())
    }

    // Equivalent to `is_in`.
    fn will_exit(self) -> impl Fn(Res<CurrentState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.will_exit(&self)
    }

    fn on_exit<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_exit())
            .in_set(OnState::<Self>::Exit)
    }

    fn will_enter(self) -> impl Fn(Res<NextState<Self>>) -> bool + 'static + Send + Sync {
        move |state| state.will_enter(&self)
    }

    fn on_enter<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_enter())
            .in_set(OnState::<Self>::Enter)
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
            .in_set(OnState::<Self>::Flush)
    }

    fn will_any_change(state: StateRef<Self>) -> bool {
        state.will_any_change()
    }

    fn on_any_change<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_change)
            .in_set(OnState::<Self>::Flush)
    }

    fn will_change_and(
        test: impl Fn(&Self, &Self) -> bool + 'static + Send + Sync,
    ) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_change_and(&test)
    }

    fn on_change_and<M>(
        test: impl Fn(&Self, &Self) -> bool + 'static + Send + Sync,
        systems: impl IntoSystemConfigs<M>,
    ) -> SystemConfigs {
        systems
            .run_if(Self::will_change_and(test))
            .in_set(OnState::<Self>::Flush)
    }

    fn will_any_refresh(state: StateRef<Self>) -> bool {
        state.will_any_refresh()
    }

    fn on_any_refresh<M>(systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(Self::will_any_refresh)
            .in_set(OnState::<Self>::Flush)
    }

    fn will_refresh(self) -> impl Fn(StateRef<Self>) -> bool + 'static + Send + Sync {
        move |state| state.will_refresh(&self)
    }

    fn on_refresh<M>(self, systems: impl IntoSystemConfigs<M>) -> SystemConfigs {
        systems
            .run_if(self.will_refresh())
            .in_set(OnState::<Self>::Flush)
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
            .in_set(OnState::<Self>::Flush)
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
