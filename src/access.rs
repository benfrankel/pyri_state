//! TODO: Module-level docs
//!
//! Use the following [`SystemParam`] types to access the [`State`] type `S` in your systems
//! and run conditions:
//!
//! | State          | Read-only access                       | Mutable access                             |
//! | -------------- | -------------------------------------- | ------------------------------------------ |
//! | Current        | [`Res<CurrentState<S>>`](CurrentState) | [`ResMut<CurrentState<S>>`](CurrentState)* |
//! | Next           | [`NextStateRef<S>`]                    | [`NextStateMut<S>`]                        |
//! | Current & Next | [`StateFlushRef<S>`]                   | [`StateFlushMut<S>`]                       |
//!
//! \* Don't mutate the current state directly unless you know what you're doing.

use bevy_ecs::{
    component::Component,
    system::{Res, ResMut, StaticSystemParam, SystemParam},
};

use crate::{
    pattern::{StatePattern, StateTransPattern},
    state::{CurrentState, State, StateMut, StateStorage, StateStorageMut, TriggerStateFlush},
};

/// A marker [`Component`] for the global states entity spawned by
/// [`StatePlugin`](crate::extra::app::StatePlugin).
#[derive(Component, Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct GlobalStates;

/// A [`SystemParam`] with read-only access to the next value of the [`State`] type `S`.
///
/// NOTE: The next state is only set in stone during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`StateHook::<S>::Compute`](crate::schedule::StateHook::Compute).
///
/// # Example
///
/// ```rust
/// fn spawn_new_menu(menu: NextStateRef<Menu>) {
///     match menu.unwrap() {
///         Menu::Main => spawn_main_menu(),
///         Menu::Settings => spawn_settings_menu(),
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct NextStateRef<'w, 's, S: State> {
    res: Res<'w, <S as State>::Storage>,
    param: StaticSystemParam<'w, 's, <<S as State>::Storage as StateStorage>::Param>,
}

impl<'w, 's, S: State> NextStateRef<'w, 's, S> {
    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        S::Storage::get_state(&self.res, &self.param)
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
}

/// A [`SystemParam`] with mutable access to the next value of the [`State`] type `S`.
///
/// NOTE: The next state should not be mutated during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`StateHook::<S>::Compute`](crate::schedule::StateHook::Compute).
///
/// # Example
///
/// ```rust
/// fn toggle_blue(mut color: NextStateMut<ColorState>) {
///     let mut color = color.unwrap_mut();
///     color.blue = !color.blue;
/// }
/// ```
#[derive(SystemParam)]
pub struct NextStateMut<'w, 's, S: StateMut> {
    res: ResMut<'w, <S as State>::Storage>,
    param: StaticSystemParam<'w, 's, <<S as State>::Storage as StateStorageMut>::ParamMut>,
    trigger: ResMut<'w, TriggerStateFlush<S>>,
}

impl<'w, 's, S: StateMut + Default> NextStateMut<'w, 's, S> {
    /// Enable the next state with the default value if it's disabled.
    pub fn enable_default(&mut self) {
        if self.will_be_disabled() {
            self.enter(S::default())
        }
    }

    /// Toggle the next state between disabled and enabled with the default value.
    pub fn toggle_default(&mut self) {
        if self.will_be_disabled() {
            self.enable_default();
        } else {
            self.disable();
        }
    }

    /// Enable the next state with the default value.
    pub fn enter_default(&mut self) {
        self.enter(S::default());
    }
}

impl<'w, 's, S: StateMut> NextStateMut<'w, 's, S> {
    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        S::Storage::get_state_from_mut(&self.res, &self.param)
    }

    /// Get a mutable reference to the next state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        S::Storage::get_state_mut(&mut self.res, &mut self.param)
    }

    /// Set the next state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        S::Storage::set_state(&mut self.res, &mut self.param, state)
    }

    /// Get a read-only reference to the next state, or panic if it's disabled.
    pub fn unwrap(&self) -> &S {
        self.get().unwrap()
    }

    /// Get a mutable reference to the next state, or panic if it's disabled.
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

    /// Trigger `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) -> &mut Self {
        self.trigger.trigger();
        self
    }

    /// Reset the trigger for `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn relax(&mut self) -> &mut Self {
        self.trigger.relax();
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
        if self.will_be_enabled() {
            self.disable();
        } else {
            self.enter(value);
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
/// schedule after [`StateHook::<S>::Compute`](crate::schedule::StateHook::Compute).
///
/// # Example
///
/// ```rust
/// fn same_red(color: StateFlushRef<ColorState>) -> bool {
///     color.will_trans(&ColorState::when(|x, y| x.red == y.red))
/// }
/// ```
#[derive(SystemParam)]
pub struct StateFlushRef<'w, 's, S: State> {
    /// A system parameter with read-only access to the current state.
    pub current: Res<'w, CurrentState<S>>,
    /// A system parameter with read-only access to the next state.
    pub next: NextStateRef<'w, 's, S>,
}

impl<'w, 's, S: State + Eq> StateFlushRef<'w, 's, S> {
    /// Check if `S` will refresh in a state that matches a specific pattern if triggered.
    pub fn will_refresh<P: StatePattern<S>>(&self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: State> StateFlushRef<'w, 's, S> {
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
/// schedule after [`StateHook::<S>::Compute`](crate::schedule::StateHook::Compute).
///
/// # Example
///
/// ```rust
/// fn copy_red_to_green(mut color: StateFlushMut<ColorState>) {
///     let (old, new) = color.unwrap_mut();
///     new.green = old.red;
/// }
/// ```
#[derive(SystemParam)]
pub struct StateFlushMut<'w, 's, S: StateMut> {
    /// A system parameter with read-only access to the current state.
    pub current: Res<'w, CurrentState<S>>,
    /// A system parameter with mutable access to the next state.
    pub next: NextStateMut<'w, 's, S>,
}

impl<'w, 's, S: StateMut + Clone> StateFlushMut<'w, 's, S> {
    /// Set the next state to remain in the current state with no flush.
    pub fn reset(&mut self) {
        self.next.relax().set(self.current.0.clone());
    }

    /// Set the next state to flush to the current state.
    pub fn refresh(&mut self) {
        self.next.trigger().set(self.current.0.clone());
    }
}

impl<'w, 's, S: StateMut + Eq> StateFlushMut<'w, 's, S> {
    /// Check if `S` will refresh in a state that matches a specific pattern if triggered.
    pub fn will_refresh<P: StatePattern<S>>(&mut self, pattern: &P) -> bool {
        matches!(
            self.get(),
            (Some(x), Some(y)) if x == y && pattern.matches(y),
        )
    }
}

impl<'w, 's, S: StateMut> StateFlushMut<'w, 's, S> {
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

    /// Enable the next state with a specific value if it's disabled.
    pub fn enable(&mut self, value: S) {
        self.next.enable(value);
    }

    /// Toggle the next state between disabled and enabled with a specific value.
    pub fn toggle(&mut self, value: S) {
        self.next.toggle(value);
    }

    /// Enable the next state with a specific value.
    pub fn enter(&mut self, value: S) {
        self.next.set(Some(value));
    }

    /// Trigger `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) -> &mut Self {
        self.next.trigger();
        self
    }

    /// Reset the trigger for `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn relax(&mut self) -> &mut Self {
        self.next.relax();
        self
    }
}
