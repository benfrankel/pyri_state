//! A [`BevyState`] wrapper for ecosystem compatibility.
//!
//! Enable the `bevy_state` feature flag to use this module.
//!
//! # Example
//!
//! Opt in to [`BevyStatePlugin<S>`] for `GameState`:
//!
//! ```rust
//! #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
//! #[state(bevy_state)]
//! enum GameState {
//!     #[default]
//!     Title,
//!     Loading,
//!     Playing,
//! }
//! ```
//!
//! Add `GameState` along with its [`BevyState`] wrapper:
//!
//! ```rust
//! app.init_state::<GameState>();
//! ```
//!
//! Change `GameState` to drive `BevyState`:
//!
//! ```rust
//! app.add_systems(Update, GameState::Title.on_update(
//!     GameState::Loading.enter().run_if(input_just_pressed(KeyCode::Enter)),
//! ));
//! ```
//!
//! Change `BevyState` to drive `GameState` (e.g. using
//! [iyes_progress](https://github.com/IyesGames/iyes_progress)):
//!
//! ```rust
//! app.add_plugins(
//!     ProgressPlugin::new(BevyState(Some(GameState::Loading)))
//!         .continue_to(BevyState(Some(GameState::Playing))),
//! );
//! ```

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use std::{fmt::Debug, hash::Hash, marker::PhantomData};

    use bevy_app::{App, Plugin, PreUpdate};
    use bevy_state::prelude as bevy;

    use crate::state::StateMut;

    use super::{schedule_bevy_state, BevyState};

    /// A plugin that adds [`BevyState<S>`] propagation systems for the
    /// [`State`](crate::state::State) type `S` to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_bevy_state<S>`].
    pub struct BevyStatePlugin<S: StateMut + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

    impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
        fn build(&self, app: &mut App) {
            bevy::AppExtStates::init_state::<BevyState<S>>(app);
            schedule_bevy_state::<S>(app.get_schedule_mut(PreUpdate).unwrap());
        }
    }

    impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use std::{fmt::Debug, hash::Hash};

use bevy_ecs::{schedule::Schedule, system::ResMut};
use bevy_state::prelude as bevy;

use crate::{
    access::NextMut,
    state::{State, StateMut},
};

/// A wrapper around the [`State`] type `S` for compatibility with the Bevy ecosystem.
///
/// Any change to `S` will propagate to `BevyState<S>`, and vice versa.
#[derive(bevy::States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State + Clone + PartialEq + Eq + Hash + Debug>(
    /// The wrapped state value, or `None` if disabled.
    pub Option<S>,
);

impl<S: State + Clone + PartialEq + Eq + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}

/// Add [`BevyState<S>`] propagation systems for the [`State`] type `S` to a schedule.
///
/// Used in [`BevyStatePlugin<S>`].
pub fn schedule_bevy_state<S: State + StateMut + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let sync_states =
        |mut pyri_state: NextMut<S>, mut bevy_state: ResMut<bevy::NextState<BevyState<S>>>| {
            match bevy_state.as_ref() {
                bevy::NextState::Pending(bevy_state) => {
                    pyri_state.trigger().set(bevy_state.0.clone());
                }
                bevy::NextState::Unchanged => {
                    bevy_state.set(BevyState(pyri_state.get().cloned()));
                }
            }
        };

    schedule.add_systems(sync_states);
}
