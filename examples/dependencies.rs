// Compute states from anything in the ECS world, including other states.

use bevy::prelude::*;
use pyri_state::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .init_state::<GameState>()
        .add_state::<CheckerboardSquare>()
        .add_state::<SquareColor>()
        .add_systems(
            StateFlush,
            (
                // Enable CheckerboardSquare only during GameState::Playing.
                GameState::Playing.on_edge(
                    CheckerboardSquare::disable,
                    CheckerboardSquare::enable_default,
                ),
                // Compute SquareColor from CheckerboardSquare.
                CheckerboardSquare::ANY.on_enter(compute_square_color),
            ),
        )
        .run();
}

#[derive(State, Clone, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Splash,
    Playing,
}

// Substate of GameState::Playing
#[derive(State, Clone, PartialEq, Eq, Default)]
#[state(after(GameState))]
struct CheckerboardSquare {
    row: u8,
    col: u8,
}

// Computed from CheckerboardSquare
#[derive(State, Clone, PartialEq, Eq)]
#[state(after(CheckerboardSquare))]
enum SquareColor {
    Black,
    White,
}

fn compute_square_color(
    board: NextRef<CheckerboardSquare>,
    mut color: NextMut<SquareColor>,
) {
    color.set(board.get().map(|board| {
        if board.row + board.col % 2 == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    }));
}
