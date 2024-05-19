use bevy_ecs::{
    event::EventWriter,
    system::{Res, ResMut},
};

use crate::{
    schedule::StateTransitionEvent,
    state::{CurrentState, NextState, State, StateMut, StateRef},
};

pub(crate) fn send_transition_event<S: State>(
    state: StateRef<S>,
    mut events: EventWriter<StateTransitionEvent<S>>,
) {
    events.send(StateTransitionEvent {
        before: state.current.inner.clone(),
        after: state.next.inner.clone(),
    });
}

pub(crate) fn apply_flush_state<S: State>(
    mut current: ResMut<CurrentState<S>>,
    next: Res<NextState<S>>,
) {
    current.inner = next.inner.clone();
}

pub fn flush_state<S: State>(mut next: ResMut<NextState<S>>) {
    next.will_flush = true;
}

pub(crate) fn clear_flush_state<S: State>(mut next: ResMut<NextState<S>>) {
    next.will_flush = false;
}

// Sets the next state to the current or default state unless there's already a next state.
pub fn init_state<S: State + Default>(mut state: StateMut<S>) {
    state.init();
}

// Sets the next state to the default state even if there is already a next state.
pub fn restart_state<S: State + Default>(mut state: StateMut<S>) {
    state.restart();
}

// Sets the next state to the given value.
pub fn set_state<S: State>(value: S) -> impl Fn(StateMut<S>) {
    move |mut state| {
        state.set(value.clone());
    }
}

// Alias for `set_state`.
pub fn insert_state<S: State>(value: S) -> impl Fn(StateMut<S>) {
    move |mut state| {
        state.insert(value.clone());
    }
}

pub fn remove_state<S: State>(mut state: StateMut<S>) {
    state.remove();
}

// This is a no-op if the state is absent.
pub fn refresh_state<S: State>(mut state: StateMut<S>) {
    state.refresh();
}

// Alias for `state_would_have_been_absent`.
pub fn state_is_absent<S: State>(state: StateRef<S>) -> bool {
    state.is_absent()
}

// Alias for `state_would_have_been_present`.
pub fn state_is_present<S: State>(state: StateRef<S>) -> bool {
    state.is_present()
}

// Alias for `state_would_have_been_in`.
pub fn state_is_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.is_in(&value)
}

// Alias for `state_is_in`.
pub fn in_state<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    state_is_in(value)
}

pub fn state_would_have_been_absent<S: State>(state: StateRef<S>) -> bool {
    state.would_have_been_absent()
}

// Equivalent to `state_would_be_exiting`.
pub fn state_would_have_been_present<S: State>(state: StateRef<S>) -> bool {
    state.would_have_been_present()
}

pub fn state_would_have_been_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_have_been_in(&value)
}

pub fn state_would_be_absent<S: State>(state: StateRef<S>) -> bool {
    state.would_be_absent()
}

// Equivalent to `state_would_be_entering`.
pub fn state_would_be_present<S: State>(state: StateRef<S>) -> bool {
    state.would_be_present()
}

pub fn state_would_be_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_be_in(&value)
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

// Equivalent to `state_would_have_been_present`.
pub fn state_would_be_exiting<S: State>(state: StateRef<S>) -> bool {
    state.would_be_exiting()
}

// Equivalent to `state_would_mutate_from`.
pub fn state_would_exit<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_exit(&from)
}

// Equivalent to `state_would_be_present`.
pub fn state_would_be_entering<S: State>(state: StateRef<S>) -> bool {
    state.would_be_entering()
}

// Equivalent to `state_would_mutate_to`.
pub fn state_would_enter<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_enter(&to)
}

pub fn state_would_mutate<S: State>(state: StateRef<S>) -> bool {
    state.would_mutate()
}

// Equivalent to `state_would_exit`.
pub fn state_would_mutate_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.would_mutate_from(&from)
}

// Equivalent to `state_would_enter`.
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

pub fn state_will_flush<S: State>(state: StateRef<S>) -> bool {
    state.will_flush()
}

pub fn state_will_have_been_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_have_been_absent()
}

// Equivalent to `state_will_be_exiting`.
pub fn state_will_have_been_present<S: State>(state: StateRef<S>) -> bool {
    state.will_have_been_present()
}

pub fn state_will_have_been_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_have_been_in(&value)
}

pub fn state_will_be_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_be_absent()
}

// Equivalent to `state_will_be_entering`.
pub fn state_will_be_present<S: State>(state: StateRef<S>) -> bool {
    state.will_be_present()
}

pub fn state_will_be_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_be_in(&value)
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

// Equivalent to `state_will_have_been_present`.
pub fn state_will_be_exiting<S: State>(state: StateRef<S>) -> bool {
    state.will_be_exiting()
}

// Equivalent to `state_will_mutate_from`.
pub fn state_will_exit<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_exit(&value)
}

// Equivalent to `state_will_be_present`.
pub fn state_will_be_entering<S: State>(state: StateRef<S>) -> bool {
    state.will_be_entering()
}

// Equivalent to `state_will_mutate_to`.
pub fn state_will_enter<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_enter(&value)
}

pub fn state_will_mutate<S: State>(state: StateRef<S>) -> bool {
    state.will_mutate()
}

// Equivalent to `state_will_exit`.
pub fn state_will_mutate_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state| state.will_mutate_from(&from)
}

// Equivalent to `state_will_enter`.
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
