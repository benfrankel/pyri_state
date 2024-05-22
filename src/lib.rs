// Allow derive macros in unit tests to use `pyri_state` to refer to this crate
extern crate self as pyri_state;

#[cfg(feature = "bevy_app")]
pub mod app;
pub mod buffer;
pub mod config;
pub mod schedule;
pub mod state;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::*;

    #[doc(hidden)]
    pub use crate::{buffer::*, schedule::*, state::*};

    #[doc(hidden)]
    pub use pyri_state_derive::State_;
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::system::{Res, ResMut};

    use crate::prelude::*;

    fn do_stuff_with<T>(x: T) {
        let _ = x;
    }

    #[derive(State_, Clone, PartialEq, Eq, Default)]
    enum GameState {
        #[default]
        MainMenu,
        Playing,
        EndScreen,
    }

    #[derive(State_, Clone, PartialEq, Eq, Default)]
    #[state(after(GameState))]
    struct PauseState(bool);

    fn unpause() {}

    fn pause() {}

    #[derive(State_, Clone, PartialEq, Eq, Default)]
    #[state(after(GameState))]
    struct LevelState {
        x: usize,
        y: usize,
    }

    fn exit_level(level: Res<CurrentState<LevelState>>) {
        let level_state = level.unwrap();
        do_stuff_with::<&LevelState>(level_state);
    }

    fn enter_level(level_state: Res<NextState_<LevelState>>) {
        let level_state = level_state.unwrap();
        do_stuff_with::<&LevelState>(level_state);
    }

    #[derive(State_, Clone, PartialEq, Eq)]
    #[state(after(LevelState))]
    enum ColorState {
        Black,
        White,
    }

    fn compute_color(
        level: Res<NextState_<LevelState>>,
        mut color: ResMut<NextState_<ColorState>>,
    ) {
        color.inner = level.get().map(|level| {
            if level.x + level.y % 2 == 0 {
                ColorState::Black
            } else {
                ColorState::White
            }
        });
    }

    fn exit_color(color_state: Res<CurrentState<ColorState>>) {
        let color_state = color_state.unwrap();
        do_stuff_with::<&ColorState>(color_state);
    }

    fn enter_color(color_state: Res<NextState_<ColorState>>) {
        let color_state = color_state.unwrap();
        do_stuff_with::<&ColorState>(color_state);
    }

    #[test]
    fn foo() {
        let mut app = App::new();

        app.add_plugins(StatePlugin)
            .init_state_::<GameState>()
            .add_state_::<PauseState>()
            .add_state_::<LevelState>()
            .add_state_::<ColorState>()
            .add_systems(
                StateFlush,
                (
                    GameState::Playing.on_exit((PauseState::remove, LevelState::remove)),
                    GameState::Playing.on_enter((PauseState::init, LevelState::init)),
                    PauseState(true).on_exit(unpause),
                    PauseState(true).on_enter(pause),
                    LevelState::on_any_exit(exit_level),
                    LevelState::on_any_enter((enter_level, compute_color)),
                    ColorState::on_any_exit(exit_color),
                    ColorState::on_any_enter(enter_color),
                ),
            );
    }
}
