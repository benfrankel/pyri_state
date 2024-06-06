// Navigate a fixed sequence of states by index (e.g. for phases in a turn-based game).

use std::fmt::Debug;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::{debug::StateDebugSettings, extra::sequence::*, prelude::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings::Enabled)
        // Add the `Page` state with the provided sequence.
        .insert_state_(StateSequence::new([
            None,
            Some(Page::A),
            Some(Page::B),
            Some(Page::C),
        ]))
        .add_systems(
            Update,
            // Set up page navigation.
            (
                Page::next.run_if(input_just_pressed(KeyCode::ArrowRight)),
                Page::prev.run_if(input_just_pressed(KeyCode::ArrowLeft)),
                Page::seek(0).run_if(input_just_pressed(KeyCode::Digit0)),
                Page::seek(1).run_if(input_just_pressed(KeyCode::Digit1)),
                Page::seek(2).run_if(input_just_pressed(KeyCode::Digit2)),
                Page::seek(3).run_if(input_just_pressed(KeyCode::Digit3)),
            ),
        )
        .run();
}

#[derive(State, Clone, PartialEq, Eq, Debug)]
// Configure `Page` to use `StateSequence` instead of `StateBuffer` for storage.
#[state(log_flush, storage(StateSequence<Self>))]
enum Page {
    A,
    B,
    C,
}
