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

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedule},
    system::{Res, ResMut},
};
use bevy_state::state::{NextState, States};

use crate::{
    schedule::StateHook,
    state::{NextStateMut, NextStateRef, State, StateMut},
};

/// A wrapper around the [`State`] type `S` for compatibility with the Bevy ecosystem.
///
/// Any change to `S` will propagate to `BevyState<S>`, and vice versa.
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State + Clone + PartialEq + Eq + Hash + Debug>(
    /// The wrapped state value, or `None` if disabled.
    pub Option<S>,
);

#[cfg(feature = "bevy_state")]
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
    let update_bevy_state =
        |pyri_state: NextStateRef<S>, mut bevy_state: ResMut<NextState<BevyState<S>>>| {
            if matches!(bevy_state.as_ref(), NextState::Unchanged) {
                bevy_state.set(BevyState(pyri_state.get().cloned()));
            }
        };

    let update_pyri_state = |mut pyri_state: NextStateMut<S>,
                             bevy_state: Res<NextState<BevyState<S>>>| {
        if let NextState::Pending(bevy_state) = bevy_state.as_ref() {
            pyri_state.trigger().set(bevy_state.0.clone());
        }
    };

    schedule.add_systems((
        update_pyri_state.in_set(StateHook::<S>::Compute),
        update_bevy_state.in_set(StateHook::<S>::Flush),
    ));
}

/// A plugin that adds [`BevyState<S>`] propagation systems for the [`State`] type `S`
/// to the [`StateFlush`](crate::schedule::StateFlush) schedule.
///
/// Calls [`schedule_bevy_state<S>`].
#[cfg(feature = "bevy_app")]
pub struct BevyStatePlugin<S: StateMut + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

#[cfg(feature = "bevy_app")]
impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> bevy_app::Plugin for BevyStatePlugin<S> {
    fn build(&self, app: &mut bevy_app::App) {
        bevy_state::app::AppExtStates::init_state::<BevyState<S>>(app);
        schedule_bevy_state::<S>(app.get_schedule_mut(crate::schedule::StateFlush).unwrap());
    }
}

#[cfg(feature = "bevy_app")]
impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
