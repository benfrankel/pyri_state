// Strip out or add plugins to your state type using the derive macro.

use bevy::prelude::*;
use pyri_state::{
    app::{
        AddState, ApplyFlushPlugin, BevyStatePlugin, DetectChangePlugin, FlushEventPlugin,
        ResolveStatePlugin,
    },
    extra::stack::*,
    prelude::*,
    state::TriggerStateFlush,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .add_state_::<MyBasicState>()
        .add_state_::<MyDerivedState>()
        .add_state_::<MyCustomState>()
        .run();
}

// You can derive State on its own if no other traits are required.
#[derive(State)]
#[state(no_defaults)]
struct MyBasicState;

// The built-in state plugins can be configured:
#[derive(State, PartialEq, Eq, Clone, Hash, Debug)]
#[state(
    // Disable default plugins: detect_change, flush_event, apply_flush.
    no_defaults,
    // Trigger a flush on any state change (requires PartialEq, Eq).
    detect_change,
    // Send an event on flush (requires Clone).
    flush_event,
    // Log on exit, transition, and enter (requires Debug).
    log_flush,
    // Include a BevyState wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
    // (see `ecosystem_compatibility` example for more information)
    bevy_state,
    // Clone the next state into the current state on flush (requires Clone).
    apply_flush,
    // Swap out the default `StateBuffer<Self>` for a custom storage type.
    // (see `custom_storage` example for more information)
    storage(StateStack<Self>),
    // Run this state's on flush systems after resolving the listed states.
    after(MyBasicState),
    // Run this state's on flush systems before resolving the listed states.
    before(MyCustomState, UselessState),
)]
struct MyDerivedState;

// Skip the derive entirely to fully customize your state type (see below).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct MyCustomState;

impl State_ for MyCustomState {
    type Storage = StateBuffer<Self>;
}

// This will be called from `app.add_state_`, `init_state_`, and `insert_state_`.
impl AddState for MyCustomState {
    type AddStorage = Self::Storage;

    fn add_state(app: &mut App) {
        app.init_resource::<CurrentState<Self>>()
            .init_resource::<TriggerStateFlush<Self>>()
            .add_plugins((
                ResolveStatePlugin::<Self>::default()
                    .after::<MyBasicState>()
                    .after::<MyDerivedState>()
                    .before::<UselessState>(),
                DetectChangePlugin::<Self>::default(),
                FlushEventPlugin::<Self>::default(),
                BevyStatePlugin::<Self>::default(),
                ApplyFlushPlugin::<Self>::default(),
            ));

        // ... some more custom configuration on app ...
    }
}

// TODO: This is confusing.
// A fully stripped down state type that does nothing.
struct UselessState;

impl State_ for UselessState {
    type Storage = StateBuffer<Self>;
}
