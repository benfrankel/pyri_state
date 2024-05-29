// Strip out or add plugins to your state type via derive macro.

use bevy::prelude::*;
use pyri_state::{
    app::{
        AddState, AddStateStorage, ApplyFlushPlugin, BevyStatePlugin, DetectChangePlugin,
        FlushEventPlugin, ResolveStatePlugin,
    },
    prelude::*,
    state::FlushState,
    storage::stack::StateStack,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .add_state_::<MyRawState>()
        .add_state_::<MyDerivedState>()
        .add_state_::<MyCustomState>()
        .run();
}

// You can derive State on its own if no other traits are required.
#[derive(State)]
#[state(no_defaults)]
struct MyRawState;

#[derive(State, PartialEq, Eq, Clone, Hash, Debug)]
// State plugins are fully customizable per state type:
#[state(
    // Disable default plugins: detect_change, flush_event, apply_flush.
    no_defaults,
    // Trigger a flush on any state change (requires PartialEq, Eq).
    detect_change,
    // Send an event on flush (requires Clone).
    flush_event,
    // Include a BevyState wrapper (requires Clone, PartialEq, Eq, Hash, Debug).
    // (see `ecosystem_compatibility` example for more information)
    bevy_state,
    // Clone the next state into the current state on flush (requires Clone).
    apply_flush,
    // Swap out the default `StateSlot<Self>` with a custom storage type.
    // (see `custom_storage` example for more information)
    storage(StateStack<Self>),
    // Run this state's on flush systems after the listed states.
    after(MyRawState),
    // Run this state's on flush systems before the listed states.
    before(MyCustomState, DummyState)
)]
struct MyDerivedState;

// Deriving RawState instead of State allows you to impl AddState yourself,
// allowing for fully custom state configuration (see below).
#[derive(RawState, Clone, PartialEq, Eq, Hash, Debug)]
struct MyCustomState;

// This will be called from `app.add_state_`, `init_state_`, and `insert_state_`.
impl AddState for MyCustomState {
    fn add_state(app: &mut App, value: Option<Self>) {
        Self::Storage::add_state_storage(app, value);
        app.init_resource::<CurrentState<Self>>()
            .init_resource::<FlushState<Self>>()
            .add_plugins((
                ResolveStatePlugin::<Self>::default()
                    .after::<MyRawState>()
                    .after::<MyDerivedState>()
                    .before::<DummyState>(),
                DetectChangePlugin::<Self>::default(),
                FlushEventPlugin::<Self>::default(),
                BevyStatePlugin::<Self>::default(),
                ApplyFlushPlugin::<Self>::default(),
            ));

        // ... some more custom configuration on app ...
    }
}

#[derive(RawState)]
struct DummyState;
