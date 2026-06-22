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
}

use bevy_ecs::schedule::{
    IntoScheduleConfigs as _, Schedule, SystemCondition, common_conditions::not,
};

use crate::{
    schedule::resolve_state::ResolveStateSystems,
    state::{State, StateExtEq as _},
};

/// Add change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`DetectChangePlugin<S>`].
pub fn schedule_detect_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(not(S::is_triggered).and_then(S::will_change))
            .in_set(ResolveStateSystems::<S>::Trigger),
    );
}
