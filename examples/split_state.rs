// A split state is a basic enum state that can be split between the modules of a crate.
// It's a useful organizational tool for cross-cutting states in a plugin-based codebase.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use input::InputMode;
use pyri_state::{prelude::*, state, storage::slot::StateSlot};

mod input {
    use pyri_state::extra::split::SplitState;
    use pyri_state_derive::State;

    // InputMode is defined as a split state in `mod input`.
    #[derive(State, Clone, PartialEq, Eq)]
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .insert_state_(StateSlot::enabled(InputMode::Move))
        .add_systems(
            Update,
            // Every state added to InputMode can be accessed normally.
            (
                state!(InputMode::Move | InputMode::Attack).on_update(
                    InputMode::Menu
                        .enter()
                        .run_if(input_just_pressed(KeyCode::Escape)),
                ),
                InputMode::Move.on_update((
                    move_left.run_if(input_just_pressed(KeyCode::KeyA)),
                    move_right.run_if(input_just_pressed(KeyCode::KeyD)),
                )),
                InputMode::Attack.on_update((
                    attack_left.run_if(input_just_pressed(KeyCode::KeyA)),
                    attack_right.run_if(input_just_pressed(KeyCode::KeyD)),
                )),
                InputMode::Menu.on_update((
                    menu_cancel.run_if(input_just_pressed(KeyCode::Escape)),
                    menu_confirm.run_if(input_just_pressed(KeyCode::Enter)),
                )),
            ),
        )
        .run();
}

// Dummy systems:
fn move_left() {}
fn move_right() {}
fn attack_left() {}
fn attack_right() {}
fn menu_cancel() {}
fn menu_confirm() {}
