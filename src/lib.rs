#[cfg(feature = "bevy_app")]
pub mod app;
pub mod conditions;
pub mod schedule;
pub mod state;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::*;

    #[doc(hidden)]
    pub use crate::{conditions::*, schedule::*, state::*};
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::schedule::IntoSystemConfigs;
    use pyrious_state_macros::State;

    use super::prelude::*;

    #[derive(State, Clone, PartialEq, Eq)]
    enum GameState {
        MainMenu,
        Playing,
        EndScreen,
    }

    // TODO: Ad hoc substate of GameState::Playing
    #[derive(State, Clone, PartialEq, Eq)]
    struct PauseState(bool);

    // TODO: Ad hoc substate of GameState::Playing
    #[derive(State, Clone, PartialEq, Eq)]
    struct LevelState {
        x: usize,
        y: usize,
    }

    // TODO: Ad hoc computed state from LevelState
    #[derive(State, Clone, PartialEq, Eq)]
    enum ColorState {
        Black,
        White,
    }

    fn do_stuff(level: &LevelState) {
        let _ = level;
    }

    fn tear_down_current_level(level: StateRef<LevelState>) {
        do_stuff(&level.get().unwrap());
    }

    fn set_up_next_level(level: StateRef<LevelState>) {
        do_stuff(&level.get_next().unwrap());
    }

    #[test]
    fn foo() {
        let mut app = App::new();

        app.add_state::<LevelState>().add_systems(
            StateTransition,
            (
                tear_down_current_level.in_set(OnTrans::<LevelState>::Exit),
                set_up_next_level.in_set(OnTrans::<LevelState>::Enter),
            ),
        );
    }
}
