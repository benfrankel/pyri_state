//! System parameters to access current and next states.
//!
//! Use the following [`SystemParam`] types to access the [`State`] type `S` in your systems
//! and run conditions:
//!
//! | State          | Read-only access  | Mutable access     |
//! | -------------- | ----------------- | ------------------ |
//! | Current        | [`CurrentRef<S>`] | [`CurrentMut<S>`]* |
//! | Next           | [`NextRef<S>`]    | [`NextMut<S>`]     |
//! | Current & Next | [`FlushRef<S>`]   | [`FlushMut<S>`]    |
//!
//! \* NOTE: Don't mutate the current state directly unless you know what you're doing.

use bevy_ecs::system::{Res, ResMut, StaticSystemParam, SystemParam};

use crate::{
    next_state::{NextState, NextStateMut, TriggerStateFlush},
    pattern::{StatePattern, StateTransPattern},
    state::{State, StateMut},
};

// TODO: Manually impl `SystemParam` to skip the query and contain `Option<&S>` directly (if that's possible).
// TODO: Manually impl `QueryData` as well.
/// A [`SystemParam`] with read-only access to the current value of the [`State`] type `S`.
#[derive(SystemParam)]
pub struct CurrentRef<'w, S: State>(Option<Res<'w, S>>);

impl<S: State> CurrentRef<'_, S> {
    /// Get a read-only reference to the current state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.0.as_deref()
    }

    /// Get a read-only reference to the current state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Check if the current state is disabled.
    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the current state is enabled.
    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the current state is enabled and matches a specific [`StatePattern`].
    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }
}

/// A [`SystemParam`] with mutable access to the current value of the [`State`] type `S`.
///
/// NOTE: Don't mutate the current state directly unless you know what you're doing.
#[derive(SystemParam)]
pub struct CurrentMut<'w, S: State>(Option<ResMut<'w, S>>);

impl<S: State> CurrentMut<'_, S> {
    /// Get a read-only reference to the current state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.0.as_deref()
    }

    /// Get a mutable reference to the current state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.0.as_deref_mut()
    }

    /// Get a read-only reference to the current state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Get a mutable reference to the current state, or panic if disabled.
    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    /// Check if the current state is disabled.
    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    /// Check if the current state is enabled.
    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Check if the current state is enabled and matches a specific [`StatePattern`].
    pub fn is_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }
}

/// A [`SystemParam`] with read-only access to the next value of the [`State`] type `S`.
///
/// NOTE: The next state is only set in stone during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`ResolveStateSystems::<S>::Compute`](crate::schedule::ResolveStateSystems::Compute).
///
/// # Example
///
/// ```
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # enum Menu {
/// #     Main,
/// #     Settings,
/// # }
/// #
/// # fn spawn_main_menu() {}
/// # fn spawn_settings_menu() {}
/// #
/// fn spawn_new_menu(menu: NextRef<Menu>) {
///     match menu.unwrap() {
///         Menu::Main => spawn_main_menu(),
///         Menu::Settings => spawn_settings_menu(),
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct NextRef<'w, 's, S: State> {
    next: Res<'w, <S as State>::Next>,
    next_param: StaticSystemParam<'w, 's, <<S as State>::Next as NextState>::Param>,
    trigger: Res<'w, TriggerStateFlush<S>>,
}

impl<S: State> NextRef<'_, '_, S> {
    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.next.next_state(&self.next_param)
    }

    /// Get a read-only reference to the next state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Check if the next state will be disabled.
    pub fn will_be_disabled(&self) -> bool {
        self.get().is_none()
    }

    /// Check if the next state will be enabled.
    pub fn will_be_enabled(&self) -> bool {
        self.get().is_some()
    }

    /// Check if the next state will be enabled and match a specific [`StatePattern`].
    pub fn will_be_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }

    /// Check if `S` is triggered to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn is_triggered(&self) -> bool {
        self.trigger.0
    }
}

/// A [`SystemParam`] with mutable access to the next value of the [`State`] type `S`.
///
/// NOTE: The next state should not be mutated during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`ResolveStateSystems::<S>::Compute`](crate::schedule::ResolveStateSystems::Compute).
///
/// # Example
///
/// ```
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct ColorState {
/// #     red: bool,
/// #     green: bool,
/// #     blue: bool,
/// # }
/// #
/// fn toggle_blue(mut color: NextMut<ColorState>) {
///     let mut color = color.unwrap_mut();
///     color.blue = !color.blue;
/// }
/// ```
#[derive(SystemParam)]
pub struct NextMut<'w, 's, S: StateMut> {
    next: ResMut<'w, <S as State>::Next>,
    next_param: StaticSystemParam<'w, 's, <<S as State>::Next as NextStateMut>::ParamMut>,
    trigger: ResMut<'w, TriggerStateFlush<S>>,
}

impl<S: StateMut + Default> NextMut<'_, '_, S> {
    /// Enable the next state with the default value if it's disabled.
    pub fn enable_default(&mut self) {
        if self.will_be_disabled() {
            self.enter(S::default())
        }
    }

    /// Toggle the next state between disabled and enabled with the default value.
    pub fn toggle_default(&mut self) {
        if self.will_be_disabled() {
            self.enter_default();
        } else {
            self.disable();
        }
    }

    /// Enable the next state with the default value.
    pub fn enter_default(&mut self) {
        self.enter(S::default());
    }
}

impl<S: StateMut> NextMut<'_, '_, S> {
    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.next.next_state_from_mut(&self.next_param)
    }

    /// Get a mutable reference to the next state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        self.next.next_state_mut(&mut self.next_param)
    }

    /// Set the next state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        self.next.set_next_state(&mut self.next_param, state);
    }

    /// Get a read-only reference to the next state, or panic if disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Get a mutable reference to the next state, or panic if disabled.
    pub fn unwrap_mut(&mut self) -> &mut S {
        self.get_mut().unwrap()
    }

    /// Check if the next state will be disabled.
    pub fn will_be_disabled(&self) -> bool {
        self.get().is_none()
    }

    /// Check if the next state will be enabled.
    pub fn will_be_enabled(&self) -> bool {
        self.get().is_some()
    }

    /// Check if the next state will be enabled and match a specific [`StatePattern`].
    pub fn will_be_in<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), Some(x) if pattern.matches(x))
    }

    /// Check if `S` is triggered to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn is_triggered(&self) -> bool {
        self.trigger.0
    }

    /// Trigger `S` to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) -> &mut Self {
        self.trigger.0 = true;
        self
    }

    /// Reset the trigger for `S` to flush in the
    /// [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn reset_trigger(&mut self) -> &mut Self {
        self.trigger.0 = false;
        self
    }

    /// Disable the next state.
    pub fn disable(&mut self) {
        self.set(None);
    }

    /// Enable the next state with a specific value if it's disabled.
    pub fn enable(&mut self, value: S) {
        if self.will_be_disabled() {
            self.enter(value);
        }
    }

    /// Toggle the next state between disabled and enabled with a specific value.
    pub fn toggle(&mut self, value: S) {
        if self.will_be_disabled() {
            self.enter(value);
        } else {
            self.disable();
        }
    }

    /// Enable the next state with a specific value.
    pub fn enter(&mut self, value: S) {
        self.set(Some(value));
    }
}

/// A [`SystemParam`] with read-only access to the current and next values of the [`State`] type `S`.
///
/// NOTE: The next state is only set in stone during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`ResolveStateSystems::<S>::Compute`](crate::schedule::ResolveStateSystems::Compute).
///
/// # Example
///
/// ```
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct ColorState {
/// #     red: bool,
/// #     green: bool,
/// #     blue: bool,
/// # }
/// #
/// fn same_red(color: FlushRef<ColorState>) -> bool {
///     color.will_trans(&ColorState::when(|x, y| x.red == y.red))
/// }
/// ```
#[derive(SystemParam)]
pub struct FlushRef<'w, 's, S: State> {
    /// A system parameter with read-only access to the current state.
    pub current: CurrentRef<'w, S>,
    /// A system parameter with read-only access to the next state.
    pub next: NextRef<'w, 's, S>,
}

impl<S: State + Eq> FlushRef<'_, '_, S> {
    /// Check if `S` will refresh in a state that matches a specific pattern if triggered.
    pub fn will_refresh<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }

    /// Check if `S` will change if triggered.
    pub fn will_change(&self) -> bool {
        matches!(self.get(), (x, y) if x != y)
    }
}

impl<S: State> FlushRef<'_, '_, S> {
    /// Get read-only references to the current and next states, or `None` if disabled.
    pub fn get(&self) -> (Option<&S>, Option<&S>) {
        (self.current.get(), self.next.get())
    }

    /// Get read-only references to the current and next states, or panic if disabled.
    pub fn unwrap(&self) -> (&S, &S) {
        let (current, next) = self.get();
        (current.unwrap(), next.unwrap())
    }

    /// Check if `S` will exit a state that matches a specific pattern if triggered.
    pub fn will_exit<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), _) if pattern.matches(x))
    }

    /// Check if `S` will become disabled from a state that matches a specific pattern if triggered.
    pub fn will_disable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), None) if pattern.matches(x))
    }

    /// Check if `S` will enter a state that matches a specific pattern if triggered.
    pub fn will_enter<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (_, Some(y)) if pattern.matches(y))
    }

    /// Check if `S` will become enabled in a state that matches a specific pattern if triggered.
    pub fn will_enable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (None, Some(y)) if pattern.matches(y))
    }

    /// Check if `S` will undergo a transition that matches a specific pattern if triggered.
    pub fn will_trans<P: StateTransPattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if pattern.matches(x, y))
    }
}

/// A [`SystemParam`] with read-only and mutable access to the current and next values of the [`State`] type `S`,
/// respectively.
///
/// NOTE: The next state should not be mutated during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`ResolveStateSystems::<S>::Compute`](crate::schedule::ResolveStateSystems::Compute).
///
/// # Example
///
/// ```
/// # use pyri_state::prelude::*;
/// #
/// # #[derive(State, Clone, PartialEq, Eq)]
/// # struct ColorState {
/// #     red: bool,
/// #     green: bool,
/// #     blue: bool,
/// # }
/// #
/// fn copy_red_to_green(mut color: FlushMut<ColorState>) {
///     let (current, next) = color.unwrap_mut();
///     next.green = current.red;
/// }
/// ```
#[derive(SystemParam)]
pub struct FlushMut<'w, 's, S: StateMut> {
    /// A system parameter with read-only access to the current state.
    pub current: CurrentRef<'w, S>,
    /// A system parameter with mutable access to the next state.
    pub next: NextMut<'w, 's, S>,
}

impl<S: StateMut + Clone> FlushMut<'_, '_, S> {
    /// Set the next state to remain in the current state with no flush.
    pub fn reset(&mut self) {
        self.next.reset_trigger().set(self.current.get().cloned());
    }

    /// Set the next state to flush to the current state.
    pub fn refresh(&mut self) {
        self.next.trigger().set(self.current.get().cloned());
    }
}

impl<S: StateMut + Eq> FlushMut<'_, '_, S> {
    /// Check if `S` will refresh in a state that matches a specific pattern if triggered.
    pub fn will_refresh<P: StatePattern<S>>(&mut self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<S: StateMut> FlushMut<'_, '_, S> {
    /// Get read-only references to the current and next states, or `None` if disabled.
    pub fn get(&self) -> (Option<&S>, Option<&S>) {
        (self.current.get(), self.next.get())
    }

    /// Get a read-only and mutable reference to the current and next state respectively, or `None` if disabled.
    pub fn get_mut(&mut self) -> (Option<&S>, Option<&mut S>) {
        (self.current.get(), self.next.get_mut())
    }

    /// Get read-only references to the current and next states, or panic if disabled.
    pub fn unwrap(&self) -> (&S, &S) {
        (self.current.unwrap(), self.next.unwrap())
    }

    /// Get a read-only and mutable reference to the current and next state respectively, or panic if disabled.
    pub fn unwrap_mut(&mut self) -> (&S, &mut S) {
        (self.current.unwrap(), self.next.unwrap_mut())
    }

    /// Check if `S` will exit a state that matches a specific pattern if triggered.
    pub fn will_exit<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), _) if pattern.matches(x))
    }

    /// Check if `S` will become disabled from a state that matches a specific pattern if triggered.
    pub fn will_disable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), None) if pattern.matches(x))
    }

    /// Check if `S` will enter a state that matches a specific pattern if triggered.
    pub fn will_enter<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (_, Some(y)) if pattern.matches(y))
    }

    /// Check if `S` will become enabled in a state that matches a specific pattern if triggered.
    pub fn will_enable<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (None, Some(y)) if pattern.matches(y))
    }

    /// Check if `S` will undergo a transition that matches a specific pattern if triggered.
    pub fn will_trans<P: StateTransPattern<S>>(&self, pattern: &P) -> bool {
        matches!(self.get(), (Some(x), Some(y)) if pattern.matches(x, y))
    }

    /// Disable the next state.
    pub fn disable(&mut self) {
        self.next.disable();
    }

    /// Enable the next state with a specific value if the current state is disabled.
    pub fn enable(&mut self, value: S) {
        if self.current.is_disabled() {
            self.enter(value);
        }
    }

    /// Set the next state to a toggle of the current state between disabled and enabled
    /// with a specific value.
    pub fn toggle(&mut self, value: S) {
        if self.current.is_disabled() {
            self.enter(value);
        } else {
            self.disable();
        }
    }

    /// Enable the next state with a specific value.
    pub fn enter(&mut self, value: S) {
        self.next.enter(value);
    }

    /// Trigger `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) -> &mut Self {
        self.next.trigger();
        self
    }

    /// Reset the trigger for `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn reset_trigger(&mut self) -> &mut Self {
        self.next.reset_trigger();
        self
    }
}

impl<S: StateMut + Default> FlushMut<'_, '_, S> {
    /// Enable the next state with the default value if the current state is disabled.
    pub fn enable_default(&mut self) {
        if self.current.is_disabled() {
            self.enter(S::default())
        }
    }

    /// Set the next state to a toggle of the current state between disabled and enabled
    /// with the default value.
    pub fn toggle_default(&mut self) {
        if self.current.is_disabled() {
            self.enter_default();
        } else {
            self.disable();
        }
    }

    /// Enable the next state with the default value.
    pub fn enter_default(&mut self) {
        self.next.enter_default();
    }
}
