//! Trigger a flush when the next state differs from the current state.

use bevy_ecs::{
    schedule::{common_conditions::not, Condition, IntoSystemConfigs, Schedule},
    system::{Query, StaticSystemParam},
};

use crate::{
    next_state::{NextState, TriggerStateFlush},
    state::{LocalState, State, StateExtEq as _},
};

use super::resolve_state::StateHook;

/// Add change detection systems for the [`State`] type `S` to a schedule.
///
/// Used in [`DetectChangePlugin<S>`].
pub fn schedule_detect_change<S: State + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::trigger
            .run_if(not(S::is_triggered).and_then(S::will_change))
            .in_set(StateHook::<S>::Trigger),
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
    schedule.add_systems(local_detect_change::<S>.in_set(StateHook::<S>::Trigger));
}

/// A plugin that adds a change detection system for the [`State`] type `S`
/// to the [`StateFlush`](crate::schedule::StateFlush) schedule.
///
/// Calls [`schedule_detect_change<S>`].
#[cfg(feature = "bevy_state")]
pub struct DetectChangePlugin<S: State + Eq>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_state")]
impl<S: State + Eq> bevy_app::Plugin for DetectChangePlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        schedule_detect_change::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_state")]
impl<S: State + Eq> Default for DetectChangePlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// A plugin that adds a local change detection system for the [`State`] type `S`
/// to the [`StateFlush`](crate::schedule::StateFlush) schedule.
///
/// Calls [`schedule_local_detect_change<S>`].
#[cfg(feature = "bevy_state")]
pub struct LocalDetectChangePlugin<S: LocalState + Eq>(std::marker::PhantomData<S>);

#[cfg(feature = "bevy_state")]
impl<S: LocalState + Eq> bevy_app::Plugin for LocalDetectChangePlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        schedule_local_detect_change::<S>(
            app.get_schedule_mut(crate::schedule::StateFlush).unwrap(),
        );
    }
}

#[cfg(feature = "bevy_state")]
impl<S: LocalState + Eq> Default for LocalDetectChangePlugin<S> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}
