use bevy_ecs::system::{Res, ResMut};

use crate::state::{CurrentState, NextState, State, StateMut, StateRef};

pub fn flush_state<S: State>(mut next: ResMut<NextState<S>>) {
    next.will_flush = true;
}

pub(crate) fn clear_flush_state<S: State>(mut next: ResMut<NextState<S>>) {
    next.will_flush = false;
}

// Sets the next state to the default state unless there's already a next state.
pub fn init_state<S: State + Default>(mut state: ResMut<NextState<S>>) {
    state.init();
}

pub fn flush_init_state<S: State + Default>(mut state: ResMut<NextState<S>>) {
    state.flush().init();
}

// Sets the next state to the default state.
pub fn restart_state<S: State + Default>(mut state: ResMut<NextState<S>>) {
    state.restart();
}

pub fn flush_restart_state<S: State + Default>(mut state: ResMut<NextState<S>>) {
    state.flush().restart();
}

// Sets the next state to the given value.
pub fn set_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state| {
        state.set(value.clone());
    }
}

pub fn flush_set_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state| {
        state.flush().set(value.clone());
    }
}

// Alias for `set_state`.
pub fn insert_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state| {
        state.insert(value.clone());
    }
}

pub fn flush_insert_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state| {
        state.flush().insert(value.clone());
    }
}

pub fn remove_state<S: State>(mut state: ResMut<NextState<S>>) {
    state.remove();
}

pub fn flush_remove_state<S: State>(mut state: ResMut<NextState<S>>) {
    state.flush().remove();
}

pub fn refresh_state<S: State>(mut state: StateMut<S>) {
    state.refresh();
}

pub fn flush_refresh_state<S: State>(mut state: StateMut<S>) {
    state.flush().refresh();
}

pub fn state_would_have_been_absent<S: State>(state: Res<CurrentState<S>>) -> bool {
    state.is_absent()
}

pub fn state_would_have_been_present<S: State>(state: Res<CurrentState<S>>) -> bool {
    state.is_present()
}

pub fn state_would_have_been_in<S: State>(value: S) -> impl Fn(Res<CurrentState<S>>) -> bool {
    move |state| state.is_in(&value)
}

pub fn state_would_be_absent<S: State>(state: Res<NextState<S>>) -> bool {
    state.would_be_absent()
}

pub fn state_would_be_present<S: State>(state: Res<NextState<S>>) -> bool {
    state.would_be_present()
}

pub fn state_would_be_in<S: State>(value: S) -> impl Fn(Res<NextState<S>>) -> bool {
    move |state| state.would_be_in(&value)
}

// Alias for `state_would_have_been_absent`.
pub fn state_is_absent<S: State>(state: Res<CurrentState<S>>) -> bool {
    state_would_have_been_absent(state)
}

// Alias for `state_would_have_been_present`.
pub fn state_is_present<S: State>(state: Res<CurrentState<S>>) -> bool {
    state_would_have_been_present(state)
}

// Alias for `state_would_have_been_in`.
pub fn state_is_in<S: State>(value: S) -> impl Fn(Res<CurrentState<S>>) -> bool {
    state_would_have_been_in(value)
}

// Alias for `state_would_have_been_in`.
pub fn in_state<S: State>(value: S) -> impl Fn(Res<CurrentState<S>>) -> bool {
    state_would_have_been_in(value)
}

// Alias for `state_would_have_been_present`.
pub fn state_would_be_exiting<S: State>(state: Res<CurrentState<S>>) -> bool {
    state_would_have_been_present(state)
}

// Alias for `state_would_have_been_in`.
pub fn state_would_exit<S: State>(from: S) -> impl Fn(Res<CurrentState<S>>) -> bool {
    state_would_have_been_in(from)
}

// Alias for `state_would_be_present`.
pub fn state_would_be_entering<S: State>(state: Res<NextState<S>>) -> bool {
    state.would_be_present()
}

// Alias for `state_would_be_in`.
pub fn state_would_enter<S: State>(to: S) -> impl Fn(Res<NextState<S>>) -> bool {
    state_would_be_in(to)
}

pub fn state_would_be_inserted<S: State>(state: StateRef<S>) -> bool {
    state.would_be_inserted()
}

pub fn state_would_insert<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_insert(&value)
}

pub fn state_would_be_removed<S: State>(state: StateRef<S>) -> bool {
    state.would_be_removed()
}

pub fn state_would_remove<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_remove(&value)
}

pub fn state_would_mutate<S: State>(state: StateRef<S>) -> bool {
    state.would_mutate()
}

pub fn state_would_mutate_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_mutate_from(&from)
}

pub fn state_would_mutate_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_mutate_to(&to)
}

// Equivalent to `state_would_change_from_to`.
pub fn state_would_mutate_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_mutate_from_to(&from, &to)
}

// Equivalent to `state_would_stay_present`.
pub fn state_would_transition<S: State>(state: StateRef<S>) -> bool {
    state.would_transition()
}

pub fn state_would_transition_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_transition_from(&from)
}

pub fn state_would_transition_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_transition_to(&to)
}

pub fn state_would_transition_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_transition_from_to(&from, &to)
}

pub fn state_would_change<S: State>(state: StateRef<S>) -> bool {
    state.would_change()
}

pub fn state_would_change_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_change_from(&from)
}

pub fn state_would_change_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_change_to(&to)
}

// Equivalent to `state_would_mutate_from_to`.
pub fn state_would_change_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_change_from_to(&from, &to)
}

pub fn state_would_stay<S: State>(state: StateRef<S>) -> bool {
    state.would_stay()
}

pub fn state_would_stay_absent<S: State>(state: StateRef<S>) -> bool {
    state.would_stay_absent()
}

// Does not imply the state would not change; only that it would be present both before and after.
// Equivalent to `state_would_transition`.
pub fn state_would_stay_present<S: State>(state: StateRef<S>) -> bool {
    state.would_stay_present()
}

// Equivalent to `state_would_refresh_as`.
pub fn state_would_stay_as<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_stay_as(&value)
}

pub fn state_would_refresh<S: State>(state: StateRef<S>) -> bool {
    state.would_refresh()
}

// Equivalent to `state_would_stay_as`.
pub fn state_would_refresh_as<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_refresh_as(&value)
}

pub fn state_will_flush<S: State>(state: Res<NextState<S>>) -> bool {
    state.will_flush()
}

pub fn state_will_have_been_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_have_been_absent()
}

pub fn state_will_have_been_present<S: State>(state: StateRef<S>) -> bool {
    state.will_have_been_present()
}

pub fn state_will_have_been_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_have_been_in(&value)
}

pub fn state_will_be_absent<S: State>(state: Res<NextState<S>>) -> bool {
    state.will_be_absent()
}

pub fn state_will_be_present<S: State>(state: Res<NextState<S>>) -> bool {
    state.will_be_present()
}

pub fn state_will_be_in<S: State>(value: S) -> impl Fn(Res<NextState<S>>) -> bool {
    move |state| state.will_be_in(&value)
}

// Alias for `state_will_have_been_present`.
pub fn state_will_be_exiting<S: State>(state: StateRef<S>) -> bool {
    state_will_have_been_present(state)
}

// Alias for `state_will_have_been_in`.
pub fn state_will_exit<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    state_will_have_been_in(value)
}

// Alias for `state_will_be_present`.
pub fn state_will_be_entering<S: State>(state: Res<NextState<S>>) -> bool {
    state_will_be_present(state)
}

// Alias for `state_will_be_in`.
pub fn state_will_enter<S: State>(value: S) -> impl Fn(Res<NextState<S>>) -> bool {
    state_will_be_in(value)
}

pub fn state_will_be_inserted<S: State>(state: StateRef<S>) -> bool {
    state.will_be_inserted()
}

pub fn state_will_insert<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_insert(&value)
}

pub fn state_will_be_removed<S: State>(state: StateRef<S>) -> bool {
    state.will_be_removed()
}

pub fn state_will_remove<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_remove(&value)
}

pub fn state_will_mutate<S: State>(state: StateRef<S>) -> bool {
    state.will_mutate()
}

pub fn state_will_mutate_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_mutate_from(&from)
}

pub fn state_will_mutate_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_mutate_to(&to)
}

// Equivalent to `state_will_change_from_to`.
pub fn state_will_mutate_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_mutate_from_to(&from, &to)
}

// Equivalent to `state_will_stay_present`.
pub fn state_will_transition<S: State>(state: StateRef<S>) -> bool {
    state.will_transition()
}

pub fn state_will_transition_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_transition_from(&from)
}

pub fn state_will_transition_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_transition_to(&to)
}

pub fn state_will_transition_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_transition_from_to(&from, &to)
}

pub fn state_will_change<S: State>(state: StateRef<S>) -> bool {
    state.will_change()
}

pub fn state_will_change_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_change_from(&from)
}

pub fn state_will_change_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_change_to(&to)
}

// Equivalent to `state_will_mutate_from_to`.
pub fn state_will_change_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_change_from_to(&from, &to)
}

pub fn state_will_stay<S: State>(state: StateRef<S>) -> bool {
    state.will_stay()
}

pub fn state_will_stay_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_stay_absent()
}

// Does not imply the state will not change; only that it will be present both before and after.
// Equivalent to `state_will_transition`.
pub fn state_will_stay_present<S: State>(state: StateRef<S>) -> bool {
    state.will_stay_present()
}

// Equivalent to `state_will_refresh_as`.
pub fn state_will_stay_as<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_stay_as(&value)
}

pub fn state_will_refresh<S: State>(state: StateRef<S>) -> bool {
    state.will_refresh()
}

// Equivalent to `state_will_stay_as`.
pub fn state_will_refresh_as<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_refresh_as(&value)
}
