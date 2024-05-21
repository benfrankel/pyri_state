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
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::{
        schedule::{IntoSystemConfigs, IntoSystemSetConfigs},
        system::{Res, ResMut},
    };
    use pyri_state_macros::State;

    use crate::{
        config::{ConfigureState, StateConfig},
        prelude::*,
    };

    fn do_stuff<T>(x: T) {
        let _ = x;
    }

    #[derive(Clone, PartialEq, Eq, Default)]
    enum GameState {
        #[default]
        MainMenu,
        Playing,
        EndScreen,
    }

    impl State for GameState {
        fn config() -> impl ConfigureState {
            StateConfig::<Self>::default()
        }
    }

    #[derive(Clone, PartialEq, Eq, Default)]
    struct PauseState(bool);

    impl State for PauseState {
        fn config() -> impl ConfigureState {
            // TODO: Specify that PauseState depends on GameState
            StateConfig::<Self>::default()
        }
    }

    fn apply_pause(pause_state: Res<NextState<PauseState>>) {
        let pause = pause_state.unwrap().0;
        do_stuff::<bool>(pause);
    }

    #[derive(Clone, PartialEq, Eq, Default)]
    struct LevelState {
        x: usize,
        y: usize,
    }

    impl State for LevelState {
        fn config() -> impl ConfigureState {
            // TODO: Specify that LevelState depends on GameState
            StateConfig::<Self>::default()
        }
    }

    fn exit_level(level: Res<CurrentState<LevelState>>) {
        let level_state = level.unwrap();
        do_stuff::<&LevelState>(level_state);
    }

    fn enter_level(level_state: Res<NextState<LevelState>>) {
        let level_state = level_state.unwrap();
        do_stuff::<&LevelState>(level_state);
    }

    #[derive(Clone)]
    enum ColorState {
        Black,
        White,
    }

    impl State for ColorState {
        fn config() -> impl ConfigureState {
            // TODO: Specify that ColorState depends on LevelState
            StateConfig::<Self>::default()
        }
    }

    fn compute_color(level: Res<NextState<LevelState>>, mut color: ResMut<NextState<ColorState>>) {
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
        do_stuff::<&ColorState>(color_state);
    }

    fn enter_color(color_state: Res<NextState<ColorState>>) {
        let color_state = color_state.unwrap();
        do_stuff::<&ColorState>(color_state);
    }

    #[test]
    fn foo() {
        let mut app = App::new();

        app.add_plugins(StatePlugin);

        // Set up GameState
        app.add_state::<GameState>()
            // TODO: Ordering dependencies should be configured via state settings
            .configure_sets(
                StateFlush,
                (
                    OnState::<GameState>::Flush,
                    (OnState::<LevelState>::Flush, OnState::<PauseState>::Flush),
                )
                    .chain(),
            )
            .add_systems(
                StateFlush,
                (
                    GameState::Playing.on_exit((LevelState::remove, PauseState::remove)),
                    GameState::Playing.on_enter((LevelState::init, PauseState::init)),
                ),
            );

        // Set up PauseState
        app.add_state::<PauseState>()
            .add_systems(StateFlush, PauseState::on_any_transition(apply_pause));

        // Set up LevelState
        app.add_state::<LevelState>()
            // TODO: Ordering dependencies should be configured via state settings
            .configure_sets(
                StateFlush,
                (OnState::<LevelState>::Flush, OnState::<ColorState>::Flush).chain(),
            )
            .add_systems(
                StateFlush,
                (
                    LevelState::on_any_exit(exit_level),
                    LevelState::on_any_enter(enter_level),
                    LevelState::on_any_change(compute_color),
                ),
            );

        // Set up ColorState
        app.add_state::<ColorState>().add_systems(
            StateFlush,
            (
                ColorState::on_any_exit(exit_color),
                ColorState::on_any_enter(enter_color),
            ),
        );
    }
}
