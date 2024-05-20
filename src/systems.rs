use bevy_ecs::system::{Res, ResMut};

use crate::state::{CurrentState, NextState, State};

// Alias for `insert_state`.
pub fn set_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    insert_state(value)
}

pub fn insert_state<S: State>(value: S) -> impl Fn(ResMut<NextState<S>>) {
    move |mut state| {
        state.insert(value.clone());
    }
}

pub fn in_state<S: State + Eq>(value: S) -> impl Fn(Res<CurrentState<S>>) -> bool {
    move |state| state.is_in(&value)
}
