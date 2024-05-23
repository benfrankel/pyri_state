use std::fmt::Debug;
use std::hash::Hash;

use bevy_ecs::schedule::States;

use crate::state::State_;

// Convenient trait alias for defining wrapper states like `StateStack<S: FullState>`.
pub trait FullState: State_ + Clone + PartialEq + Eq {}

impl<T: State_ + Clone + PartialEq + Eq> FullState for T {}

// Wrapper for compatibility with bevy states
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: FullState + Hash + Debug>(pub Option<S>);

impl<S: FullState + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}
