// Keep track of a state's history (e.g. for a UI "go back" feature).

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use pyri_state::{prelude::*, state, storage::stack::*};

#[derive(State, Clone, PartialEq, Eq, Default)]
// Configure Menu to use StateStack instead of StateSlot as its storage.
#[state(storage(StateStack<Self>))]
enum Menu {
    #[default]
    Main,
    MainOverlay,
    Settings,
    SettingsAudio,
    SettingsGraphics,
}

#[derive(State, Clone, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Title,
    PlayingGame,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        // Add Menu as a state stack, starting with `vec![Menu::MainMenu]`.
        .init_state_::<Menu>()
        .init_state_::<GameState>()
        .add_systems(
            Update,
            (
                // Open the in-game menu overlay on Escape press if there is no menu open.
                GameState::PlayingGame.on_update(
                    Menu::MainOverlay
                        .push()
                        .run_if(Menu::is_disabled.and_then(input_just_pressed(KeyCode::Escape))),
                ),
                // Enter settings from main menu on S press.
                state!(Menu::Main | Menu::MainOverlay).on_update(
                    Menu::Settings
                        .push()
                        .run_if(input_just_pressed(KeyCode::KeyS)),
                ),
                // Enter settings sub-menus from settings on A or G press.
                Menu::Settings.on_update((
                    Menu::SettingsAudio
                        .push()
                        .run_if(input_just_pressed(KeyCode::KeyA)),
                    Menu::SettingsGraphics
                        .push()
                        .run_if(input_just_pressed(KeyCode::KeyG)),
                )),
                // Go back to the previous menu on Escape press.
                Menu::ANY.on_update(Menu::pop.run_if(input_just_pressed(KeyCode::Escape))),
            ),
        )
        .run();
}