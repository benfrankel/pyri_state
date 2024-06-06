//! TODO: Module-level documentation

use bevy_ecs::system::{lifetimeless::SRes, ResMut, Resource, SystemParamItem};

use crate::{state::State_, storage::StateStorage};

// A fixed sequence of states with an index to the current state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    // TODO: In bevy 0.14 this will be possible.
    //reflect(Resource),
)]
pub struct StateSequence<S: State_> {
    sequence: Vec<Option<S>>,
    pub index: usize,
}

impl<S: State_> StateStorage<S> for StateSequence<S> {
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

impl<S: State_> StateSequence<S> {
    pub fn empty() -> Self {
        Self {
            sequence: vec![None],
            index: 0,
        }
    }

    pub fn new(sequence: impl Into<Vec<Option<S>>>) -> Self {
        let sequence = sequence.into();
        assert!(!sequence.is_empty());

        Self { sequence, index: 0 }
    }

    pub fn at(mut self, index: isize) -> Self {
        self.seek(index);
        self
    }

    pub fn get(&self) -> Option<&S> {
        self.sequence[self.index].as_ref()
    }

    pub fn seek(&mut self, to: isize) {
        self.index = to.clamp(0, self.sequence.len() as isize - 1) as usize;
    }

    pub fn step(&mut self, by: isize) {
        self.seek(self.index as isize + by);
    }

    pub fn next(&mut self) {
        self.step(1);
    }

    pub fn prev(&mut self) {
        self.step(-1);
    }

    pub fn wrapping_seek(&mut self, to: isize) {
        self.index = to.rem_euclid(self.sequence.len() as isize) as usize;
    }

    pub fn wrapping_step(&mut self, by: isize) {
        self.wrapping_seek(self.index as isize + by);
    }

    pub fn wrapping_next(&mut self) {
        self.wrapping_step(1);
    }

    pub fn wrapping_prev(&mut self) {
        self.wrapping_step(-1);
    }
}

// TODO: Do we want to always flush on seek? Or on seek to different index?
pub trait StateSequenceMut: State_ {
    fn seek(to: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.seek(to)
    }

    fn step(by: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.step(by)
    }

    fn next(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.step(1);
    }

    fn prev(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.step(-1);
    }

    fn wrapping_seek(to: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.wrapping_seek(to)
    }

    fn wrapping_step(by: isize) -> impl 'static + Send + Sync + Fn(ResMut<StateSequence<Self>>) {
        move |mut sequence| sequence.wrapping_step(by)
    }

    fn wrapping_next(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.wrapping_step(1);
    }

    fn wrapping_prev(mut sequence: ResMut<StateSequence<Self>>) {
        sequence.wrapping_step(-1);
    }
}

impl<S: State_<Storage = StateSequence<S>>> StateSequenceMut for S {}
