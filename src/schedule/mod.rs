//! State flush scheduling types and functions.
//!
//! The [`StateFlush`] schedule handles all [`State`](crate::state::State) flush logic
//! and emits [`StateFlushEvent`](flush_event::StateFlushEvent).

pub use apply_flush::ApplyFlushSystems;
pub use resolve_state::ResolveStateSystems;

pub mod apply_flush;
pub mod detect_change;
pub mod flush_event;
pub mod resolve_state;

use core::{fmt::Debug, hash::Hash};

use bevy_ecs::schedule::ScheduleLabel;

/// The schedule that handles all [`State`](crate::state::State) flush logic, added before
/// [`PreUpdate`](bevy_app::PreUpdate) by [`StatePlugin`](crate::setup::StatePlugin).
///
/// State flush hooks run in [`ResolveStateSystems::<S>::Flush`] and the flush is applied in
/// [`ApplyFlushSystems`].
#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;
