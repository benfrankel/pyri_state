//! Enable a [`BevyState<S>`] wrapper to interact with ecosystem crates that expect it.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

use iyes_progress::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<Screen>()
        .add_plugins(
            ProgressPlugin::new()
                // Changes to `BevyState<Screen>` will propagate to `Screen`.
                .with_state_transition(Screen::Loading.bevy(), Screen::Gameplay.bevy()),
        )
        .add_systems(
            Update,
            Screen::Title.on_update(
                // Changes to `Screen` will propagate to `BevyState<Screen>`.
                Screen::Loading
                    .enter()
                    .run_if(input_just_pressed(KeyCode::Enter)),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Hash, Debug, Default)]
// Enable the `bevy_state` plugin to set up `BevyState<Screen>`:
#[state(bevy_state, log_flush)]
#[reflect(Resource)]
enum Screen {
    #[default]
    Title,
    Loading,
    Gameplay,
}
