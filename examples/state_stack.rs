// A state stack is used to keep track of a state's "history". This can be used e.g.
// to easily implement a "go back" feature for UI.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use pyri_state::extra::stack::StateStack;
use pyri_state::{prelude::*, state};

#[derive(State, Clone, PartialEq, Eq, Default)]
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
        // (this adds both `StateStack<Menu>` and `Menu` itself as states)
        .init_state_::<StateStack<Menu>>()
        .init_state_::<GameState>()
        .add_systems(
            Update,
            (
                // Open the in-game menu overlay on Escape press if there is no menu open.
                GameState::PlayingGame.on_update(
                    StateStack::push(Menu::MainOverlay)
                        .run_if(Menu::is_disabled.and_then(input_just_pressed(KeyCode::Escape))),
                ),
                // Enter settings from main menu on S press.
                state!(Menu::Main | Menu::MainOverlay).on_update(
                    StateStack::push(Menu::Settings).run_if(input_just_pressed(KeyCode::KeyS)),
                ),
                // Enter settings sub-menus from settings on A or G press.
                Menu::Settings.on_update((
                    StateStack::push(Menu::SettingsAudio).run_if(input_just_pressed(KeyCode::KeyA)),
                    StateStack::push(Menu::SettingsGraphics)
                        .run_if(input_just_pressed(KeyCode::KeyG)),
                )),
                // Go back to the previous menu on Escape press.
                Menu::ANY
                    .on_update(StateStack::<Menu>::pop.run_if(input_just_pressed(KeyCode::Escape))),
            ),
        )
        .run();
}
