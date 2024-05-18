use crate::state::{State, StateRef};

pub fn state_is_absent<S: State>(state: StateRef<S>) -> bool {
    state.is_absent()
}

pub fn state_will_be_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_be_absent()
}

pub fn state_is_present<S: State>(state: StateRef<S>) -> bool {
    state.is_present()
}

pub fn state_will_be_present<S: State>(state: StateRef<S>) -> bool {
    state.will_be_present()
}

pub fn state_will_remain_absent<S: State>(state: StateRef<S>) -> bool {
    state.will_remain_absent()
}

pub fn state_will_remain_present<S: State>(state: StateRef<S>) -> bool {
    state.will_remain_present()
}

pub fn state_will_be_inserted<S: State>(state: StateRef<S>) -> bool {
    state.will_be_inserted()
}

pub fn state_will_insert<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_insert(&value)
}

pub fn state_will_be_removed<S: State>(state: StateRef<S>) -> bool {
    state.will_be_removed()
}

pub fn state_will_remove<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_remove(&value)
}

pub fn state_is_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.is_in(&value)
}

pub fn state_will_be_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_be_in(&value)
}

pub fn state_will_stay<S: State>(state: StateRef<S>) -> bool {
    state.will_stay()
}

pub fn state_will_stay_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_stay_in(&value)
}

pub fn state_will_change<S: State>(state: StateRef<S>) -> bool {
    state.will_change()
}

pub fn state_will_change_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_change_from(&from)
}

pub fn state_will_change_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_change_to(&to)
}

pub fn state_will_change_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_change_from_to(&from, &to)
}

pub fn state_will_flush<S: State>(state: StateRef<S>) -> bool {
    state.will_flush()
}

pub fn state_will_flush_from<S: State>(from: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_flush_from(&from)
}

pub fn state_will_flush_to<S: State>(to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_flush_to(&to)
}

pub fn state_will_flush_from_to<S: State>(from: S, to: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_flush_from_to(&from, &to)
}

pub fn state_will_refresh<S: State>(state: StateRef<S>) -> bool {
    state.will_refresh()
}

pub fn state_will_refresh_in<S: State>(value: S) -> impl Fn(StateRef<S>) -> bool {
    move |state: StateRef<S>| state.will_refresh_in(&value)
}
