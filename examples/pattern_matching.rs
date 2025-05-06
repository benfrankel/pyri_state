//! Use pattern-matching and run conditions to schedule state flush hooks.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use pyri_state::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .insert_resource(StateDebugSettings {
            log_flush: true,
            ..default()
        })
        .init_state::<Level>()
        .init_resource::<LevelMeta>()
        .add_systems(
            StateFlush,
            (
                // Schedule the basic level teardown / setup.
                Level::ANY.on_edge(tear_down_old_level, load_new_level),
                // Level 10 is the final boss fight, so play boss music.
                Level(10).on_enter(play_boss_music),
                // Levels 4, 7, and 10 are checkpoints, so save progress.
                state!(Level(4 | 7 | 10)).on_enter(save_checkpoint),
                // Even numbered levels introduce new game mechanics.
                Level::with(|x| x.0 % 2 == 0).on_enter(spawn_tutorial_popup),
                // Spawn an easter egg for very specific level transitions.
                state!(Level(x @ (2 | 5..=8)) => y if y.0 == 10 - x).on_enter(spawn_easter_egg),
                // Play a sound effect when progressing to a later level.
                Level::when(|x, y| y.0 > x.0).on_enter(play_progress_sfx),
                // Randomly generate the next level before loading it, if necessary.
                Level::ANY.on_enter(generate_new_level.before(load_new_level).run_if(
                    |level: NextRef<Level>, meta: Res<LevelMeta>| !meta.generated[level.unwrap().0],
                )),
            ),
        )
        .add_systems(
            Update,
            (
                Level(1).enter().run_if(input_just_pressed(KeyCode::Digit1)),
                Level(2).enter().run_if(input_just_pressed(KeyCode::Digit2)),
                Level(3).enter().run_if(input_just_pressed(KeyCode::Digit3)),
                Level(4).enter().run_if(input_just_pressed(KeyCode::Digit4)),
                Level(5).enter().run_if(input_just_pressed(KeyCode::Digit5)),
                Level(6).enter().run_if(input_just_pressed(KeyCode::Digit6)),
                Level(7).enter().run_if(input_just_pressed(KeyCode::Digit7)),
                Level(8).enter().run_if(input_just_pressed(KeyCode::Digit8)),
                Level(9).enter().run_if(input_just_pressed(KeyCode::Digit9)),
                Level(10)
                    .enter()
                    .run_if(input_just_pressed(KeyCode::Digit0)),
            ),
        )
        .run()
}

#[derive(State, Reflect, Clone, PartialEq, Eq, Debug)]
#[state(log_flush)]
#[reflect(Resource)]
struct Level(usize);

impl Default for Level {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct LevelMeta {
    generated: Vec<bool>,
}

impl Default for LevelMeta {
    fn default() -> Self {
        Self {
            generated: vec![false; 11],
        }
    }
}

fn tear_down_old_level(_level: CurrentRef<Level>) {
    info!("tear_down_old_level")
}

fn load_new_level(_level: NextRef<Level>) {
    info!("load_new_level")
}

fn play_boss_music() {
    info!("play_boss_music");
}

fn play_progress_sfx() {
    info!("play_progress_sfx");
}

fn save_checkpoint() {
    info!("save_checkpoint");
}

fn spawn_tutorial_popup() {
    info!("spawn_tutorial_popup");
}

fn spawn_easter_egg() {
    info!("spawn_easter_egg");
}

fn generate_new_level(level: NextRef<Level>, mut meta: ResMut<LevelMeta>) {
    info!("generate_new_level");
    meta.generated[level.unwrap().0] = true;
}
