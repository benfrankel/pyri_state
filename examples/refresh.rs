// Trigger a transition from the current state to itself (e.g. to restart the current level).

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .init_state_::<Level>()
        .add_systems(
            StateFlush,
            (
                Level::ANY.on_exit(tear_down_old_level),
                Level::ANY.on_enter(set_up_new_level),
            ),
        )
        .add_systems(
            Update,
            // Restart the current level on R press:
            Level::refresh.run_if(input_just_pressed(KeyCode::KeyR)),
        )
        .run();
}

#[derive(State, Clone, PartialEq, Eq, Default)]
struct Level(usize);

// Dummy systems:
fn tear_down_old_level(_level: Res<CurrentState<Level>>) {}
fn set_up_new_level(_level: NextStateRef<Level>) {}
