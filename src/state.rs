//! TODO: Module-level documentation
//!
//! You can write custom systems and run conditions using the following
//! [`SystemParam`s](bevy_ecs::system::SystemParam):
//!
//! | State          | Read-only                              | Mutable                                   |
//! | -------------- | -------------------------------------- | ----------------------------------------- |
//! | Current        | [`Res<CurrentState<S>>`](CurrentState) | [`ResMut<CurrentState<S>>`](CurrentState) |
//! | Next           | [`NextStateRef<S>`]                    | [`NextStateMut<S>`]                       |
//! | Current + Next | [`StateFlushRef<S>`]                   | [`StateFlushMut<S>`]                      |

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::system::{Res, ResMut, Resource, StaticSystemParam, SystemParam};

#[cfg(feature = "bevy_state")]
use bevy_state::state::States;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;

use crate::{
    pattern::{
        AnyStatePattern, AnyStateTransPattern, FnStatePattern, FnStateTransPattern, StatePattern,
        StateTransPattern,
    },
    storage::{StateStorage, StateStorageMut},
};

/// A data type that can be used as a state.
///
/// The current state will be stored in the [`CurrentState`] resource,
/// and the next state will be stored in the specified [`StateStorage`].
///
/// This trait can be [derived](pyri_state_derive::State) or implemented manually:
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// enum GameState { ... }
///
/// enum MenuState { ... }
/// impl State for MenuState {
///     type Storage = StateBuffer<Self>;
/// }
/// ```
///
/// The derive macro would also implement [`AddState`](crate::app::AddState) for `MenuState`.
///
/// See the following extension traits with additional bounds on `Self` and [`Self::Storage`](State::Storage):
///
/// - [`StateMut`]
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait State: 'static + Send + Sync + Sized {
    /// The [`StateStorage`] type that describes how this state will be stored in the ECS world.
    type Storage: StateStorage<Self>;

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
    fn is_disabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_disabled()
    }

    /// A run condition that checks if the current state is enabled.
    fn is_enabled(state: Res<CurrentState<Self>>) -> bool {
        state.is_enabled()
    }

    /// A run condition that checks if the next state will be disabled.
    fn will_be_disabled(next: NextStateRef<Self>) -> bool {
        next.get().is_none()
    }

    /// A run condition that checks if the next state will be enabled.
    fn will_be_enabled(next: NextStateRef<Self>) -> bool {
        next.get().is_some()
    }

    /// A system that triggers this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn trigger(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.trigger();
    }

    /// A system that resets the trigger for this state type to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    fn relax(mut trigger: ResMut<TriggerStateFlush<Self>>) {
        trigger.relax();
    }
}

/// An extension trait for [`State`] types with [mutable storage](StateStorageMut).
///
/// See the following extension traits with additional bounds on `Self`:
///
/// - [`StateMutExtClone`]
/// - [`StateMutExtDefault`]
pub trait StateMut: State {
    /// This is the same type as `<Self as State>::Storage` but with an additional [`StateStorageMut`] bound.
    type StorageMut: StateStorageMut<Self>;

    /// A system that disables the next state.
    fn disable(mut state: NextStateMut<Self>) {
        state.set(None);
    }
}

/// An extension trait for [`StateMut`] types that also implement [`Clone`].
pub trait StateMutExtClone: StateMut + Clone {
    /// Build a system that enables the next state with a specific value if it's disabled.
    fn enable(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            }
        }
    }

    /// Build a system that toggles the next state between disabled and enabled with a specific value.
    fn toggle(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            if state.will_be_disabled() {
                state.enter(self.clone());
            } else {
                state.disable();
            }
        }
    }

    /// Build a system that enables the next state with a specific value.
    fn enter(self) -> impl Fn(NextStateMut<Self>) + 'static + Send + Sync {
        move |mut state| {
            state.set(Some(self.clone()));
        }
    }

    /// A system that resets the next state to the current state and relaxes the trigger to flush.
    fn reset(mut state: StateFlushMut<Self>) {
        state.reset();
    }

    /// A system that resets the next state to the current state and triggers a flush.
    fn refresh(mut state: StateFlushMut<Self>) {
        state.refresh();
    }
}

impl<S: StateMut + Clone> StateMutExtClone for S {}

/// An extension trait for [`StateMut`] types that also implement [`Default`].
pub trait StateMutExtDefault: StateMut + Default {
    /// A system that enables the next state with the default value if it's disabled.
    fn enable_default(mut state: NextStateMut<Self>) {
        state.enable_default();
    }

    /// A system that toggles the next state between disabled and enabled with the default value.
    fn toggle_default(mut state: NextStateMut<Self>) {
        state.toggle_default();
    }

    /// A system that enables the next state with the default value.
    fn enter_default(mut state: NextStateMut<Self>) {
        state.enter_default();
    }
}

impl<S: StateMut + Default> StateMutExtDefault for S {}

/// A resource that contains the current value of the [`State`] type `S`.
///
/// Use [`StateFlushRef`] or [`StateFlushMut`] in a system to access the next state alongside
/// the current state.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct CurrentState<S: State>(
    /// The current state, or `None` if disabled.
    pub Option<S>,
);

impl<S: State> Default for CurrentState<S> {
    fn default() -> Self {
        Self::disabled()
    }
}

impl<S: State> CurrentState<S> {
    /// Create a disabled `CurrentState`.
    pub fn disabled() -> Self {
        Self(None)
    }

    /// Create an enabled `CurrentState` with a specific value.
    pub fn enabled(value: S) -> Self {
        Self(Some(value))
    }

    /// Get a read-only reference to the current state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        self.0.as_ref()
    }

    /// Get a read-only reference to the current state, or panic if it's disabled.
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

/// A resource that determines whether the [`State`] type `S` will flush in the
/// [`StateFlush`](crate::schedule::StateFlush) schedule.
#[derive(Resource, Debug)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct TriggerStateFlush<S: State>(
    /// The flush flag. If true, `S` will flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub bool,
    PhantomData<S>,
);

impl<S: State> Default for TriggerStateFlush<S> {
    fn default() -> Self {
        Self(false, PhantomData)
    }
}

impl<S: State> TriggerStateFlush<S> {
    /// Trigger `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn trigger(&mut self) {
        self.0 = true;
    }

    /// Reset the trigger for `S` to flush in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    pub fn relax(&mut self) {
        self.0 = false;
    }
}

/// A [`SystemParam`] with read-only access to the next value of the [`State`] type `S`.
///
/// NOTE: The next state is only set in stone during the [`StateFlush`](crate::schedule::StateFlush)
/// schedule after [`StateFlushSet::<S>::Compute`](crate::schedule::StateFlushSet::Compute).
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
pub struct NextStateRef<'w, 's, S: State>(
    StaticSystemParam<'w, 's, <<S as State>::Storage as StateStorage<S>>::Param>,
);

impl<'w, 's, S: State> NextStateRef<'w, 's, S> {
    /// Get a read-only reference to the next state, or `None` if disabled.
    pub fn get(&self) -> Option<&S> {
        S::Storage::get_state(&self.0)
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
/// schedule after [`StateFlushSet::<S>::Compute`](crate::schedule::StateFlushSet::Compute).
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
    next: StaticSystemParam<'w, 's, <<S as StateMut>::StorageMut as StateStorageMut<S>>::ParamMut>,
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
        S::StorageMut::get_state_from_mut(&self.next)
    }

    /// Get a mutable reference to the next state, or `None` if disabled.
    pub fn get_mut(&mut self) -> Option<&mut S> {
        S::StorageMut::get_state_mut(&mut self.next)
    }

    /// Set the next state to a new value, or `None` to disable.
    pub fn set(&mut self, state: Option<S>) {
        S::StorageMut::set_state(&mut self.next, state)
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
/// schedule after [`StateFlushSet::<S>::Compute`](crate::schedule::StateFlushSet::Compute).
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
/// schedule after [`StateFlushSet::<S>::Compute`](crate::schedule::StateFlushSet::Compute).
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

/// A wrapper around the [`State`] type `S` for compatibility with the Bevy ecosystem.
///
/// Any change to `S` will propagate to `BevyState<S>`, and vice versa.
///
/// # Example
///
/// Include [`BevyStatePlugin<S>`](crate::app::BevyStatePlugin) for `GameState`:
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
/// #[state(bevy_state)]
/// enum GameState {
///     #[default]
///     Title,
///     Loading,
///     Playing,
/// }
/// ```
///
/// Add `GameState` along with its `BevyState` wrapper:
///
/// ```rust
/// app.init_state::<GameState>();
/// ```
///
/// Use `GameState` to drive `BevyState`:
///
/// ```rust
/// app.add_systems(Update, GameState::Title.on_update(
///     GameState::Loading.enter().run_if(input_just_pressed(KeyCode::Enter)),
/// ));
/// ```
///
/// Use `BevyState` to drive `GameState` (using `iyes_progress`):
///
/// ```rust
/// app.add_plugins(
///     ProgressPlugin::new(BevyState(Some(GameState::Loading)))
///         .continue_to(BevyState(Some(GameState::Playing))),
/// );
/// ```
#[cfg(feature = "bevy_state")]
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BevyState<S: State + Clone + PartialEq + Eq + Hash + Debug>(
    /// The wrapped state value, or `None` if disabled.
    pub Option<S>,
);

#[cfg(feature = "bevy_state")]
impl<S: State + Clone + PartialEq + Eq + Hash + Debug> Default for BevyState<S> {
    fn default() -> Self {
        Self(None)
    }
}
