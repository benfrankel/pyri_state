use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{common_conditions::not, Condition, IntoSystemConfigs, IntoSystemSetConfigs},
    system::ResMut,
    world::FromWorld,
};

use crate::{
    conditions::{
        state_is_present, state_will_be_present, state_will_change, state_will_flush,
        state_will_remain_present,
    },
    schedule::{OnTrans, PostStateTransition, PreStateTransition, StateTransition},
    state::{CurrentState, NextState, State},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PreStateTransition)
            .init_schedule(StateTransition)
            .init_schedule(PostStateTransition);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, PreStateTransition);
        order.insert_after(PreStateTransition, StateTransition);
        order.insert_after(StateTransition, PostStateTransition);
    }
}

fn flush<S: State>(mut next: ResMut<NextState<S>>) {
    next.flush = true;
}

fn apply_flush<S: State>(mut current: ResMut<CurrentState<S>>, mut next: ResMut<NextState<S>>) {
    current.value = next.value.clone();
    next.flush = false;
}

fn set_up_systems<S: State>(app: &mut App) -> &mut App {
    app.add_systems(
        PreStateTransition,
        // TODO: Make this opt-out via settings
        flush::<S>.run_if(state_will_change::<S>.or_else(not(state_will_remain_present::<S>))),
    )
    .configure_sets(
        StateTransition,
        (
            OnTrans::<S>::Any.run_if(state_will_flush::<S>),
            (
                OnTrans::<S>::Exit.run_if(state_is_present::<S>),
                OnTrans::<S>::Enter.run_if(state_will_be_present::<S>),
            )
                .chain()
                .in_set(OnTrans::<S>::Any),
        ),
    )
    .add_systems(
        PostStateTransition,
        apply_flush::<S>.run_if(state_will_flush::<S>),
    )
}

// TODO: add_state_with_settings, init_state_with_settings, insert_state_with_settings
pub trait AppStateExt {
    fn add_state<S: State>(&mut self) -> &mut Self;

    fn init_state<S: State + FromWorld>(&mut self) -> &mut Self;

    fn insert_state<S: State>(&mut self, value: S) -> &mut Self;
}

impl AppStateExt for App {
    fn add_state<S: State>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        set_up_systems::<S>(self)
            .init_resource::<CurrentState<S>>()
            .init_resource::<NextState<S>>()
    }

    fn init_state<S: State + FromWorld>(&mut self) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        let value = S::from_world(&mut self.world);

        set_up_systems::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }

    fn insert_state<S: State>(&mut self, value: S) -> &mut Self {
        if self.world.contains_resource::<CurrentState<S>>() {
            return self;
        }

        set_up_systems::<S>(self)
            .init_resource::<CurrentState<S>>()
            .insert_resource(NextState::new(value))
    }
}
