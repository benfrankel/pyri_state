//! State flush scheduling types and functions.
//!
//! The [`StateFlush`] schedule handles all [`State`](crate::state::State) flush logic
//! and emits [`StateFlushEvent`](flush_event::StateFlushEvent).

pub mod apply_flush;
pub mod detect_change;
pub mod flush_event;
pub mod resolve_state;

use std::{fmt::Debug, hash::Hash};

use bevy_ecs::schedule::ScheduleLabel;

pub use apply_flush::ApplyFlushSet;
pub use resolve_state::StateHook;

/// The schedule that handles all [`State`](crate::state::State) flush logic, added after
/// [`PreUpdate`](bevy_app::PreUpdate) by [`StatePlugin`](crate::extra::app::StatePlugin).
///
/// State flush hooks run in [`StateHook<S>`] system sets and the flush is applied in
/// [`ApplyFlushSet`].
#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;
