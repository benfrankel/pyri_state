//! Disable or enable any state on command (e.g. for simple toggle states and substates).

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .add_state::<Paused>()
        .add_systems(StateFlush, Paused.on_edge(unpause, pause))
        .add_systems(
            Update,
            (
                Paused.enable().run_if(window_lost_focus),
                Paused::disable.run_if(window_gained_focus),
                Paused.toggle().run_if(input_just_pressed(KeyCode::Escape)),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug)]
#[state(log_flush)]
#[reflect(Resource)]
struct Paused;

// Dummy systems:
fn unpause() {}
fn pause() {}
fn window_lost_focus() -> bool {
    // Not implemented in this example.
    false
}
fn window_gained_focus() -> bool {
    // Not implemented in this example.
    false
}
