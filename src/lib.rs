//! `pyri_state` is a `bevy_state` alternative offering flexible change detection & scheduling.
//!
//! # Overview
//!
//! 1. The current state is stored in a [`CurrentState`](state::CurrentState) resource.
//! 2. The next state is stored in a [`StateBuffer`](storage::StateBuffer) resource by default
//! (see [storage] for more information).
//! 3. A state flush is triggered by the [`TriggerStateFlush`](state::TriggerStateFlush) resource
//! and handled in the [`StateFlush`](schedule::StateFlush) schedule.
//! 4. State flush hooks are organized into [`StateHook`](schedule::StateHook)
//! system sets.
//! 5. Tools are provided for [state configuration](extra::app), [debugging](extra::debug),
//! [pattern-matching](pattern), [and more](extra).
//!
//! # Getting started
//!
//! Import the [prelude] to bring traits and common types into scope:
//!
//! ```rust
//! // Note: Disable the `bevy_state` feature of `bevy` to avoid prelude interference.
//! //use bevy::prelude::*;
//! use pyri_state::prelude::*;
//! ```
//!
//! Define your own [`State`](state::State) type using the
//! [derive macro](pyri_state_derive::State):
//!
//! ```rust
//! #[derive(State, Clone, PartialEq, Eq, Default)]
//! struct Level(pub usize);
//! ```
//!
//! Add [`StatePlugin`](extra::app::StatePlugin) and initialize your state type:
//!
//! ```rust
//! app.add_plugins(StatePlugin).init_state::<Level>();
//! ```
//!
//! Add update systems with [`StatePattern::on_update`](pattern::StatePattern::on_update):
//!
//! ```rust
//! app.add_systems(Update, (
//!     Level::ANY.on_update(update_level_timer),
//!     Level(10).on_update(update_boss_health_bar),
//!     state!(Level(4..=6)).on_update(spawn_enemy_waves),
//! ));
//! ```
//!
//! Add flush hooks with other [`StatePattern`](pattern::StatePattern) methods:
//!
//! ```rust
//! app.add_systems(StateFlush, (
//!     // Short-hand for `on_exit` followed by `on_enter`.
//!     Level::ANY.on_edge(despawn_old_level, spawn_new_level),
//!     Level(10).on_enter(play_boss_music),
//!     state!(Level(4 | 7 | 10)).on_enter(save_checkpoint),
//!     Level::with(|x| x.0 < 4).on_enter(spawn_tutorial_popup),
//! ));
//! ```

#![deny(missing_docs)]

// Allow macros to refer to this crate as `pyri_state` internally.
extern crate self as pyri_state;

pub mod extra;
pub mod pattern;
pub mod schedule;
pub mod state;
pub mod storage;

/// Re-exported traits and common types.
///
/// Import the prelude to get started:
///
/// ```rust
/// use pyri_state::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        pattern::{
            StatePattern, StatePatternExtClone, StatePatternExtEq, StateTransPattern,
            StateTransPatternExtClone,
        },
        schedule::{StateFlush, StateFlushEvent},
        state,
        state::{
            CurrentState, NextStateMut, NextStateRef, State, StateFlushMut, StateFlushRef,
            StateMut, StateMutExtClone, StateMutExtDefault,
        },
        storage::StateBuffer,
    };

    #[cfg(feature = "bevy_app")]
    pub use crate::extra::app::{AppExtState, StatePlugin};

    #[cfg(feature = "bevy_state")]
    pub use crate::extra::bevy_state::BevyState;

    #[cfg(feature = "debug")]
    pub use crate::extra::debug::StateDebugSettings;

    #[cfg(feature = "stack")]
    pub use crate::extra::stack::{StateStack, StateStackMut, StateStackMutExtClone};

    #[cfg(feature = "sequence")]
    pub use crate::extra::sequence::{StateSequence, StateSequenceMut};

    #[cfg(feature = "split")]
    pub use crate::add_to_split_state;
    #[cfg(feature = "split")]
    pub use crate::extra::split::SplitState;

    /// A derive macro for the [`State`] and [`AddState`](crate::extra::app::AddState) traits.
    ///
    /// # Examples
    ///
    /// The derive macro requires `Clone`, `PartialEq`, and `Eq` by default:
    ///
    /// ```rust
    /// #[derive(State, Clone, PartialEq, Eq)]
    /// enum GameState { ... }
    /// ```
    ///
    /// They can be omitted if you disable the default options:
    ///
    /// ```rust
    /// #[derive(State)]
    /// #[state(no_defaults)]
    /// struct RawState;
    /// ```
    ///
    /// The following options are provided:
    ///
    /// ```rust
    /// #[derive(State, Clone, PartialEq, Eq, Hash, Debug)]
    /// #[state(
    ///     // Disable default plugins: detect_change, flush_event, apply_flush.
    ///     no_defaults,
    ///     // Trigger a flush on any state change (requires PartialEq, Eq).
    ///     detect_change,
    ///     // Send an event on flush (requires Clone).
    ///     flush_event,
    ///     // Log on exit, transition, and enter (requires Debug).
    ///     log_flush,
    ///     // Include a `BevyState<Self>` wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
    ///     bevy_state,
    ///     // Clone the next state into the current state on flush (requires Clone).
    ///     apply_flush,
    ///     // Swap out the default `StateBuffer<Self>` for a custom storage type.
    ///     storage(StateStack<Self>),
    ///     // Run this state's on flush systems after resolving the listed states.
    ///     after(GameState),
    ///     // Run this state's on flush systems before resolving the listed states.
    ///     before(RawState),
    /// )]
    /// struct ConfiguredState;
    /// ```
    pub use pyri_state_derive::State;
}
