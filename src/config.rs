use std::marker::PhantomData;

use bevy_app::App;
use bevy_ecs::schedule::InternedSystemSet;

use crate::{
    buffer::{CurrentState, NextState},
    prelude::{
        schedule_apply_flush, schedule_on_flush, schedule_send_event_on_flush,
        schedule_set_flush_on_change, PostStateFlush, PreStateFlush, State, StateFlush,
        StateFlushEvent,
    },
};

pub trait ConfigureState {
    fn configure(self, app: &mut App);
}

// TODO: Builder methods?
pub struct StateConfig<S: State> {
    // TODO: Support init FromWorld
    initial_state: Option<S>,
    flush_after: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: State> Default for StateConfig<S> {
    fn default() -> Self {
        Self {
            initial_state: None,
            flush_after: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S: State> ConfigureState for StateConfig<S> {
    fn configure(self, app: &mut App) {
        // TODO: Handle initial_state
        app.init_resource::<CurrentState<S>>()
            .init_resource::<NextState<S>>();

        schedule_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap(), &self.flush_after);
    }
}

struct StateConfigExtClone<S: State + Clone> {
    send_event_on_flush: bool,
    apply_flush: bool,
    _phantom: PhantomData<S>,
}

impl<S: State + Clone> ConfigureState for StateConfigExtClone<S> {
    fn configure(self, app: &mut App) {
        if self.send_event_on_flush {
            app.add_event::<StateFlushEvent<S>>();
            schedule_send_event_on_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }

        if self.apply_flush {
            schedule_apply_flush::<S>(app.get_schedule_mut(PostStateFlush).unwrap());
        }
    }
}

struct StateConfigExtEq<S: State + Eq> {
    set_flush_on_change: bool,
    _phantom: PhantomData<S>,
}

impl<S: State + Eq> ConfigureState for StateConfigExtEq<S> {
    fn configure(self, app: &mut App) {
        if self.set_flush_on_change {
            schedule_set_flush_on_change::<S>(app.get_schedule_mut(PreStateFlush).unwrap());
        }
    }
}

struct StateConfigClone<S: State + Clone> {
    common: StateConfig<S>,
    ext_clone: StateConfigExtClone<S>,
}

impl<S: State + Clone> ConfigureState for StateConfigClone<S> {
    fn configure(self, app: &mut App) {
        self.common.configure(app);
        self.ext_clone.configure(app);
    }
}

struct StateConfigEq<S: State + Eq> {
    common: StateConfig<S>,
    ext_eq: StateConfigExtEq<S>,
}

impl<S: State + Eq> ConfigureState for StateConfigEq<S> {
    fn configure(self, app: &mut App) {
        self.common.configure(app);
        self.ext_eq.configure(app);
    }
}

struct StateConfigCloneEq<S: State + Clone + Eq> {
    common: StateConfig<S>,
    ext_clone: StateConfigExtClone<S>,
    ext_eq: StateConfigExtEq<S>,
}

impl<S: State + Clone + Eq> ConfigureState for StateConfigCloneEq<S> {
    fn configure(self, app: &mut App) {
        self.common.configure(app);
        self.ext_clone.configure(app);
        self.ext_eq.configure(app);
    }
}
