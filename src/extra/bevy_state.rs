//! A [`BevyState`] wrapper for ecosystem compatibility.
//!
//! Enable the `bevy_state` feature flag to use this module.
//!
//! # Example
//!
//! Opt in to the [`BevyStatePlugin`] for `Screen` by adding `#[state(bevy_state)]`:
//!
//! ```
//! # use bevy::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
//! #[state(bevy_state)]
//! enum Screen {
//!     #[default]
//!     Title,
//!     Loading,
//!     Gameplay,
//! }
//! ```
//!
//! Add the `Screen` state type to your app:
//!
//! ```
//! # use bevy::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
//! # #[state(bevy_state)]
//! # enum Screen {
//! #     #[default]
//! #     Title,
//! #     Loading,
//! #     Gameplay,
//! # }
//! #
//! # fn plugin(app: &mut App) {
//! app.init_state::<Screen>();
//! # }
//! ```
//!
//! Make changes to `Screen` to drive `BevyState<Screen>`:
//!
//! ```
//! # use bevy::{input::common_conditions::input_just_pressed, prelude::*};
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
//! # #[state(bevy_state)]
//! # enum Screen {
//! #     #[default]
//! #     Title,
//! #     Loading,
//! #     Gameplay,
//! # }
//! #
//! # fn plugin(app: &mut App) {
//! app.add_systems(Update, Screen::Title.on_update(
//!     Screen::Loading.enter().run_if(input_just_pressed(KeyCode::Enter)),
//! ));
//! # }
//! ```
//!
//! Make changes to `BevyState<Screen>` to drive `Screen` (e.g. with
//! [iyes_progress](https://github.com/IyesGames/iyes_progress)):
//!
//! ```
//! # use bevy::prelude::*;
//! # use iyes_progress::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
//! # #[state(bevy_state)]
//! # enum Screen {
//! #     #[default]
//! #     Title,
//! #     Loading,
//! #     Gameplay,
//! # }
//! #
//! # fn plugin(app: &mut App) {
//! app.add_plugins(
//!     ProgressPlugin::new()
//!         .with_state_transition(Screen::Loading.bevy(), Screen::Gameplay.bevy()),
//! );
//! # }
//! ```

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use core::marker::PhantomData;

    use bevy_app::{App, Plugin};

    use crate::schedule::StateFlush;

    use super::*;

    /// A plugin that adds [`BevyState<S>`] propagation systems for the
    /// [`State`] type `S` to the [`StateFlush`] schedule.
    ///
    /// Calls [`schedule_bevy_state<S>`].
    pub struct BevyStatePlugin<S: StateMut + Clone + PartialEq + Eq + Hash + Debug>(PhantomData<S>);

    impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
        fn build(&self, app: &mut App) {
            bevy::AppExtStates::init_state::<BevyState<S>>(app);
            schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
        }
    }

    impl<S: StateMut + Clone + PartialEq + Eq + Hash + Debug> Default for BevyStatePlugin<S> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

use core::{fmt::Debug, hash::Hash};

use bevy_ecs::{
    schedule::{IntoScheduleConfigs as _, Schedule},
    system::{Res, ResMut},
};
use bevy_state::prelude as bevy;

use crate::{
    access::{NextMut, NextRef},
    schedule::ResolveStateSystems,
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

impl<S: State + Clone + PartialEq + Eq + Hash + Debug> From<S> for BevyState<S> {
    fn from(value: S) -> Self {
        Self(Some(value))
    }
}

/// An extension trait for [`State`] types that provides conversion to [`BevyState`].
pub trait StateExtBevy: State + Clone + PartialEq + Eq + Hash + Debug {
    /// Convert into a [`BevyState`].
    fn bevy(self) -> BevyState<Self>;
}

impl<S: State + Clone + PartialEq + Eq + Hash + Debug> StateExtBevy for S {
    fn bevy(self) -> BevyState<Self> {
        self.into()
    }
}

/// Add [`BevyState<S>`] propagation systems for the [`State`] type `S` to a schedule.
///
/// Used in [`BevyStatePlugin<S>`].
pub fn schedule_bevy_state<S: State + StateMut + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let sync_pyri_state =
        |mut pyri_state: NextMut<S>, bevy_state: Res<bevy::NextState<BevyState<S>>>| {
            if let bevy::NextState::Pending(bevy_state) = bevy_state.as_ref() {
                pyri_state.trigger().set(bevy_state.0.clone());
            }
        };

    let sync_bevy_state =
        |pyri_state: NextRef<S>, mut bevy_state: ResMut<bevy::NextState<BevyState<S>>>| {
            bevy_state.set(BevyState(pyri_state.get().cloned()));
        };

    schedule.add_systems((
        sync_pyri_state.in_set(ResolveStateSystems::<S>::Compute),
        sync_bevy_state.in_set(ResolveStateSystems::<S>::AnyFlush),
    ));
}
