//! Trigger a flush when the next state differs from the current state.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use core::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that adds a change detection system for the [`State`] type `S`
    /// to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_detect_change<S>`].
    pub struct DetectChangePlugin<S: State + Eq>(PhantomData<S>);

    impl<S: State + Eq> Plugin for DetectChangePlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: State + Eq> Default for DetectChangePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    /// A plugin that adds a local change detection system for the [`State`] type `S`
    /// to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_local_detect_change<S>`].
    pub struct LocalDetectChangePlugin<S: LocalState + Eq>(PhantomData<S>);

    impl<S: LocalState + Eq> Plugin for LocalDetectChangePlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_local_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: LocalState + Eq> Default for LocalDetectChangePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use bevy_ecs::{
    schedule::{Condition, IntoScheduleConfigs as _, Schedule, common_conditions::not},
    system::{Query, StaticSystemParam},
};

use crate::{
    next_state::{NextState, TriggerStateFlush},
    schedule::resolve_state::ResolveStateSystems,
    state::{LocalState, State, StateExtEq as _},
};

/// Add change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`DetectChangePlugin<S>`].
pub fn schedule_detect_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(not(S::is_triggered).and(S::will_change))
            .in_set(ResolveStateSystems::<S>::Trigger),
    );
}

fn local_detect_change<S: LocalState + Eq>(
    next_param: StaticSystemParam<<S::Next as NextState>::Param>,
    mut state_query: Query<(Option<&S>, &S::Next, &mut TriggerStateFlush<S>)>,
) {
    for (current, next, mut trigger) in &mut state_query {
        if !trigger.0 && current != next.next_state(&next_param) {
            trigger.0 = true;
        }
    }
}

/// Add local change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`LocalDetectChangePlugin<S>`].
pub fn schedule_local_detect_change<S: LocalState + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(local_detect_change::<S>.in_set(ResolveStateSystems::<S>::Trigger));
}
