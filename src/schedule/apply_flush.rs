//! Apply state flush by cloning the next state into the current state.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use core::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that adds an apply flush system for the [`State`] type `S`
    /// to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_apply_flush<S>`].
    pub struct ApplyFlushPlugin<S: State + Clone>(PhantomData<S>);

    impl<S: State + Clone> Plugin for ApplyFlushPlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State + Clone> Default for ApplyFlushPlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use core::{fmt::Debug, hash::Hash};

use bevy_ecs::{
    schedule::{IntoScheduleConfigs as _, Schedule, SystemSet},
    system::Commands,
};

use crate::{
    access::{CurrentMut, NextRef},
    state::State,
};

/// A system set that applies all triggered [`State`] flushes at the end of
/// the [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(SystemSet, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ApplyFlushSystems;

fn apply_flush<S: State + Clone>(
    mut commands: Commands,
    mut current: CurrentMut<S>,
    next: NextRef<S>,
) {
    match (current.get_mut(), next.get()) {
        (Some(x), Some(y)) => *x = y.clone(),
        (Some(_), None) => {
            commands.remove_resource::<S>();
        }
        (None, Some(y)) => {
            commands.insert_resource(y.clone());
        }
        _ => (),
    }
}

/// Add an apply flush system for the [`State`] type `S` to a schedule.
///
/// Used in [`ApplyFlushPlugin<S>`].
pub fn schedule_apply_flush<S: State + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (apply_flush::<S>, S::reset_trigger)
            .run_if(S::is_triggered)
            .in_set(ApplyFlushSystems),
    );
}
