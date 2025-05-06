//! State pattern-matching tools.
//!
//! Use the [`state!`](crate::state!) macro to build [`StatePattern`] and
//! [`StateTransPattern`] instances.

use core::marker::PhantomData;

use bevy_ecs::{
    schedule::{Condition, IntoScheduleConfigs, ScheduleConfigs},
    system::ScheduleSystem,
};

use crate::{
    access::{CurrentRef, FlushRef, NextRef},
    schedule::ResolveStateSystems,
    state::State,
};

/// A type that can match a subset of values of the [`State`] type `S`.
///
/// If `S` implements `Eq`, it can be used directly as a state pattern.
///
/// See the following extension traits with additional bounds on `Self` and `S`:
///
/// - [`StatePatternExtClone<S>`]
/// - [`StatePatternExtEq<S>`]
pub trait StatePattern<S: State>: 'static + Send + Sync + Sized {
    /// Check if the pattern matches a particular state.
    fn matches(&self, state: &S) -> bool;

    /// Build a run condition that checks if `S` is in a matching state.
    fn will_update(self) -> impl 'static + Send + Sync + Fn(CurrentRef<S>) -> bool {
        self.will_exit()
    }

    /// Configure systems to run if `S` is in a matching state.
    fn on_update<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems.run_if(self.will_update())
    }

    /// Build a run condition that checks if `S` will exit a matching state if triggered.
    fn will_exit(self) -> impl 'static + Send + Sync + Fn(CurrentRef<S>) -> bool {
        move |state| state.is_in(&self)
    }

    /// Configure systems to run when `S` exits a matching state.
    fn on_exit<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_exit())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Exit)
    }

    /// Build a run condition that checks if `S` will become disabled from a matching state if triggered.
    fn will_disable(self) -> impl 'static + Send + Sync + Fn(FlushRef<S>) -> bool {
        move |state| state.will_disable(&self)
    }

    /// Configure systems to run when `S` is disabled from a matching state.
    fn on_disable<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_disable())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Exit)
    }

    /// Build a run condition that checks if `S` will enter into a matching state if triggered.
    fn will_enter(self) -> impl 'static + Send + Sync + Fn(NextRef<S>) -> bool {
        move |state| state.will_be_in(&self)
    }

    /// Configure systems to run when `S` enters a matching state.
    fn on_enter<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_enter())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Enter)
    }

    /// Build a run condition that checks if `S` will become enabled in a matching state if triggered.
    fn will_enable(self) -> impl 'static + Send + Sync + Fn(FlushRef<S>) -> bool {
        move |state| state.will_enable(&self)
    }

    /// Configure systems to run when `S` becomes enabled in a matching state.
    fn on_enable<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(S::is_triggered.and(self.will_enable()))
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Enter)
    }
}

/// An extension trait for [`StatePattern`] types that also implement `Clone`.
pub trait StatePatternExtClone<S: State>: StatePattern<S> + Clone {
    /// Helper method for configuring [`on_exit`](StatePattern::on_exit) and
    /// [`on_enter`](StatePattern::on_enter) systems for the same `StatePattern`.
    fn on_edge<M1, M2>(
        self,
        exit_systems: impl IntoScheduleConfigs<ScheduleSystem, M1>,
        enter_systems: impl IntoScheduleConfigs<ScheduleSystem, M2>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        (
            self.clone().on_exit(exit_systems),
            self.on_enter(enter_systems),
        )
            .into_configs()
    }
}

impl<S: State, P: StatePattern<S> + Clone> StatePatternExtClone<S> for P {}

/// An extension trait for [`StatePattern<S>`] when `S` also implements `Eq`.
pub trait StatePatternExtEq<S: State + Eq>: StatePattern<S> {
    /// Build a run condition that checks if `S` will refresh in a matching state if triggered.
    fn will_refresh(self) -> impl 'static + Send + Sync + Fn(FlushRef<S>) -> bool {
        move |state| state.will_refresh(&self)
    }

    /// Configure systems to run when `S` refreshes in a matching state.
    fn on_refresh<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_refresh())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Trans)
    }
}

impl<S: State + Eq, P: StatePattern<S>> StatePatternExtEq<S> for P {}

impl<S: State + Eq> StatePattern<S> for S {
    fn matches(&self, state: &S) -> bool {
        self == state
    }
}

/// A wildcard [`StatePattern`] for the [`State`] type `S`.
///
/// The usual way to use `AnyStatePattern` is through the associated constant [`State::ANY`]:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn reset_timer() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, Level::ANY.on_enter(reset_timer));
/// # }
/// ```
pub struct AnyStatePattern<S: State>(pub(crate) PhantomData<S>);

impl<S: State> Clone for AnyStatePattern<S> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

// TODO: Optimization: Instead of impling the trait, raw impl the methods and use `AnyExit` etc. system sets.
impl<S: State> StatePattern<S> for AnyStatePattern<S> {
    fn matches(&self, _state: &S) -> bool {
        true
    }
}

/// A [`StatePattern`] that runs a callback to determine which values of the [`State`]
/// type `S` should match.
///
/// The usual way to construct this type is with the [`state!`](crate::state!) macro or
/// [`State::with`]:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn save_checkpoint() {}
/// # fn my_systems() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, state!(Level(4 | 7 | 10)).on_enter(save_checkpoint));
/// app.add_systems(StateFlush, Level::with(|x| x.0 < 4).on_refresh(my_systems));
/// # }
/// ```
#[derive(Clone)]
pub struct FnStatePattern<S: State, F>(F, PhantomData<S>)
where
    F: 'static + Send + Sync + Fn(&S) -> bool;

impl<S: State, F> StatePattern<S> for FnStatePattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S) -> bool,
{
    fn matches(&self, state: &S) -> bool {
        self.0(state)
    }
}

impl<S: State, F> FnStatePattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S) -> bool,
{
    /// Create a new `FnStatePattern`.
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

/// A type that can match a subset of transitions in the [`State`] type `S`.
///
/// A tuple of two [`StatePattern`] types can be used as a transition pattern.
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`StateTransPatternExtClone`]
pub trait StateTransPattern<S: State>: 'static + Send + Sync + Sized {
    /// Check if the pattern matches a particular pair of states.
    fn matches(&self, old: &S, new: &S) -> bool;

    /// Build a run condition that checks if `S` will undergo a matching transition if triggered.
    fn will_trans(self) -> impl 'static + Send + Sync + Fn(FlushRef<S>) -> bool {
        move |state| state.will_trans(&self)
    }

    /// Configure systems to run when `S` exits as part of a matching transition.
    fn on_exit<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_trans())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Exit)
    }

    /// Configure systems to run when `S` undergoes a matching transition.
    fn on_trans<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_trans())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Trans)
    }

    /// Configure systems to run when `S` enters as part of a matching transition.
    fn on_enter<M>(
        self,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        systems
            .run_if(self.will_trans())
            .in_set(ResolveStateSystems::<S>::AnyFlush)
            .in_set(ResolveStateSystems::<S>::Enter)
    }
}

/// An extension trait for [`StateTransPattern`] types that also implement `Clone`.
pub trait StateTransPatternExtClone<S: State>: StateTransPattern<S> + Clone {
    /// Helper method for configuring [`on_exit`](StateTransPattern::on_exit) and
    /// [`on_enter`](StateTransPattern::on_enter) systems for the same `StateTransPattern`.
    fn on_edge<M1, M2>(
        self,
        exit_systems: impl IntoScheduleConfigs<ScheduleSystem, M1>,
        enter_systems: impl IntoScheduleConfigs<ScheduleSystem, M2>,
    ) -> ScheduleConfigs<ScheduleSystem> {
        (
            self.clone().on_exit(exit_systems),
            self.on_enter(enter_systems),
        )
            .into_configs()
    }
}

impl<S: State, P: StateTransPattern<S> + Clone> StateTransPatternExtClone<S> for P {}

impl<S: State, P1: StatePattern<S>, P2: StatePattern<S>> StateTransPattern<S> for (P1, P2) {
    fn matches(&self, old: &S, new: &S) -> bool {
        self.0.matches(old) && self.1.matches(new)
    }
}

/// A wildcard [`StateTransPattern`] for the [`State`] type `S`.
///
/// The usual way to use this type is through the associated constant [`State::ANY_TO_ANY`]:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn reset_timer() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, Level::ANY_TO_ANY.on_trans(reset_timer));
///
/// // Equivalent to:
/// app.add_systems(StateFlush, (Level::ANY, Level::ANY).on_trans(reset_timer));
/// # }
/// ```
///
#[derive(Clone)]
pub struct AnyStateTransPattern<S: State>(pub(crate) PhantomData<S>);

impl<S: State> StateTransPattern<S> for AnyStateTransPattern<S> {
    fn matches(&self, _old: &S, _new: &S) -> bool {
        true
    }
}

/// A [`StateTransPattern`] that runs a callback to determine which transitions
/// of the [`State`] type `S` should match.
///
/// The usual way to construct this type is with the [`state!`](crate::state!) macro or
/// [`State::when`]:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn spawn_something_cool() {}
/// # fn play_next_level_sfx() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, state!(Level(2..=5 | 7) => Level(8 | 10)).on_enter(spawn_something_cool));
/// app.add_systems(StateFlush, Level::when(|x, y| y.0 > x.0).on_enter(play_next_level_sfx));
/// # }
/// ```
#[derive(Clone)]
pub struct FnStateTransPattern<S: State, F>(F, PhantomData<S>)
where
    F: 'static + Send + Sync + Fn(&S, &S) -> bool;

impl<S: State, F> StateTransPattern<S> for FnStateTransPattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S, &S) -> bool,
{
    fn matches(&self, old: &S, new: &S) -> bool {
        self.0(old, new)
    }
}

impl<S: State, F> FnStateTransPattern<S, F>
where
    F: 'static + Send + Sync + Fn(&S, &S) -> bool,
{
    /// Create a new `FnStateTransPattern`.
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

/// A macro for building pattern-matching [`FnStatePattern`] and [`FnStateTransPattern`] instances.
///
/// # Examples
///
/// State pattern-matching:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn save_checkpoint() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, state!(Level(4 | 7 | 10)).on_enter(save_checkpoint));
/// # }
/// ```
///
/// State transition pattern-matching:
///
/// ```
/// # use bevy::prelude::*;
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct Level(usize);
/// #
/// # fn do_something_cool() {}
/// #
/// # fn plugin(app: &mut App) {
/// app.add_systems(StateFlush, state!(Level(x @ 1..=3) => y if y.0 == 10 - x).on_trans(do_something_cool));
/// # }
/// ```
#[macro_export]
macro_rules! state {
    ($state:pat $(if $guard:expr)? $(,)?) => {
        pyri_state::pattern::FnStatePattern::new(
            |state| matches!(state, $state $(if $guard)?),
        )
    };

    ($old:pat => $new:pat $(if $guard:expr)? $(,)?) => {
        pyri_state::pattern::FnStateTransPattern::new(
            |old, new| matches!((old, new), ($old, $new) $(if $guard)?),
        )
    };
}
