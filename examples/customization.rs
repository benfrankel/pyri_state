// Strip out or add plugins to your state type.

use bevy::prelude::*;
use pyri_state::{
    extra::{
        app::{
            AddState, ApplyFlushPlugin, DetectChangePlugin, FlushEventPlugin, ResolveStatePlugin,
        },
        bevy_state::BevyStatePlugin,
    },
    prelude::*,
    state::TriggerStateFlush,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .add_state::<BasicState>()
        .add_state::<RawState>()
        .add_state::<CustomState>()
        .run();
}

// The derive macro requires `Clone`, `PartialEq`, and `Eq` by default.`
#[derive(State, Clone, PartialEq, Eq)]
struct BasicState;

// They can be omitted if you disable the default options:
#[derive(State)]
#[state(no_defaults)]
struct RawState;

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
    // Run this state's on-flush hooks after the listed states.
    after(BasicState, RawState),
    // Run this state's on-flush hooks before the listed states.
    before(CustomState),
)]
struct DerivedState;

// Skip the derive entirely to fully customize your state type (see below).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct CustomState;

impl State for CustomState {
    type Storage = StateBuffer<Self>;
}

// This will be called from `app.add_state`, `init_state`, and `insert_state`.
impl AddState for CustomState {
    type AddStorage = Self::Storage;

    fn add_state(app: &mut App) {
        // You'll probably want to insert these resources:
        app.init_resource::<CurrentState<Self>>()
            .init_resource::<TriggerStateFlush<Self>>()
            // The derive macro's plugins can be added if desired:
            .add_plugins((
                ResolveStatePlugin::<Self>::default()
                    .after::<BasicState>()
                    .after::<RawState>()
                    .after::<DerivedState>(),
                DetectChangePlugin::<Self>::default(),
                FlushEventPlugin::<Self>::default(),
                BevyStatePlugin::<Self>::default(),
                ApplyFlushPlugin::<Self>::default(),
            ));

        // ... and more customization on `app` ...
    }
}
