//! [`State`] trait and extension traits.

use std::marker::PhantomData;

use bevy_ecs::{
    component::{Component, Mutable},
    resource::Resource,
    system::{Res, ResMut},
};

use crate::{
    access::{CurrentRef, FlushMut, FlushRef, NextMut, NextRef},
    next_state::{NextState, NextStateMut, TriggerStateFlush},
    pattern::{AnyStatePattern, AnyStateTransPattern, FnStatePattern, FnStateTransPattern},
};

/// A [`Resource`] that can be used as a state.
///
/// This trait can be [derived](pyri_state_derive::State) or implemented manually:
///
/// ```
/// # /*
/// #[derive(State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// #[derive(Resource)]
/// enum MenuState { ... }
/// impl State for MenuState {
///     type Next = NextStateBuffer<Self>;
/// }
/// # */
/// ```
///
/// The derive macro would also implement
/// [`RegisterState`](crate::setup::RegisterState) for `MenuState`.
///
/// See the following extension traits with additional bounds on `Self` and [`Self::Next`](State::Next):
///
/// - [`StateExtEq`]
/// - [`StateMut`]
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait State: Resource + Sized {
    /// The [`NextState`] type that determines the next state for this state type.
    type Next: NextState<State = Self>;

    /// The [`AnyStatePattern`] for this state type.
    const ANY: AnyStatePattern<Self> = AnyStatePattern(PhantomData);

    /// The [`AnyStateTransPattern`] for this state type.
    const ANY_TO_ANY: AnyStateTransPattern<Self> = AnyStateTransPattern(PhantomData);

    /// Create a [`FnStatePattern`] from a callback.
    fn with<F>(f: F) -> FnStatePattern<Self, F>
    where
        F: 'static + Send + Sync + Fn(&Self) -> bool,
    {
        FnStatePattern::new(f)
    }

    /// Create a [`FnStateTransPattern`] from a callback.
    fn when<F>(f: F) -> FnStateTransPattern<Self, F>
    where
        F: 'static + Send + Sync + Fn(&Self, &Self) -> bool,
    {
        FnStateTransPattern::new(f)
    }

    /// A run condition that checks if the current state is disabled.
    fn is_disabled(state: CurrentRef<Self>) -> bool {
        state.is_disabled()
    }

    /// A run condition that checks if the current state is enabled.
    fn is_enabled(state: CurrentRef<Self>) -> bool {
        state.is_enabled()
    }

    /// A run condition that checks if the next state will be disabled.
    fn will_be_disabled(next: NextRef<Self>) -> bool {
        next.get().is_none()
    }

    /// A run condition that checks if the next state will be enabled.
    fn will_be_enabled(next: NextRef<Self>) -> bool {
        next.get().is_some()
    }

    /// A run condition that checks if this state type is triggered to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn is_triggered(trigger: Res<TriggerStateFlush<Self>>) -> bool {
        trigger.0
    }

    /// A system that triggers this state type to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.0 = true;
    }

    /// A system that resets the trigger for this state type to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn reset_trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.0 = false;
    }
}

/// An extension trait for [`State`] types that also implement [`Eq`].
pub trait StateExtEq: State + Eq {
    /// A run condition that checks if this state type will change if triggered.
    fn will_change(state: FlushRef<Self>) -> bool {
        state.will_change()
    }
}

impl<S: State + Eq> StateExtEq for S {}

/// An extension trait for [`State`] types with [mutable `NextState`](NextStateMut).
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait StateMut: State<Next: NextStateMut> {
    /// A system that disables the next state.
    fn disable(mut state: NextMut<Self>) {
        state.set(None);
    }
}

impl<S: State<Next: NextStateMut>> StateMut for S {}

/// An extension trait for [`StateMut`] types that also implement [`Clone`].
pub trait StateMutExtClone: StateMut + Clone {
    /// Build a system that enables the next state with a specific value if the current
    /// state is disabled.
    fn enable(self) -> impl Fn(FlushMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.enable(self.clone());
        }
    }

    /// Build a system that sets the next state to a toggle of the current state between
    /// disabled and enabled with a specific value.
    fn toggle(self) -> impl Fn(FlushMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.toggle(self.clone());
        }
    }

    /// Build a system that enables the next state with a specific value.
    fn enter(self) -> impl Fn(NextMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.enter(self.clone());
        }
    }

    /// A system that resets the next state to the current state and resets the trigger to flush.
    fn reset(mut state: FlushMut<Self>) {
        state.reset();
    }

    /// A system that resets the next state to the current state and triggers a flush.
    fn refresh(mut state: FlushMut<Self>) {
        state.refresh();
    }
}

impl<S: StateMut + Clone> StateMutExtClone for S {}

/// An extension trait for [`StateMut`] types that also implement [`Default`].
pub trait StateMutExtDefault: StateMut + Default {
    /// A system that enables the next state with the default value if the current state is
    /// disabled.
    fn enable_default(mut state: FlushMut<Self>) {
        state.enable_default();
    }

    /// A system that sets the next state to a toggle of the current state between disabled
    /// and enabled with the default value.
    fn toggle_default(mut state: FlushMut<Self>) {
        state.toggle_default();
    }

    /// A system that enables the next state with the default value.
    fn enter_default(mut state: NextMut<Self>) {
        state.enter_default();
    }
}

impl<S: StateMut + Default> StateMutExtDefault for S {}

/// A marker trait for [`State`] types that can be stored as components on entities.
pub trait LocalState: State<Next: Component> + Component<Mutability = Mutable> {}

impl<S: State<Next: Component> + Component<Mutability = Mutable>> LocalState for S {}
