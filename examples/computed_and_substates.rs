// Roll your own computed and substates with the full power of bevy ECS.

use bevy::prelude::*;
use pyri_state::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(PyriStatePlugin)
        .init_state_::<GameState>()
        .add_state_::<CheckerboardSquare>()
        .add_state_::<SquareColor>()
        .add_systems(
            StateFlush,
            (
                // Enable CheckerboardSquare only during GameState::Playing.
                GameState::Playing.on_exit(CheckerboardSquare::disable),
                GameState::Playing.on_enter(CheckerboardSquare::enable),
                // Compute SquareColor from CheckerboardSquare.
                CheckerboardSquare::ANY.on_enter(compute_square_color),
            ),
        );
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
    board: Res<NextState_<CheckerboardSquare>>,
    mut color: ResMut<NextState_<SquareColor>>,
) {
    color.inner = board.get().map(|board| {
        if board.row + board.col % 2 == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    });
}
