//! Strip out or add plugins to your state type.

use bevy::prelude::*;
use pyri_state::{
    debug::log_flush::LogFlushPlugin,
    extra::{bevy_state::BevyStatePlugin, react::ReactPlugin},
    prelude::*,
    schedule::{
        apply_flush::ApplyFlushPlugin, detect_change::DetectChangePlugin,
        flush_event::FlushEventPlugin, resolve_state::ResolveStatePlugin,
    },
    setup::RegisterState,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .add_state::<BasicState>()
        .add_state::<RawState>()
        .add_state::<CustomState>()
        .run()
}

// The derive macro requires `Clone`, `PartialEq`, and `Eq` by default.
#[derive(State, Reflect, Clone, PartialEq, Eq)]
#[reflect(Resource)]
struct BasicState;

// They can be omitted if you disable the default options:
#[derive(State, Reflect)]
#[state(no_defaults)]
#[reflect(Resource)]
struct RawState;

// The built-in state plugins can be configured:
#[derive(State, Reflect, Clone, PartialEq, Eq, Hash, Debug)]
#[state(
    // Disable default plugins: detect_change, flush_event, apply_flush.
    no_defaults,
    // Trigger a flush on any state change (requires PartialEq, Eq).
    detect_change,
    // Send an event on flush (requires Clone).
    flush_event,
    // Log on flush (requires Debug).
    log_flush,
    // Include a `BevyState<Self>` wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
    // (see `ecosystem_compatibility` example for more information)
    bevy_state,
    // Enable reaction components such as `DespawnOnExitState<Self>` (requires Eq).
    react,
    // Clone the next state into the current state on flush (requires Clone).
    apply_flush,
    // Swap out the default `NextStateBuffer<Self>` for another `NextState` type.
    // (see `custom_next_state` example for more information)
    next(NextStateStack<Self>),
    // Run this state's on-flush hooks after the listed states.
    after(BasicState, RawState),
    // Run this state's on-flush hooks before the listed states.
    before(CustomState),
)]
#[reflect(Resource)]
struct DerivedState;

// Skip the derive entirely to fully customize your state type (see below).
#[derive(Resource, Reflect, Clone, PartialEq, Eq, Hash, Debug)]
#[reflect(Resource)]
struct CustomState;

impl State for CustomState {
    type Next = NextStateBuffer<Self>;
}

// This will be called from `app.add_state`, `init_state`, and `insert_state`.
impl RegisterState for CustomState {
    fn register_state(app: &mut App) {
        // Plugins from the derive macro can still be added if desired:
        app.add_plugins((
            ResolveStatePlugin::<Self>::default()
                .after::<BasicState>()
                .after::<RawState>()
                .after::<DerivedState>(),
            DetectChangePlugin::<Self>::default(),
            FlushEventPlugin::<Self>::default(),
            LogFlushPlugin::<Self>::default(),
            BevyStatePlugin::<Self>::default(),
            ReactPlugin::<Self>::default(),
            ApplyFlushPlugin::<Self>::default(),
        ));

        // ... and more customization on `app` ...
    }
}
