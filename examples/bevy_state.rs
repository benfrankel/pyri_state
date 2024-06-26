// Enable a `BevyState<S>` wrapper to interact with ecosystem crates that expect it.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

use iyes_progress::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .init_state::<GameState>()
        .add_plugins(
            ProgressPlugin::new(BevyState(Some(GameState::LoadingGame)))
                // Changes to BevyState<GameState> will propagate to GameState.
                .continue_to(BevyState(Some(GameState::PlayingGame))),
        )
        .add_systems(
            Update,
            GameState::Title.on_update(
                // Changes to GameState will propagate to BevyState<GameState>.
                GameState::LoadingGame
                    .enter()
                    .run_if(input_just_pressed(KeyCode::Enter)),
            ),
        )
        .run();
}

#[derive(State, Component, Clone, PartialEq, Eq, Hash, Debug, Default)]
// Enable the `bevy_state` plugin to set up `BevyState<GameState>`:
#[state(bevy_state)]
enum GameState {
    #[default]
    Splash,
    Title,
    LoadingGame,
    PlayingGame,
}
