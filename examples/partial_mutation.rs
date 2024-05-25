// Directly update the next state instead of setting an entirely new value.

use std::time::Duration;

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, time::common_conditions::on_timer,
};
use pyri_state::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .init_state_::<ColorMode>()
        .add_systems(
            Update,
            // These systems might run on the same frame sometimes.
            // With partial mutation, that's totally fine and expected.
            ColorMode::ANY.on_update((
                (
                    disable_red.run_if(took_damage),
                    enable_red.run_if(dealt_damage),
                )
                    .chain(),
                toggle_green.run_if(input_just_pressed(KeyCode::Space)),
                toggle_blue.run_if(on_timer(Duration::from_secs(5))),
            )),
        )
        .run();
}

// Player has different abilities depending on the color mode. For example,
// yellow mode is its own thing, not just red and green modes at the same time.
#[derive(State, Clone, PartialEq, Eq, Default)]
struct ColorMode {
    r: bool,
    g: bool,
    b: bool,
}

fn enable_red(mut color: ResMut<NextState_<ColorMode>>) {
    color.unwrap_mut().r = true;
}

fn disable_red(mut color: ResMut<NextState_<ColorMode>>) {
    color.unwrap_mut().r = false;
}

fn toggle_green(mut color: ResMut<NextState_<ColorMode>>) {
    let color = color.unwrap_mut();
    color.g = !color.g;
}

fn toggle_blue(mut color: ResMut<NextState_<ColorMode>>) {
    let color = color.unwrap_mut();
    color.b = !color.b;
}

fn took_damage() -> bool {
    unimplemented!()
}

fn dealt_damage() -> bool {
    unimplemented!()
}
