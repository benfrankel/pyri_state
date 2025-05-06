//! Directly update the next state instead of setting an entirely new value.

use core::{fmt::Debug, time::Duration};

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, time::common_conditions::on_timer,
};
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<ColorMode>()
        .add_systems(
            Update,
            // These systems might run on the same frame sometimes.
            // With partial mutation, that's totally fine and expected.
            ColorMode::ANY.on_update((
                // Toggle red on damage events.
                (
                    disable_red.run_if(took_damage),
                    enable_red.run_if(dealt_damage),
                )
                    .chain(),
                // Toggle green on Space press.
                toggle_green.run_if(input_just_pressed(KeyCode::Space)),
                // Toggle blue every 5 seconds.
                toggle_blue.run_if(on_timer(Duration::from_secs(5))),
            )),
        )
        .run()
}

// The player has different abilities depending on the color mode.
// Yellow mode is its own thing, for example; not just red and green at the same time.
#[derive(State, Reflect, Clone, PartialEq, Eq, Default)]
#[state(log_flush)]
#[reflect(Resource)]
struct ColorMode {
    red: bool,
    green: bool,
    blue: bool,
}

impl Debug for ColorMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match (self.red as u8, self.green as u8, self.blue as u8) {
            (0, 0, 0) => "Black",
            (0, 0, 1) => "Blue",
            (0, 1, 0) => "Green",
            (0, 1, 1) => "Cyan",
            (1, 0, 0) => "Red",
            (1, 0, 1) => "Magenta",
            (1, 1, 0) => "Yellow",
            (1, 1, 1) => "White",
            _ => unreachable!(),
        })
    }
}

fn enable_red(mut color: NextMut<ColorMode>) {
    color.unwrap_mut().red = true;
}

fn disable_red(mut color: NextMut<ColorMode>) {
    color.unwrap_mut().red = false;
}

fn toggle_green(mut color: NextMut<ColorMode>) {
    let color = color.unwrap_mut();
    color.green = !color.green;
}

fn toggle_blue(mut color: NextMut<ColorMode>) {
    let color = color.unwrap_mut();
    color.blue = !color.blue;
}

fn took_damage() -> bool {
    // Not implemented in this example.
    false
}

fn dealt_damage() -> bool {
    // Not implemented in this example.
    false
}
