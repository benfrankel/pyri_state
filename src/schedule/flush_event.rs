//! Send a [`StateFlushEvent`] on state flush.

use bevy_ecs::{
    entity::Entity,
    event::{Event, EventWriter},
    schedule::{IntoSystemConfigs, Schedule},
    system::{Query, StaticSystemParam},
};

use crate::{
    access::FlushRef,
    next_state::{NextState, TriggerStateFlush},
    schedule::StateHook,
    state::{LocalState, State},
};

/// An event sent whenever the [`State`] type `S` flushes.
///
/// Added [by default](pyri_state_derive::State) by [`FlushEventPlugin<S>`].
#[derive(Event)]
pub struct StateFlushEvent<S: State> {
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
}

/// An event sent whenever a local [`State`] type `S` flushes.
///
/// Added [by default](pyri_state_derive::State) by [`LocalFlushEventPlugin<S>`].
#[derive(Event)]
pub struct LocalStateFlushEvent<S: LocalState> {
    /// The entity for which the state flush occurred.
    pub entity: Entity,
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
}

fn send_flush_event<S: State + Clone>(
    state: FlushRef<S>,
    mut events: EventWriter<StateFlushEvent<S>>,
) {
    let (old, new) = state.get();
    events.send(StateFlushEvent {
        old: old.cloned(),
        new: new.cloned(),
    });
}

/// Add a [`StateFlushEvent<S>`] sending system for the [`State`] type `S` to a schedule.
///
/// Used in [`FlushEventPlugin<S>`].
pub fn schedule_flush_event<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        send_flush_event::<S>
            .run_if(S::is_triggered)
            .in_set(StateHook::<S>::Flush),
    );
}

fn send_local_flush_event<S: LocalState + Clone>(
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, Option<&S>, &S::Next, &TriggerStateFlush<S>)>,
    mut events: EventWriter<LocalStateFlushEvent<S>>,
) {
    for (entity, current, next, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }

        events.send(LocalStateFlushEvent {
            entity,
            old: current.cloned(),
            new: next.next_state(&next_param).cloned(),
        });
    }
}

/// Add a local [`StateFlushEvent<S>`] sending system for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalFlushEventPlugin<S>`].
pub fn schedule_local_flush_event<S: LocalState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(send_local_flush_event::<S>.in_set(StateHook::<S>::Flush));
}

/// A plugin that adds a [`StateFlushEvent<S>`] sending system for the [`State`] type `S`
/// to the [`StateFlush`](crate::schedule::StateFlush) schedule.
///
/// Calls [`schedule_flush_event<S>`].
#[cfg(feature = "bevy_app")]
pub struct FlushEventPlugin<S: State + Clone>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: State + Clone> bevy_app::Plugin for FlushEventPlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_flush_event::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: State + Clone> Default for FlushEventPlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// A plugin that adds a [`LocalStateFlushEvent<S>`] sending system for the [`State`] type `S`
/// to the [`StateFlush`](crate::schedule::StateFlush) schedule.
///
/// Calls [`schedule_local_flush_event<S>`].
#[cfg(feature = "bevy_app")]
pub struct LocalFlushEventPlugin<S: State + Clone>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: LocalState + Clone> bevy_app::Plugin for LocalFlushEventPlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<LocalStateFlushEvent<S>>();
        schedule_local_flush_event::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: LocalState + Clone> Default for LocalFlushEventPlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
