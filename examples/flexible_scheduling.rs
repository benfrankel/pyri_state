// Harness the full power of bevy ECS to schedule your state transitions.

use bevy::prelude::*;
use pyri_state::{prelude::*, state, will_flush};

fn main() {
    let mut app = App::new();
    app.add_plugins(PyriStatePlugin)
        .add_state_::<Level>()
        .init_resource::<LevelMeta>()
        // Upon exiting any level, remove its entities and resources.
        .add_systems(StateFlush, Level::ANY.on_exit(tear_down_level))
        .add_systems(
            StateFlush,
            // Upon entering any level, do the following:
            Level::ANY.on_enter((
                //
                // Level 10 is the final boss fight, so play boss music.
                play_boss_music.run_if(Level(10).will_enter()),
                //
                // Levels 4, 7, and 10 are checkpoints, so save progress.
                save_progress.run_if(state!(Level(4 | 7 | 10)).will_enter()),
                //
                // Early levels (0, 1, 2, and 3) introduce the player to the game.
                spawn_tutorial_popup.run_if(Level::with(|x| x.0 < 4).will_enter()),
                //
                // Spawn an easter egg for very specific level transitions.
                spawn_easter_egg.run_if(will_flush!(
                    (Some(Level(x @ (2 | 5..=8))), Some(&Level(y))) if y == 10 - x,
                )),
                //
                // Levels are randomly generated, but they should only be generated once.
                gen_level.run_if(|level: Res<NextState_<Level>>, meta: Res<LevelMeta>| {
                    !meta.generated[level.unwrap().0]
                }),
                //
                // Load the next level after it's been generated.
                load_level.after(gen_level),
            )),
        );
}

#[derive(State, Clone, PartialEq, Eq)]
struct Level(usize);

#[derive(Resource, Default)]
struct LevelMeta {
    generated: Vec<bool>,
}

// Dummy systems:
fn tear_down_level(_level: Res<CurrentState<Level>>) {}
fn play_boss_music() {}
fn save_progress() {}
fn spawn_tutorial_popup() {}
fn spawn_easter_egg() {}
fn gen_level(_level: Res<NextState_<Level>>) {}
fn load_level(_level: Res<NextState_<Level>>) {}
