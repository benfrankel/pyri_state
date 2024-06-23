//! Store the [`NextState`] as a [`StateSequenceIndex`] that reads from a [`StateSequence`].
//!
//! Enable the `sequence` feature flag to use this module.
//!
//! This can be used to implement phases in a turn-based game, for example.

use std::marker::PhantomData;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::system::{lifetimeless::SRes, Res, ResMut, Resource, SystemParamItem};

use crate::state::{NextState, State};

/// A [`Resource`] that stores a sequence of the [`State`] type `S`.
///
/// Indexed into by the [`NextState`] type [`StateSequenceIndex<S>`].
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateSequence<S: State>(
    /// The sequence of states.
    pub Vec<Option<S>>,
);

impl<S: State> StateSequence<S> {
    /// Create a new `StateSequence` from a sequence of `Option<S>`.
    pub fn new(sequence: impl Into<Vec<Option<S>>>) -> Self {
        Self(sequence.into())
    }
}

/// A [`NextState`] type that stores the [`State`] type `S` as an index into
/// an external [`StateSequence<S>`] resource.
///
/// Using this as [`State::Next`] unlocks the [`StateSequenceIndexMut`] extension trait for `S`.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateSequenceIndex<S: State>(
    /// The index into the sequence, or `None` if not in the sequence.
    pub Option<usize>,
    PhantomData<S>,
);

impl<S: State> NextState for StateSequenceIndex<S> {
    type State = S;

    type Param = SRes<StateSequence<Self::State>>;

    fn empty() -> Self {
        Self(None, PhantomData)
    }

    fn get_state<'s>(&'s self, param: &'s SystemParamItem<Self::Param>) -> Option<&'s Self::State> {
        self.0
            .and_then(|index| param.0.get(index))
            .and_then(Option::as_ref)
    }
}

impl<S: State> Default for StateSequenceIndex<S> {
    fn default() -> Self {
        Self(Some(0), PhantomData)
    }
}

impl<S: State> StateSequenceIndex<S> {
    /// Create a new `StateSequenceIndex` from an initial index, clamped within bounds.
    pub fn new(index: isize, len: usize) -> Self {
        let mut this = Self::empty();
        this.seek(index, len);
        this
    }

    /// Set the sequence index and clamp within bounds.
    pub fn seek(&mut self, to: isize, len: usize) {
        self.0 = (len > 0).then(|| to.clamp(0, len as isize - 1) as usize);
    }

    /// Adjust the sequence index and clamp within bounds.
    pub fn step(&mut self, by: isize, len: usize) {
        self.seek(self.0.unwrap_or_default() as isize + by, len);
    }

    /// Step the sequence index forwards by 1 and clamp within bounds.
    pub fn next(&mut self, len: usize) {
        self.step(1, len);
    }

    /// Step the sequence index backwards by 1 and clamp within bounds.
    pub fn prev(&mut self, len: usize) {
        self.step(-1, len);
    }

    /// Set the sequence index and wrap within bounds.
    pub fn wrapping_seek(&mut self, to: isize, len: usize) {
        self.0 = (len > 0).then(|| to.rem_euclid(len as isize) as usize);
    }

    /// Adjust the sequence index and wrap within bounds.
    pub fn wrapping_step(&mut self, by: isize, len: usize) {
        self.wrapping_seek(self.0.unwrap_or_default() as isize + by, len);
    }

    /// Step the sequence index forwards by 1 and wrap within bounds.
    pub fn wrapping_next(&mut self, len: usize) {
        self.wrapping_step(1, len);
    }

    /// Step the sequence index backwards by 1 and wrap within bounds.
    pub fn wrapping_prev(&mut self, len: usize) {
        self.wrapping_step(-1, len);
    }
}

/// An extension trait for [`State`] types with [`StateSequenceIndex`] as their [`NextState`] type.
pub trait StateSequenceIndexMut: State {
    /// A system that sets the sequence index and clamps within bounds.
    fn seek(
        to: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<StateSequenceIndex<Self>>, Res<StateSequence<Self>>)
    {
        move |mut index, sequence| index.seek(to, sequence.0.len())
    }

    /// A system that adjusts the sequence index and clamps within bounds.
    fn step(
        by: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<StateSequenceIndex<Self>>, Res<StateSequence<Self>>)
    {
        move |mut index, sequence| index.step(by, sequence.0.len())
    }

    /// A system that steps the sequence index forwards by 1 and clamps within bounds.
    fn next(mut index: ResMut<StateSequenceIndex<Self>>, sequence: Res<StateSequence<Self>>) {
        index.step(1, sequence.0.len());
    }

    /// A system that steps the sequence index backwards by 1 and clamps within bounds.
    fn prev(mut index: ResMut<StateSequenceIndex<Self>>, sequence: Res<StateSequence<Self>>) {
        index.step(-1, sequence.0.len());
    }

    /// A system that sets the sequence index and wraps within bounds.
    fn wrapping_seek(
        to: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<StateSequenceIndex<Self>>, Res<StateSequence<Self>>)
    {
        move |mut index, sequence| index.wrapping_seek(to, sequence.0.len())
    }

    /// A system that adjusts the sequence index and wraps within bounds.
    fn wrapping_step(
        by: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<StateSequenceIndex<Self>>, Res<StateSequence<Self>>)
    {
        move |mut index, sequence| index.wrapping_step(by, sequence.0.len())
    }

    /// A system that steps the sequence index forwards by 1 and wraps within bounds.
    fn wrapping_next(
        mut index: ResMut<StateSequenceIndex<Self>>,
        sequence: Res<StateSequence<Self>>,
    ) {
        index.wrapping_step(1, sequence.0.len());
    }

    /// A system that steps the sequence index backwards by 1 and wraps within bounds.
    fn wrapping_prev(
        mut index: ResMut<StateSequenceIndex<Self>>,
        sequence: Res<StateSequence<Self>>,
    ) {
        index.wrapping_step(-1, sequence.0.len());
    }
}

impl<S: State<Next = StateSequenceIndex<S>>> StateSequenceIndexMut for S {}
