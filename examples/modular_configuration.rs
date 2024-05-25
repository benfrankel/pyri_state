// Strip out or add plugins to your state type.

use bevy::prelude::*;
use pyri_state::{
    app::{
        ApplyFlushPlugin, BevyStatePlugin, ConfigureState, DetectChangePlugin, FlushEventPlugin,
        ResolveStatePlugin,
    },
    prelude::*,
};

#[derive(State, PartialEq, Eq, Clone, Hash, Debug)]
#[state(
            // Disable default plugins: detect_change, flush_event, apply_flush.
            no_defaults,
            // Trigger a flush on any state change.
            detect_change,
            // Send an event on flush.
            flush_event,
            // Include a BevyState wrapper (see ecosystem_compatibility example).
            bevy_state,
            // Clone the next state into the current state on flush.
            apply_flush,
            // Run this state's on flush systems after the listed states.
            after(FooState, BarState<i32>),
            // Run this state's on flush systems before the listed states.
            before(QuuxState)
        )]
struct MyDerivedState;

// Clone, PartialEq, and Eq can be omitted if they won't be needed.
#[derive(State)]
#[state(no_defaults)]
struct MyRawState;

// TODO: Explain this in comments
#[derive(RawState, Clone, PartialEq, Eq, Hash, Debug)]
struct MyCustomState;

impl ConfigureState for MyCustomState {
    fn configure(app: &mut App) {
        app.add_plugins((
            ResolveStatePlugin::<Self>::default()
                .after::<MyDerivedState>()
                .before::<MyRawState>(),
            DetectChangePlugin::<Self>::default(),
            FlushEventPlugin::<Self>::default(),
            BevyStatePlugin::<Self>::default(),
            ApplyFlushPlugin::<Self>::default(),
        ));

        // ... some more custom configuration ...
    }
}

fn main() {
    let mut app = App::new();
    app.add_state_::<MyDerivedState>()
        .add_state_::<MyRawState>()
        .add_state_::<MyCustomState>();
}
