// Disable or enable any state type on command (great for simple on/off states or substates).

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

#[derive(State, Clone, PartialEq, Eq, Default)]
struct Paused;

// Systems:
fn unpause() {}
fn pause() {}
fn window_lost_focus() -> bool {
    unimplemented!()
}
fn window_gained_focus() -> bool {
    unimplemented!()
}

fn main() {
    let mut app = App::new();
    app.add_plugins(PyriStatePlugin)
        .add_state_::<Paused>()
        .add_systems(
            StateFlush,
            (Paused.on_exit(unpause), Paused.on_enter(pause)),
        )
        .add_systems(
            Update,
            (
                Paused::enable.run_if(window_lost_focus),
                Paused::disable.run_if(window_gained_focus),
                Paused::toggle.run_if(input_just_pressed(KeyCode::Escape)),
            ),
        );
}
