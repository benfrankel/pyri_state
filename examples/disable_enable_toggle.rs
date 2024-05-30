// Disable or enable any state on command (great for toggle states and substates).

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .add_state_::<Paused>()
        .add_systems(
            StateFlush,
            (Paused.on_exit(unpause), Paused.on_enter(pause)),
        )
        .add_systems(
            Update,
            (
                Paused.enable().run_if(window_lost_focus),
                Paused::disable.run_if(window_gained_focus),
                Paused.toggle().run_if(input_just_pressed(KeyCode::Escape)),
            ),
        )
        .run();
}

#[derive(State, Clone, PartialEq, Eq)]
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
