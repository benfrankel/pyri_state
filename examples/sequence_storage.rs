// Navigate a fixed sequence of states by index (e.g. pages).

use std::fmt::Debug;

use bevy::{core::FrameCount, input::common_conditions::input_just_pressed, prelude::*};
use bevy_ecs::schedule::SystemConfigs;
use pyri_state::{prelude::*, storage::sequence::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        // Add the `Page` state using the provided sequence as storage.
        .insert_state_(StateSequence::new([
            None,
            Some(Page::A),
            Some(Page::B),
            Some(Page::C),
        ]))
        .add_systems(StateFlush, log_state_flush::<Page>())
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
// Configure `Page` to use `StateSequence` instead of `StateSlot` as storage.
#[state(storage(StateSequence<Self>))]
enum Page {
    A,
    B,
    C,
}

// TODO: Provide something like this in `pyri_state` itself.
pub fn log_state_flush<S: GetState + Debug>() -> SystemConfigs {
    (
        S::ANY.on_exit(log_state_exit::<S>),
        // TODO: The story for flush / transition handling is not great right now.
        S::on_transition(
            log_state_transition::<S>.run_if(S::ANY.will_exit().and_then(S::ANY.will_enter())),
        ),
        S::ANY.on_enter(log_state_enter::<S>),
    )
        .into_configs()
}

fn log_state_exit<S: GetState + Debug>(frame: Res<FrameCount>, old: Res<CurrentState<S>>) {
    let frame = frame.0;
    let old = old.unwrap();
    info!("[Frame {frame}] Exit: {old:?}");
}

fn log_state_transition<S: GetState + Debug>(frame: Res<FrameCount>, state: StateRef<S>) {
    let frame = frame.0;
    let (old, new) = state.unwrap();
    info!("[Frame {frame}] Transition: {old:?} -> {new:?}");
}

fn log_state_enter<S: GetState + Debug>(frame: Res<FrameCount>, new: NextStateRef<S>) {
    let frame = frame.0;
    let new = new.unwrap();
    info!("[Frame {frame}] Enter: {new:?}");
}
