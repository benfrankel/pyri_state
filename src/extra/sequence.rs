//! Navigate a fixed [`StateSequence`] by index.
//!
//! This can be used to implement phases in a turn-based game, for example.

use bevy_ecs::system::{lifetimeless::SRes, ResMut, Resource, SystemParamItem};

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{state::State, storage::StateStorage};

/// A [`StateStorage`] type that stores the [`State`] type `S` in a fixed sequence with an
/// index to the next state.
///
/// Using this as storage unlocks the [`StateSequenceMut`] extension trait for `S`.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateSequence<S: State> {
    sequence: Vec<Option<S>>,
    index: usize,
}

impl<S: State> StateStorage<S> for StateSequence<S> {
    type Param = SRes<Self>;

    fn get_state<'s>(param: &'s SystemParamItem<Self::Param>) -> Option<&'s S> {
        param.get()
    }
}

#[cfg(feature = "bevy_app")]
impl<S: crate::app::AddState<AddStorage = Self>> crate::app::AddStateStorage for StateSequence<S> {
    type AddState = S;

    fn add_state_storage(app: &mut bevy_app::App, storage: Option<Self>) {
        app.insert_resource(storage.unwrap_or_else(StateSequence::empty));
    }
}

impl<S: State> StateSequence<S> {
    /// Create an emty `StateSequence`.
    pub fn empty() -> Self {
        Self {
            sequence: vec![None],
            index: 0,
        }
    }

    /// Create a new `StateSequence` from a sequence of states.
    ///
    /// Use [`Self::at`] to set the initial index to something other than 0.
    pub fn new(sequence: impl Into<Vec<Option<S>>>) -> Self {
        let sequence = sequence.into();
        assert!(!sequence.is_empty());

        Self { sequence, index: 0 }
    }

    /// Set the initial index and clamp within bounds.
    pub fn at(mut self, index: isize) -> Self {
        self.seek(index);
        self
    }

    /// Get a read-only reference to the next state.
    pub fn get(&self) -> Option<&S> {
        self.sequence[self.index].as_ref()
    }

    /// Set the sequence index and clamp within bounds.
    pub fn seek(&mut self, to: isize) {
        self.index = to.clamp(0, self.sequence.len() as isize - 1) as usize;
    }

    /// Adjust the sequence index and clamp within bounds.
    pub fn step(&mut self, by: isize) {
        self.seek(self.index as isize + by);
    }

    /// Step the sequence index forwards by 1 and clamp within bounds.
    pub fn next(&mut self) {
        self.step(1);
    }

    /// Step the sequence index backwards by 1 and clamp within bounds.
    pub fn prev(&mut self) {
        self.step(-1);
    }

    /// Set the sequence index and wrap within bounds.
    pub fn wrapping_seek(&mut self, to: isize) {
        self.index = to.rem_euclid(self.sequence.len() as isize) as usize;
    }

    /// Adjust the sequence index and wrap within bounds.
    pub fn wrapping_step(&mut self, by: isize) {
        self.wrapping_seek(self.index as isize + by);
    }

    /// Step the sequence index forwards by 1 and wrap within bounds.
    pub fn wrapping_next(&mut self) {
        self.wrapping_step(1);
    }

    /// Step the sequence index backwards by 1 and wrap within bounds.
    pub fn wrapping_prev(&mut self) {
        self.wrapping_step(-1);
    }
}

/// An extension trait for [`State`] types with [`StateSequence`] storage.
pub trait StateSequenceMut: State {
    /// A system that sets the sequence index and clamps within bounds.
    fn seek(to: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.seek(to)
    }

    /// A system that adjusts the sequence index and clamps within bounds.
    fn step(by: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.step(by)
    }

    /// A system that steps the sequence index forwards by 1 and clamps within bounds.
    fn next(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.step(1);
    }

    /// A system that steps the sequence index backwards by 1 and clamps within bounds.
    fn prev(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.step(-1);
    }

    /// A system that sets the sequence index and wraps within bounds.
    fn wrapping_seek(to: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.wrapping_seek(to)
    }

    /// A system that adjusts the sequence index and wraps within bounds.
    fn wrapping_step(by: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.wrapping_step(by)
    }

    /// A system that steps the sequence index forwards by 1 and wraps within bounds.
    fn wrapping_next(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.wrapping_step(1);
    }

    /// A system that steps the sequence index backwards by 1 and wraps within bounds.
    fn wrapping_prev(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.wrapping_step(-1);
    }
}

impl<S: State<Storage = StateSequence<S>>> StateSequenceMut for S {}
