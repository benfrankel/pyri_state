//! Compute states from anything in the ECS world, including other states.

use bevy::prelude::*;
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .init_state::<Screen>()
        .add_state::<CheckerboardSquare>()
        .add_state::<SquareColor>()
        .add_systems(
            StateFlush,
            (
                // Enable `CheckerboardSquare` only during `Screen::Gameplay`.
                Screen::Gameplay.on_edge(
                    CheckerboardSquare::disable,
                    CheckerboardSquare::enable_default,
                ),
                // Compute `SquareColor` from `CheckerboardSquare`.
                CheckerboardSquare::ANY.on_enter(compute_square_color),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Default)]
#[reflect(Resource)]
enum Screen {
    #[default]
    Splash,
    Gameplay,
}

// Substate of `Screen::Gameplay`
#[derive(State, Reflect, Clone, PartialEq, Eq, Default)]
#[state(after(Screen))]
#[reflect(Resource)]
struct CheckerboardSquare {
    row: u8,
    col: u8,
}

// Computed from `CheckerboardSquare`
#[derive(State, Reflect, Clone, PartialEq, Eq)]
#[state(after(CheckerboardSquare))]
#[reflect(Resource)]
enum SquareColor {
    Black,
    White,
}

fn compute_square_color(board: NextRef<CheckerboardSquare>, mut color: NextMut<SquareColor>) {
    color.set(board.get().map(|board| {
        if board.row + board.col % 2 == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    }));
}
