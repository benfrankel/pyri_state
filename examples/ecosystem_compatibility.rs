// Enable a `BevyState<S>` wrapper to interact with crates that expect it.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::{buffer::BevyState, prelude::*};

use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(PyriStatePlugin)
        .init_state_::<GameState>()
        .add_loading_state(
            LoadingState::new(BevyState(Some(GameState::LoadingGame)))
                .load_collection::<GameAssets>(),
        )
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
        );
}

#[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
// Enable the `bevy_state` plugin to set up `BevyState<GameState>`:
#[state(bevy_state)]
enum GameState {
    #[default]
    Splash,
    Title,
    LoadingGame,
    PlayingGame,
}
