//! Navigate a fixed sequence of states by index (e.g. for phases in a turn-based game).

use core::fmt::Debug;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        // Add the `Page` sequence.
        .insert_resource(NextStateSequence::new([
            None,
            Some(Page::A),
            Some(Page::B),
            Some(Page::C),
        ]))
        // Add the `Page` state, initially pointing to index 0 of the sequence.
        .init_state::<Page>()
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
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug)]
// Configure `Page` to use `NextStateIndex` instead of `NextStateBuffer` as its `NextState` type.
#[state(log_flush, next(NextStateIndex<Self>))]
#[reflect(Resource)]
enum Page {
    A,
    B,
    C,
}
