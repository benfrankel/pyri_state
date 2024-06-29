// Strip out or add plugins to your state type.

use bevy::prelude::*;
use pyri_state::{
    debug::log_flush::LogFlushPlugin,
    extra::{app::RegisterState, bevy_state::BevyStatePlugin, entity_scope::EntityScopePlugin},
    prelude::*,
    schedule::{
        apply_flush::ApplyFlushPlugin, detect_change::DetectChangePlugin,
        flush_event::FlushEventPlugin, resolve_state::ResolveStatePlugin,
    },
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
#[derive(Resource, State, Clone, PartialEq, Eq)]
struct BasicState;

// They can be omitted if you disable the default options:
#[derive(Resource, State)]
#[state(no_defaults)]
struct RawState;

// The built-in state plugins can be configured:
#[derive(Resource, State, Clone, PartialEq, Eq, Hash, Debug)]
#[state(
    // Disable default plugins: detect_change, flush_event, apply_flush.
    no_defaults,
    // Trigger a flush on any state change (requires PartialEq, Eq).
    detect_change,
    // Send an event on flush (requires Clone).
    flush_event,
    // Log on flush (requires Debug).
    log_flush,
    // Include a BevyState wrapper (requires StateMut, Clone, PartialEq, Eq, Hash, Debug).
    // (see `ecosystem_compatibility` example for more information)
    bevy_state,
    // Despawn entities marked with `StateScope<Self>` on any exit.
    entity_scope,
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
struct DerivedState;

// Skip the derive entirely to fully customize your state type (see below).
#[derive(Resource, Clone, PartialEq, Eq, Hash, Debug)]
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
            EntityScopePlugin::<Self>::default(),
            ApplyFlushPlugin::<Self>::default(),
        ));

        // ... and more customization on `app` ...
    }
}
