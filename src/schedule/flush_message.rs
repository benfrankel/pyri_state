//! Write a [`StateFlushMessage`] on state flush.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use core::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that adds a [`StateFlushMessage<S>`] writing system for the [`State`] type `S`
    /// to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_flush_message<S>`].
    pub struct FlushMessagePlugin<S: State + Clone>(PhantomData<S>);

    impl<S: State + Clone> Plugin for FlushMessagePlugin<S> {
        fn build(&self, app: &mut App) {
            app.add_message::<StateFlushMessage<S>>();
            schedule_flush_message::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State + Clone> Default for FlushMessagePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    /// A plugin that adds a [`LocalStateFlushMessage<S>`] writing system for the [`State`] type `S`
    /// to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_local_flush_message<S>`].
    pub struct LocalFlushMessagePlugin<S: State + Clone>(PhantomData<S>);

    impl<S: LocalState + Clone> Plugin for LocalFlushMessagePlugin<S> {
        fn build(&self, app: &mut App) {
            app.add_message::<LocalStateFlushMessage<S>>();
            schedule_local_flush_message::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: LocalState + Clone> Default for LocalFlushMessagePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use bevy_ecs::{
    entity::Entity,
    message::{Message, MessageWriter},
    schedule::{IntoScheduleConfigs as _, Schedule},
    system::{Query, StaticSystemParam},
};

use crate::{
    access::FlushRef,
    next_state::{NextState, TriggerStateFlush},
    schedule::ResolveStateSystems,
    state::{LocalState, State},
};

/// A message written whenever the [`State`] type `S` flushes.
///
/// Added [by default](pyri_state_derive::State) by [`FlushMessagePlugin<S>`].
#[derive(Message)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct StateFlushMessage<S: State> {
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
}

/// A message written whenever a local [`State`] type `S` flushes.
///
/// Added [by default](pyri_state_derive::State) by [`LocalFlushMessagePlugin<S>`].
#[derive(Message)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct LocalStateFlushMessage<S: LocalState> {
    /// The entity for which the state flush occurred.
    pub entity: Entity,
    /// The state before the flush, or `None` if disabled.
    pub old: Option<S>,
    /// The state after the flush, or `None` if disabled.
    pub new: Option<S>,
}

fn write_flush_message<S: State + Clone>(
    state: FlushRef<S>,
    mut message: MessageWriter<StateFlushMessage<S>>,
) {
    let (old, new) = state.get();
    message.write(StateFlushMessage {
        old: old.cloned(),
        new: new.cloned(),
    });
}

/// Add a [`StateFlushMessage<S>`] writing system for the [`State`] type `S` to a schedule.
///
/// Used in [`FlushMessagePlugin<S>`].
pub fn schedule_flush_message<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(write_flush_message::<S>.in_set(ResolveStateSystems::<S>::AnyFlush));
}

fn write_local_flush_message<S: LocalState + Clone>(
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    state_query: Query<(Entity, Option<&S>, &S::Next, &TriggerStateFlush<S>)>,
    mut messages: MessageWriter<LocalStateFlushMessage<S>>,
) {
    for (entity, current, next, trigger) in &state_query {
        if !trigger.0 {
            continue;
        }

        messages.write(LocalStateFlushMessage {
            entity,
            old: current.cloned(),
            new: next.next_state(&next_param).cloned(),
        });
    }
}

/// Add a local [`StateFlushMessage<S>`] writing system for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalFlushMessagePlugin<S>`].
pub fn schedule_local_flush_message<S: LocalState + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(write_local_flush_message::<S>.in_set(ResolveStateSystems::<S>::Flush));
}
