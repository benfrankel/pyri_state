//! `pyri_state` is a `bevy_state` alternative offering flexible change detection & scheduling.
//!
//! # Design overview
//!
//! TODO: Improve this section.
//!
//! The next state is stored in a [`StateBuffer`](storage::StateBuffer) by default, but this
//! can be swapped for any [`StateStorage`](storage::StateStorage) type with one line of code.
//!
//! States are flushed in the [`StateFlush`](schedule::StateFlush) schedule, which is
//! organized into [`StateFlushSet`](schedule::StateFlushSet) system sets.
//!
//! A flush is triggered by the [`TriggerStateFlush`](state::TriggerStateFlush) flag, which
//! can be set manually or by the opt-out change detection system.
//!
//! # Getting started
//!
//! Import the [prelude] to bring traits and common types into scope:
//!
//! ```rust
//! // Note: Disable the `bevy_state` feature of `bevy` to avoid prelude interference.
//! // use bevy::prelude::*;
//! use pyri_state::prelude::*;
//! ```
//!
//!
//! Derive [`State`](state::State) to define your own state types:
//!
//! ```rust
//! #[derive(State, Clone, PartialEq, Eq, Default)]
//! struct Level(pub usize);
//! ```
//!
//! Add [`StatePlugin`](app::StatePlugin), and initialize your state types:
//!
//! ```rust
//! app.add_plugins(StatePlugin).init_state::<Level>();
//! ```
//!
//! Use [`on_update`](pattern::StatePattern::on_update) to add pattern-matching update systems:
//!
//! ```rust
//! app.add_systems(Update, (
//!     Level::ANY.on_update(update_level_timer),
//!     Level(10).on_update(update_boss_health_bar),
//!     state!(Level(4..=6)).on_update(spawn_enemy_waves),
//! ));
//! ```
//!
//! Use the other [`StatePattern`](pattern::StatePattern) methods to add flush hooks:
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
//!
//! # Index
//!
//! TODO: Split this into module-level documentation
//!
//! Helper systems and run conditions are provided by the following traits:
//!
//! - [`State`](state::State)
//! - [`StateMut`](state::StateMut)
//! - [`StateMutExtClone`](state::StateMutExtClone)
//! - [`StateMutExtDefault`](state::StateMutExtDefault)
//! - [`StatePattern`](pattern::StatePattern)
//! - [`StatePatternExtClone`](pattern::StatePatternExtClone)
//! - [`StatePatternExtEq`](pattern::StatePatternExtEq)
//! - [`StateTransPattern`](pattern::StateTransPattern)
//! - [`StateTransPatternExtClone`](pattern::StateTransPattern)
//!
//! You can write custom systems and run conditions using the following
//! [`SystemParam`s](bevy_ecs::system::SystemParam):
//!
//! | State          | Read-only                                     | Mutable                                          |
//! | -------------- | --------------------------------------------- | ------------------------------------------------ |
//! | Current        | [`Res<CurrentState<S>>`](state::CurrentState) | [`ResMut<CurrentState<S>>`](state::CurrentState) |
//! | Next           | [`NextStateRef<S>`](state::NextStateRef)      | [`NextStateMut<S>`](state::NextStateMut)         |
//! | Current + Next | [`StateFlushRef<S>`](state::StateFlushRef)    | [`StateFlushMut<S>`](state::StateFlushMut)       |
//!
//! State storage types provided:
//!
//! - [`StateBuffer`](storage::StateBuffer) (default)
//! - [`StateStack`](extra::stack::StateStack)
//! - [`StateSequence`](extra::sequence::StateSequence)
//!
//! State plugins provided:
//!
//! - [`ResolveStatePlugin`](app::ResolveStatePlugin)
//! - [`DetectChangePlugin`](app::DetectChangePlugin)
//! - [`FlushEventPlugin`](app::FlushEventPlugin)
//! - [`LogFlushPlugin`](debug::LogFlushPlugin)
//! - [`BevyStatePlugin`](app::BevyStatePlugin)
//! - [`ApplyFlushPlugin`](app::ApplyFlushPlugin)

#![deny(missing_docs)]

// Allow macros to refer to this crate as `pyri_state` internally.
extern crate self as pyri_state;

#[cfg(feature = "bevy_app")]
pub mod app;
#[cfg(feature = "debug")]
pub mod debug;
#[cfg(feature = "extra")]
pub mod extra;
pub mod pattern;
pub mod schedule;
pub mod state;
pub mod storage;

/// TODO: Module-level documentation
pub mod prelude {
    #[cfg(feature = "bevy_app")]
    pub use crate::app::{AppExtState, StatePlugin};

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

    pub use pyri_state_derive::State;
}
