//! Configure [`StateHook`] system sets.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use std::marker::PhantomData;

    use bevy_app::{App, Plugin};
    use bevy_ecs::schedule::{InternedSystemSet, SystemSet};

    use crate::state::State;

    use super::{schedule_resolve_state, StateHook};

    /// A plugin that configures the [`StateHook<S>`] system sets for the [`State`] type `S`
    /// in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    ///
    /// To specify a dependency relative to another `State` type `T`, add
    /// [`StateHook::<T>::Resolve`] to [`after`](Self::after) or [`before`](Self::before).
    ///
    /// Calls [`schedule_resolve_state<S>`].
    pub struct ResolveStatePlugin<S: State> {
        after: Vec<InternedSystemSet>,
        before: Vec<InternedSystemSet>,
        _phantom: PhantomData<S>,
    }

    impl<S: State> Plugin for ResolveStatePlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_resolve_state::<S>(
                app.get_schedule_mut(crate::schedule::StateFlush).unwrap(),
                &self.after,
                &self.before,
            );
        }
    }

    impl<S: State> Default for ResolveStatePlugin<S> {
        fn default() -> Self {
            Self {
                after: Vec::new(),
                before: Vec::new(),
                _phantom: PhantomData,
            }
        }
    }

    impl<S: State> ResolveStatePlugin<S> {
        /// Create a [`ResolveStatePlugin`] from `.after` and `.before` system sets.
        pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
            Self {
                after,
                before,
                _phantom: PhantomData,
            }
        }

        /// Configure a `.after` system set.
        pub fn after<T: State>(mut self) -> Self {
            self.after.push(StateHook::<T>::Resolve.intern());
            self
        }

        /// Configure a `.before` system set.
        pub fn before<T: State>(mut self) -> Self {
            self.before.push(StateHook::<T>::Resolve.intern());
            self
        }
    }
}

use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::schedule::{InternedSystemSet, IntoSystemSetConfigs, Schedule, SystemSet};

use crate::{schedule::ApplyFlushSet, state::State};

/// A suite of system sets in the [`StateFlush`](crate::schedule::StateFlush)
/// schedule for each [`State`] type `S`.
///
/// Configured [by default](pyri_state_derive::State) by
/// [`ResolveStatePlugin<S>`] as follows:
///
/// 1. [`Resolve`](Self::Resolve) (before or after other `Resolve` system sets based on
/// state dependencies, and before [`ApplyFlushSet`])
///     1. [`Compute`](Self::Compute)
///     2. [`Trigger`](Self::Trigger)
///     3. [`Flush`](Self::Flush)
///         1. [`Exit`](Self::Exit)
///         2. [`Trans`](Self::Trans)
///         3. [`Enter`](Self::Enter)
#[derive(SystemSet)]
pub enum StateHook<S: State> {
    /// Resolve the state flush logic for `S` this frame.
    Resolve,
    /// Optionally compute the next value for `S`.
    Compute,
    /// Decide whether to trigger a flush for `S` this frame.
    Trigger,
    /// Run flush hooks for `S`.
    Flush,
    /// Run exit hooks for `S`.
    Exit,
    /// Run transition hooks for `S`.
    Trans,
    /// Run enter hooks for `S`.
    Enter,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State> Clone for StateHook<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolve => Self::Resolve,
            Self::Compute => Self::Compute,
            Self::Trigger => Self::Trigger,
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Trans => Self::Trans,
            Self::Enter => Self::Enter,
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

impl<S: State> PartialEq for StateHook<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State> Eq for StateHook<S> {}

impl<S: State> Hash for StateHook<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State> Debug for StateHook<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve => write!(f, "Resolve"),
            Self::Compute => write!(f, "Compute"),
            Self::Trigger => write!(f, "Trigger"),
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Trans => write!(f, "Trans"),
            Self::Enter => write!(f, "Enter"),
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

/// Configure [`StateHook<S>`] system sets for the [`State`] type `S` in a schedule.
///
/// To specify a dependency relative to another `State` type `T`, include
/// [`StateHook::<T>::Resolve`] in `after` or `before`.
///
/// Used in [`ResolveStatePlugin<S>`].
pub fn schedule_resolve_state<S: State>(
    schedule: &mut Schedule,
    after: &[InternedSystemSet],
    before: &[InternedSystemSet],
) {
    // External ordering
    for &system_set in after {
        schedule.configure_sets(StateHook::<S>::Resolve.after(system_set));
    }
    for &system_set in before {
        schedule.configure_sets(StateHook::<S>::Resolve.before(system_set));
    }

    // Internal ordering
    schedule.configure_sets((
        StateHook::<S>::Resolve.before(ApplyFlushSet),
        (
            StateHook::<S>::Compute,
            // Systems in this system set should only run if not triggered.
            StateHook::<S>::Trigger,
            // Systems in this system set should only run if triggered.
            StateHook::<S>::Flush,
        )
            .chain()
            .in_set(StateHook::<S>::Resolve),
        (
            StateHook::<S>::Exit,
            StateHook::<S>::Trans,
            StateHook::<S>::Enter,
        )
            .chain()
            .in_set(StateHook::<S>::Flush),
    ));
}
