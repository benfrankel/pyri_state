//! `pyri_state` is a `bevy_state` alternative offering flexible change detection & scheduling.
//!
//! **NOTE:** This crate is incompatible with the `bevy/bevy_state` feature, so make sure it's
//! disabled.
//!
//! # Overview
//!
//! 1. The current state is a [`Resource`](bevy_ecs::resource::Resource) or
//!    [`Component`](bevy_ecs::component::Component) that implements [`State`](state::State).
//! 2. The [next state](next_state) is stored in a
//!    [`NextStateBuffer`](next_state::buffer::NextStateBuffer) resource by default.
//! 3. A state flush is triggered by the [`TriggerStateFlush`](next_state::TriggerStateFlush)
//!    resource and handled in the [`StateFlush`](schedule::StateFlush) schedule.
//! 4. State flush hooks are organized into [`ResolveStateSystems`](schedule::ResolveStateSystems)
//!    system sets.
//! 5. Tools are provided for state [setup], [access], [pattern-matching](pattern),
//!    [debugging](debug), and [more](extra).
//!
//! # Getting started
//!
//! Import the [prelude] to bring traits and common types into scope:
//!
//! ```
//! use pyri_state::prelude::*;
//! ```
//!
//! Define your own [`State`](state::State) type using the
//! [derive macro](pyri_state_derive::State):
//!
//! ```
//! # use pyri_state::prelude::*;
//! #
//! #[derive(State, Clone, PartialEq, Eq, Default)]
//! struct Level(pub usize);
//! ```
//!
//! Add [`StatePlugin`](setup::StatePlugin) and initialize your state type:
//!
//! ```
//! # use bevy::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Default)]
//! # struct Level(pub usize);
//! #
//! # fn plugin(app: &mut App) {
//! app.add_plugins(StatePlugin).init_state::<Level>();
//! # }
//! ```
//!
//! Add update systems with [`StatePattern::on_update`](pattern::StatePattern::on_update):
//!
//! ```
//! # use bevy::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Default)]
//! # struct Level(pub usize);
//! #
//! # fn update_level_timer() {}
//! # fn update_boss_health_bar() {}
//! # fn spawn_enemy_waves() {}
//! #
//! # fn plugin(app: &mut App) {
//! app.add_systems(Update, (
//!     Level::ANY.on_update(update_level_timer),
//!     Level(10).on_update(update_boss_health_bar),
//!     state!(Level(4..=6)).on_update(spawn_enemy_waves),
//! ));
//! # }
//! ```
//!
//! Add flush hooks with other [`StatePattern`](pattern::StatePattern) methods:
//!
//! ```
//! # use bevy::prelude::*;
//! # use pyri_state::prelude::*;
//! #
//! # #[derive(State, Clone, PartialEq, Eq, Default)]
//! # struct Level(pub usize);
//! #
//! # fn despawn_old_level() {}
//! # fn spawn_new_level() {}
//! # fn play_boss_music() {}
//! # fn save_checkpoint() {}
//! # fn spawn_tutorial_popup() {}
//! #
//! # fn plugin(app: &mut App) {
//! app.add_systems(StateFlush, (
//!     // Short-hand for `on_exit` followed by `on_enter`.
//!     Level::ANY.on_edge(despawn_old_level, spawn_new_level),
//!     Level(10).on_enter(play_boss_music),
//!     state!(Level(4 | 7 | 10)).on_enter(save_checkpoint),
//!     Level::with(|x| x.0 < 4).on_enter(spawn_tutorial_popup),
//! ));
//! # }
//! ```

#![no_std]
// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

extern crate alloc;

// Allow macros to refer to this crate as `pyri_state` internally.
extern crate self as pyri_state;

pub mod access;
#[cfg(feature = "debug")]
pub mod debug;
pub mod extra;
pub mod next_state;
pub mod pattern;
pub mod schedule;
pub mod setup;
pub mod state;

/// Re-exported traits and common types.
///
/// Import the prelude to get started:
///
/// ```
/// use pyri_state::prelude::*;
/// ```
pub mod prelude {
    pub use crate::{
        access::{CurrentMut, CurrentRef, FlushMut, FlushRef, NextMut, NextRef},
        next_state::{buffer::NextStateBuffer, stack::NextStateStackCommandsExt as _},
        pattern::{
            StatePattern as _, StatePatternExtClone as _, StatePatternExtEq as _,
            StateTransPattern as _, StateTransPatternExtClone as _,
        },
        schedule::{StateFlush, flush_event::StateFlushEvent},
        setup::{CommandsExtState as _, EntityCommandsExtState as _},
        state,
        state::{
            State, StateExtEq as _, StateMut as _, StateMutExtClone as _, StateMutExtDefault as _,
        },
    };

    #[cfg(feature = "bevy_app")]
    pub use crate::setup::{AppExtState as _, StatePlugin};

    #[cfg(feature = "bevy_state")]
    pub use crate::extra::bevy_state::{BevyState, StateExtBevy as _};

    #[cfg(feature = "debug")]
    pub use crate::debug::StateDebugSettings;

    #[cfg(feature = "react")]
    pub use crate::extra::react::{
        DespawnOnDisableState, DespawnOnExitState, EnabledInEnabledState, EnabledInState,
        VisibleInEnabledState, VisibleInState,
    };

    #[cfg(feature = "sequence")]
    pub use crate::next_state::sequence::{
        NextStateIndex, NextStateIndexMut as _, NextStateSequence,
    };

    #[cfg(feature = "split")]
    pub use crate::{add_to_split_state, extra::split::SplitState};

    #[cfg(feature = "stack")]
    pub use crate::next_state::stack::{
        NextStateStack, NextStateStackMut as _, NextStateStackMutExtClone as _,
    };

    /// A derive macro for the [`State`],
    /// [`RegisterState`](crate::setup::RegisterState), and
    /// [`Resource`](bevy_ecs::resource::Resource) traits.
    ///
    /// # Examples
    ///
    /// The derive macro requires `Clone`, `PartialEq`, and `Eq`:
    ///
    /// ```
    /// # use pyri_state::prelude::*;
    /// #
    /// #[derive(State, Clone, PartialEq, Eq)]
    /// enum MyState { /* ... */ }
    /// ```
    ///
    /// They can be omitted if you disable the default options:
    ///
    /// ```
    /// # use pyri_state::prelude::*;
    /// #
    /// #[derive(State)]
    /// #[state(no_defaults)]
    /// struct RawState;
    /// ```
    ///
    /// The following options are provided:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use pyri_state::prelude::*;
    /// #
    /// # #[derive(State, Clone, PartialEq, Eq)]
    /// # enum MyState { /* ... */ }
    /// #
    /// # #[derive(State)]
    /// # #[state(no_defaults)]
    /// # struct RawState;
    /// #
    /// #[derive(State, Component, Clone, PartialEq, Eq, Hash, Debug)]
    /// #[state(
    ///     // Disable default plugins: detect_change, flush_event, apply_flush.
    ///     no_defaults,
    ///     // Support local state (requires Component).
    ///     local,
    ///     // Trigger a flush on any state change (requires PartialEq, Eq).
    ///     detect_change,
    ///     // Send an event on flush (requires Clone).
    ///     flush_event,
    ///     // Log on flush (requires Debug).
    ///     log_flush,
    ///     // Include a `BevyState<Self>` wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
    ///     bevy_state,
    ///     // Enable reaction components such as `DespawnOnExitState<Self>` (requires Eq).
    ///     react,
    ///     // Clone the next state into the current state on flush (requires Clone).
    ///     apply_flush,
    ///     // Swap out the default `NextStateBuffer<Self>` for another `NextState` type.
    ///     next(NextStateStack<Self>),
    ///     // Run this state's on-flush hooks after the listed states.
    ///     after(MyState),
    ///     // Run this state's on-flush hooks before the listed states.
    ///     before(RawState),
    /// )]
    /// struct ConfiguredState;
    /// ```
    pub use pyri_state_derive::State;
}
