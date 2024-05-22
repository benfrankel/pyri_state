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
    use std::marker::PhantomData;

    use bevy_app::App;
    use bevy_ecs::{
        schedule::SystemSet,
        system::{Res, ResMut},
    };

    use crate::{
        config::{ConfigureState, StateConfigOnFlush},
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
            StateConfigOnFlush::<Self>(vec![], PhantomData)
        }
    }

    #[derive(Clone, PartialEq, Eq, Default)]
    struct PauseState(bool);

    impl State for PauseState {
        fn config() -> impl ConfigureState {
            StateConfigOnFlush::<Self>(vec![StateFlushSet::<GameState>::Resolve.intern()], PhantomData)
        }
    }

    fn unpause() {}

    fn pause() {}

    #[derive(Clone, PartialEq, Eq, Default)]
    struct LevelState {
        x: usize,
        y: usize,
    }

    impl State for LevelState {
        fn config() -> impl ConfigureState {
            StateConfigOnFlush::<Self>(
                vec![StateFlushSet::<GameState>::Resolve.intern()],
                PhantomData,
            )
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
            StateConfigOnFlush::<Self>(
                vec![StateFlushSet::<LevelState>::Resolve.intern()],
                PhantomData,
            )
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

        app.init_state_::<GameState>()
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
