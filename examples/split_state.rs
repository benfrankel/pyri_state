// A split state is a basic enum state that can be split between the modules of a crate.
// It's a useful organizational tool for cross-cutting states in a plugin-based codebase.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use input::InputMode;
use pyri_state::{debug::StateDebugSettings, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .insert_state(StateBuffer::enabled(InputMode::Move))
        .add_systems(
            Update,
            // Every state added to InputMode can be accessed normally.
            (
                // While in `Move` mode, move with A/D or enter `Attack` mode with W.
                InputMode::Move.on_update((
                    move_left.run_if(input_just_pressed(KeyCode::KeyA)),
                    move_right.run_if(input_just_pressed(KeyCode::KeyD)),
                    InputMode::Attack
                        .enter()
                        .run_if(input_just_pressed(KeyCode::KeyW)),
                )),
                // While in `Attack` mode, attack with A/D and return to `Move` mode.
                InputMode::Attack.on_update((
                    (attack_left, InputMode::Move.enter())
                        .run_if(input_just_pressed(KeyCode::KeyA)),
                    (attack_right, InputMode::Move.enter())
                        .run_if(input_just_pressed(KeyCode::KeyD)),
                )),
                // Enter `Menu` mode on Escape press.
                InputMode::with(|x| x != &InputMode::Menu).on_update(
                    InputMode::Menu
                        .enter()
                        .run_if(input_just_pressed(KeyCode::Escape)),
                ),
                // While in `Menu` mode, cancel / confirm with Escape / Enter.
                InputMode::Menu.on_update((
                    (menu_cancel, InputMode::Move.enter())
                        .run_if(input_just_pressed(KeyCode::Escape)),
                    menu_confirm.run_if(input_just_pressed(KeyCode::Enter)),
                )),
            ),
        )
        .run();
}

mod input {
    use pyri_state::extra::split::SplitState;
    use pyri_state_derive::State;

    // InputMode is defined as a split state in `mod input`.
    #[derive(State, Clone, PartialEq, Eq, Debug)]
    #[state(log_flush)]
    pub struct InputMode(pub SplitState);
}

mod game {
    use super::input::InputMode;
    use pyri_state::add_to_split_state;

    // The Move and Attack states are added to InputMode in `mod game`.
    add_to_split_state!(InputMode, Move, Attack);
}

mod ui {
    use super::input::InputMode;
    use pyri_state::add_to_split_state;

    // The Menu state is added to InputMode in `mod ui`.
    add_to_split_state!(InputMode, Menu);
}

fn move_left() {
    info!("Moved left.");
}

fn move_right() {
    info!("Moved right.");
}

fn attack_left() {
    info!("Attacked left.");
}

fn attack_right() {
    info!("Attacked right.");
}

fn menu_cancel() {
    info!("Canceled menu.");
}

fn menu_confirm() {
    info!("Confirmed menu.");
}
