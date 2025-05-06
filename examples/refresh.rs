//! Trigger a transition from the current state to itself (e.g. to restart the current level).

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<Level>()
        .add_systems(
            StateFlush,
            Level::ANY.on_edge(tear_down_old_level, set_up_new_level),
        )
        .add_systems(
            Update,
            (
                // Restart the current level on up arrow press.
                Level::refresh.run_if(input_just_pressed(KeyCode::ArrowUp)),
                // Enter the previous level on left arrow press.
                go_to_prev_level.run_if(input_just_pressed(KeyCode::ArrowLeft)),
                // Enter the next level on right arrow press.
                go_to_next_level.run_if(input_just_pressed(KeyCode::ArrowRight)),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug, Default)]
#[state(log_flush)]
#[reflect(Resource)]
struct Level(isize);

impl Level {
    fn prev(&mut self) {
        self.0 -= 1;
    }

    fn next(&mut self) {
        self.0 += 1;
    }
}

fn go_to_prev_level(mut level: NextMut<Level>) {
    level.unwrap_mut().prev();
}

fn go_to_next_level(mut level: NextMut<Level>) {
    level.unwrap_mut().next();
}

// Dummy systems:
fn tear_down_old_level(_level: CurrentRef<Level>) {}
fn set_up_new_level(_level: NextRef<Level>) {}
