#[cfg(feature = "bevy_app")]
pub mod app;
pub mod schedule;
pub mod state;
pub mod systems;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::*;

    #[doc(hidden)]
    pub use crate::{schedule::*, state::*};
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::{
        schedule::IntoSystemConfigs,
        system::{Res, ResMut},
    };
    use pyrious_state_macros::State;

    use crate::{
        prelude::*,
        systems::{init_state, remove_state, state_will_enter, state_will_exit, state_will_mutate},
    };

    fn do_stuff<T>(x: T) {
        let _ = x;
    }

    #[derive(State, Clone, PartialEq, Eq, Default)]
    enum GameState {
        #[default]
        MainMenu,
        Playing,
        EndScreen,
    }

    // TODO: Specify that PauseState depends on GameState
    #[derive(State, Clone, PartialEq, Eq, Default)]
    struct PauseState(bool);

    fn apply_pause(pause_state: Res<CurrentState<PauseState>>) {
        let pause = pause_state.unwrap().0;
        do_stuff::<bool>(pause);
    }

    // TODO: Specify that LevelState depends on GameState
    #[derive(State, Clone, PartialEq, Eq, Default)]
    struct LevelState {
        x: usize,
        y: usize,
    }

    fn exit_level(level: Res<CurrentState<LevelState>>) {
        let level_state = level.unwrap();
        do_stuff::<&LevelState>(level_state);
    }

    fn enter_level(level_state: Res<NextState<LevelState>>) {
        let level_state = level_state.unwrap();
        do_stuff::<&LevelState>(level_state);
    }

    // TODO: Specify that ColorState depends on LevelState
    #[derive(State, Clone, PartialEq, Eq)]
    enum ColorState {
        Black,
        White,
    }

    fn compute_color(
        level: Res<CurrentState<LevelState>>,
        mut color: ResMut<NextState<ColorState>>,
    ) {
        color.inner = level.inner.as_ref().map(|level| {
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

        app.add_plugins(StatePlugin)
            .init_state_ext::<GameState>()
            .add_state::<PauseState>()
            .add_state::<LevelState>()
            .add_state::<ColorState>()
            .add_systems(
                StateTransition,
                (
                    // GameState
                    (
                        (init_state::<LevelState>, init_state::<PauseState>)
                            .run_if(state_will_enter(GameState::Playing)),
                        (remove_state::<LevelState>, remove_state::<PauseState>)
                            .run_if(state_will_exit(GameState::Playing)),
                    )
                        .in_set(OnTrans::<GameState>::Apply),
                    // PauseState
                    apply_pause.in_set(OnTrans::<PauseState>::Apply),
                    // LevelState
                    exit_level.in_set(OnTrans::<LevelState>::Exit),
                    enter_level.in_set(OnTrans::<LevelState>::Enter),
                    compute_color
                        .run_if(state_will_mutate::<LevelState>)
                        .in_set(OnTrans::<LevelState>::Apply),
                    // ColorState
                    exit_color.in_set(OnTrans::<ColorState>::Exit),
                    enter_color.in_set(OnTrans::<ColorState>::Enter),
                ),
            );
    }
}
