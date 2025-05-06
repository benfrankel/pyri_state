//! Keep track of a state's history in a stack (e.g. for a back button).

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        // Add the `Menu` state with `Menu::Main` as the base of the stack.
        .insert_state(NextStateStack::with_base(Menu::Main))
        .init_state::<Screen>()
        .add_systems(
            Update,
            (
                // Open the in-game menu overlay on Escape press if there is no menu open.
                Screen::Gameplay.on_update(
                    Menu::MainOverlay
                        .push()
                        .run_if(Menu::is_disabled.and(input_just_pressed(KeyCode::Escape))),
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
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug)]
// Configure `Menu` to use `NextStateStack` instead of `NextStateBuffer` as its `NextState` type.
#[state(log_flush, next(NextStateStack<Self>))]
#[reflect(Resource)]
enum Menu {
    Main,
    MainOverlay,
    Settings,
    SettingsAudio,
    SettingsGraphics,
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Default)]
#[reflect(Resource)]
enum Screen {
    #[default]
    Title,
    Gameplay,
}
