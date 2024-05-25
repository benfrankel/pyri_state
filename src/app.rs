use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{InternedSystemSet, SystemSet},
    world::FromWorld,
};

use crate::{
    buffer::{BevyState, CurrentState, NextState_},
    prelude::StateFlushSet,
    schedule::{
        schedule_apply_flush, schedule_bevy_state, schedule_detect_change, schedule_resolve_state,
        schedule_send_event, StateFlush, StateFlushEvent,
    },
    state::RawState,
};

pub struct PyriStatePlugin;

impl Plugin for PyriStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(StateFlush)
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);
    }
}

fn add_state_helper<S: ConfigureState>(app: &mut App, value: Option<S>) -> &mut App {
    if !app.world.contains_resource::<CurrentState<S>>() {
        app.init_resource::<CurrentState<S>>()
            .insert_resource(NextState_::new(value));
        S::configure(app);
    }
    app
}

pub trait AppExtPyriState {
    fn add_state_<S: ConfigureState>(&mut self) -> &mut Self;

    fn init_state_<S: ConfigureState + FromWorld>(&mut self) -> &mut Self;

    fn insert_state_<S: ConfigureState>(&mut self, value: S) -> &mut Self;
}

impl AppExtPyriState for App {
    fn add_state_<S: ConfigureState>(&mut self) -> &mut Self {
        add_state_helper::<S>(self, None)
    }

    fn init_state_<S: ConfigureState + FromWorld>(&mut self) -> &mut Self {
        let value = S::from_world(&mut self.world);
        add_state_helper(self, Some(value))
    }

    fn insert_state_<S: ConfigureState>(&mut self, value: S) -> &mut Self {
        add_state_helper(self, Some(value))
    }
}

pub trait ConfigureState: RawState {
    fn configure(app: &mut App);
}

pub struct ResolveStatePlugin<S: RawState> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: RawState> Plugin for ResolveStatePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_resolve_state::<S>(
            app.get_schedule_mut(StateFlush).unwrap(),
            &self.after,
            &self.before,
        );
    }
}

impl<S: RawState> Default for ResolveStatePlugin<S> {
    fn default() -> Self {
        Self {
            after: Vec::new(),
            before: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S: RawState> ResolveStatePlugin<S> {
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }

    pub fn after<T: RawState>(mut self) -> Self {
        self.after.push(StateFlushSet::<T>::Resolve.intern());
        self
    }

    pub fn before<T: RawState>(mut self) -> Self {
        self.before.push(StateFlushSet::<T>::Resolve.intern());
        self
    }
}

pub struct DetectChangePlugin<S: RawState + Eq>(PhantomData<S>);

impl<S: RawState + Eq> Plugin for DetectChangePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Eq> Default for DetectChangePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct FlushEventPlugin<S: RawState + Clone>(PhantomData<S>);

impl<S: RawState + Clone> Plugin for FlushEventPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_send_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone> Default for FlushEventPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct BevyStatePlugin<S: RawState + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

impl<S: RawState + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_state::<BevyState<S>>();
        schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct ApplyFlushPlugin<S: RawState + Clone>(PhantomData<S>);

impl<S: RawState + Clone> Plugin for ApplyFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone> Default for ApplyFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
