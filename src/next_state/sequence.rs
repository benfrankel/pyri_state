//! Store the [`NextState`] as a [`NextStateIndex`] that reads from a [`NextStateSequence`].
//!
//! Enable the `sequence` feature flag to use this module.
//!
//! This can be used to implement phases in a turn-based game, for example.

use alloc::vec::Vec;
use core::marker::PhantomData;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{Res, ResMut, SystemParamItem, lifetimeless::SRes},
};

use crate::{next_state::NextState, state::State};

/// A [`Resource`] that stores a sequence of next states for the [`State`] type `S`.
///
/// Indexed into by the [`NextState`] type [`NextStateIndex<S>`].
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct NextStateSequence<S: State>(
    /// The sequence of states.
    pub Vec<Option<S>>,
);

impl<S: State> NextStateSequence<S> {
    /// Create a new `NextStateSequence` from a sequence of `Option<S>`.
    pub fn new(sequence: impl Into<Vec<Option<S>>>) -> Self {
        Self(sequence.into())
    }
}

/// A [`NextState`] type that stores the [`State`] type `S` as an index into
/// an external [`NextStateSequence<S>`] resource.
///
/// Using this as [`State::Next`] unlocks the [`NextStateIndexMut`] extension trait for `S`.
#[derive(Resource, Component, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct NextStateIndex<S: State>(
    /// The index into the sequence, or `None` if not in the sequence.
    pub Option<usize>,
    PhantomData<S>,
);

impl<S: State> NextState for NextStateIndex<S> {
    type State = S;

    type Param = SRes<NextStateSequence<Self::State>>;

    fn empty() -> Self {
        Self(None, PhantomData)
    }

    fn next_state<'s>(
        &'s self,
        param: &'s SystemParamItem<Self::Param>,
    ) -> Option<&'s Self::State> {
        self.0
            .and_then(|index| param.0.get(index))
            .and_then(Option::as_ref)
    }
}

impl<S: State> Default for NextStateIndex<S> {
    fn default() -> Self {
        Self(Some(0), PhantomData)
    }
}

impl<S: State> NextStateIndex<S> {
    /// Create a new `NextStateIndex` from an initial index, clamped within bounds.
    pub fn new(index: isize, len: usize) -> Self {
        let mut this = Self::empty();
        this.seek(index, len);
        this
    }

    /// Set the index and clamp within bounds.
    pub fn seek(&mut self, to: isize, len: usize) {
        self.0 = (len > 0).then(|| to.clamp(0, len as isize - 1) as usize);
    }

    /// Adjust the index and clamp within bounds.
    pub fn step(&mut self, by: isize, len: usize) {
        self.seek(self.0.unwrap_or_default() as isize + by, len);
    }

    /// Step the index forwards by 1 and clamp within bounds.
    pub fn next(&mut self, len: usize) {
        self.step(1, len);
    }

    /// Step the index backwards by 1 and clamp within bounds.
    pub fn prev(&mut self, len: usize) {
        self.step(-1, len);
    }

    /// Set the index and wrap within bounds.
    pub fn wrapping_seek(&mut self, to: isize, len: usize) {
        self.0 = (len > 0).then(|| to.rem_euclid(len as isize) as usize);
    }

    /// Adjust the index and wrap within bounds.
    pub fn wrapping_step(&mut self, by: isize, len: usize) {
        self.wrapping_seek(self.0.unwrap_or_default() as isize + by, len);
    }

    /// Step the index forwards by 1 and wrap within bounds.
    pub fn wrapping_next(&mut self, len: usize) {
        self.wrapping_step(1, len);
    }

    /// Step the index backwards by 1 and wrap within bounds.
    pub fn wrapping_prev(&mut self, len: usize) {
        self.wrapping_step(-1, len);
    }
}

/// An extension trait for [`State`] types with [`NextStateIndex`] as their [`NextState`] type.
pub trait NextStateIndexMut: State {
    /// A system that sets the index and clamps within bounds.
    fn seek(
        to: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<NextStateIndex<Self>>, Res<NextStateSequence<Self>>)
    {
        move |mut index, sequence| index.seek(to, sequence.0.len())
    }

    /// A system that adjusts the index and clamps within bounds.
    fn step(
        by: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<NextStateIndex<Self>>, Res<NextStateSequence<Self>>)
    {
        move |mut index, sequence| index.step(by, sequence.0.len())
    }

    /// A system that steps the index forwards by 1 and clamps within bounds.
    fn next(mut index: ResMut<NextStateIndex<Self>>, sequence: Res<NextStateSequence<Self>>) {
        index.step(1, sequence.0.len());
    }

    /// A system that steps the index backwards by 1 and clamps within bounds.
    fn prev(mut index: ResMut<NextStateIndex<Self>>, sequence: Res<NextStateSequence<Self>>) {
        index.step(-1, sequence.0.len());
    }

    /// A system that sets the index and wraps within bounds.
    fn wrapping_seek(
        to: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<NextStateIndex<Self>>, Res<NextStateSequence<Self>>)
    {
        move |mut index, sequence| index.wrapping_seek(to, sequence.0.len())
    }

    /// A system that adjusts the index and wraps within bounds.
    fn wrapping_step(
        by: isize,
    ) -> impl 'static + Send + Sync + Fn(ResMut<NextStateIndex<Self>>, Res<NextStateSequence<Self>>)
    {
        move |mut index, sequence| index.wrapping_step(by, sequence.0.len())
    }

    /// A system that steps the index forwards by 1 and wraps within bounds.
    fn wrapping_next(
        mut index: ResMut<NextStateIndex<Self>>,
        sequence: Res<NextStateSequence<Self>>,
    ) {
        index.wrapping_step(1, sequence.0.len());
    }

    /// A system that steps the index backwards by 1 and wraps within bounds.
    fn wrapping_prev(
        mut index: ResMut<NextStateIndex<Self>>,
        sequence: Res<NextStateSequence<Self>>,
    ) {
        index.wrapping_step(-1, sequence.0.len());
    }
}

impl<S: State<Next = NextStateIndex<S>>> NextStateIndexMut for S {}
