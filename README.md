`pyri_state` is an experimental 3rd-party alternative to `bevy_state`. In `pyri_state`, states are simple double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

# Features

- Ergonomics
- [Partial mutation](#partial-mutation)
- [Flexible scheduling](#flexible-scheduling)
- [Modular configuration](#modular-configuration)
- [Ecosystem compatibility](#ecosystem-compatibility)
- [Refresh](#refresh)
- [Disable, enable, toggle](#disable-enable-toggle)
- [Computed & substates](#computed--substates)

## Partial mutation

Directly update the next state instead of setting an entirely new value:

```rust
// Player has different abilities depending on the color mode. For example,
// yellow mode is its own thing, not just red and green modes at the same time.
#[derive(State, Clone, PartialEq, Eq, Default)]
struct ColorMode {
    r: bool,
    g: bool,
    b: bool,
}

fn enable_red(mut color: ResMut<NextState_<ColorMode>>) {
    color.unwrap_mut().r = true;
}

fn disable_red(mut color: ResMut<NextState_<ColorMode>>) {
    color.unwrap_mut().r = false;
}

fn toggle_green(mut color: ResMut<NextState_<ColorMode>>) {
    let color = color.unwrap_mut();
    color.g = !color.g;
}

fn toggle_blue(mut color: ResMut<NextState_<ColorMode>>) {
    let color = color.unwrap_mut();
    color.b = !color.b;
}

.init_state_::<ColorMode>()
.add_systems(
    Update,
    // These systems might run on the same frame sometimes.
    // With partial mutation, that's totally fine and expected.
    ColorMode::ANY.on_update(
        (disable_red.run_if(took_damage), enable_red.run_if(dealt_damage)).chain(),
        toggle_green.run_if(input_just_pressed(KeyCode::Space)),
        toggle_blue.run_if(on_timer(Duration::from_secs(5))),
    ),
)
```

## Flexible scheduling

Harness the full power of bevy ECS to schedule your state transitions:

```rust
#[derive(State, Clone, PartialEq, Eq)]
struct Level(usize);

.add_state_::<Level>()
.add_systems(
    StateFlush,
    Level::ANY.on_enter((
        play_boss_music.run_if(Level(10).will_enter()),
        spawn_tutorial_popup.run_if(Level::with(|level| level.0 < 4).will_enter()),
        spawn_easter_egg.run_if(|level: StateRef<Level>| matches!(
            level.get(),
            (Some(Level(x @ 2 | 5..8)), Some(Level(y))) if x * x > y,
        )),
        gen_level.run_if(|level: Res<NextState_<Level>>, meta: Res<LevelMeta>| {
            !meta.has_been_generated(level.unwrap().0)
        }),
        load_level.after(gen_level),
    )),
);
```

## Modular configuration

Strip out or add features to your states on a per-type basis:

```rust
#[derive(State, PartialEq, Eq, Clone, Hash, Debug)]
#[state(
    // Disable default configs: detect_change, send_event, apply_flush.
    no_defaults,
    // Trigger a flush on any state change.
    detect_change,
    // Send a flush event on flush.
    send_event,
    // Include a BevyState wrapper (see Ecosystem compatibility).
    bevy_state,
    // Clone the next state into the current state on flush.
    apply_flush,
    // Run this state's on flush systems after the listed states.
    after(FooState, BarState<i32>),
    // Run this state's on flush systems before the listed states.
    before(QuuxState)
)]
struct MyCustomState(i32);

// Derived traits can be omitted if they won't be used:
#[derive(State)]
#[state(no_defaults)]
struct MyRawState(i32);

.add_state_::<MyCustomState>()
.add_state_::<MyRawState>()
```

## Ecosystem compatibility

Easily enable an associated `BevyState<S>` wrapper to interact with crates that expect it:

```rust
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;

#[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
// Set up `BevyState<GameState>` by enabling this config:
#[state(bevy_state)]
enum GameState {
    #[default]
    Splash,
    Title,
    LoadingGame,
    PlayingGame,
}

.init_state_::<GameState>()
.add_loading_state(
    LoadingState::new(BevyState(Some(GameState::LoadingGame)))
        .load_collection::<GameAssets>(),
)
.add_plugins(
    ProgressPlugin::new(BevyState(Some(GameState::LoadingGame)))
        // Changes to BevyState<GameState> will propagate to GameState.
        .continue_to(Some(GameState::PlayingGame)),
)
.add_systems(
    Update,
    GameState::Title.on_update(
        // Changes to GameState will propagate to BevyState<GameState>.
        GameState::LoadingGame.enter().run_if(input_just_pressed(KeyCode::Enter)),
    ),
)
```

## Refresh

Trigger a transition from the current state to itself (e.g. to restart the current level):

```rust
#[derive(State, Clone, PartialEq, Eq, Default)]
struct LevelIdx(usize);

fn tear_down_old_level(level: Res<CurrentState<LevelIdx>>) { ... }
fn set_up_new_level(level: Res<NextState_<LevelIdx>>) { ... }

.init_state_::<LevelIdx>()
.add_systems(
    StateFlush,
    (
        LevelIdx::ANY.on_exit(tear_down_old_level),
        LevelIdx::ANY.on_enter(set_up_new_level),
    )
)
.add_systems(
    Update,
    // Restarts the current level on R press:
    LevelIdx::refresh.run_if(input_just_pressed(KeyCode::KeyR)),
);
```

## Disable, enable, toggle

Disable or enable any state type on command (great for simple on/off states or substates):

```rust
#[derive(State, Clone, PartialEq, Eq, Default)]
struct Paused;

fn unpause() { ... }
fn pause() { ... }

.add_state_::<Paused>()
.add_systems(
    StateFlush,
    Paused.on_exit(unpause),
    Paused.on_enter(pause),
)
.add_systems(
    Update,
    (
        Paused::enable.run_if(window_lost_focus),
        Paused::disable.run_if(window_gained_focus),
        Paused::toggle.run_if(input_just_pressed(KeyCode::Escape)),
    ),
)
```

## Computed & substates

Roll your own computed and substates with the full power of bevy ECS:

```rust
#[derive(State, Clone, PartialEq, Eq)]
enum GameState {
    Splash,
    Title,
    Playing,
}

// Substate of GameState::Playing
#[derive(State, Clone, PartialEq, Eq, Default)]
#[after(GameState)]
struct CheckerboardSquare {
    row: u8,
    col: u8,
}

// Computed from CheckerboardSquare
#[derive(State, Clone, PartialEq, Eq)]
#[after(CheckerboardSquare)]
enum SquareColor {
    Black,
    White,
}

fn compute_square_color(
    board: Res<NextState_<CheckerboardSquare>>,
    mut color: ResMut<NextState_<ColorState>>,
) {
    color.inner = board.get().map(|board| {
        if board.row + board.col % 2 == 0 {
            SquareColor::Black
        } else {
            SquareColor::White
        }
    });
}

.init_state_::<GameState>()
.add_state_::<CheckerboardSquare>()
.add_state_::<SquareColor>()
.add_systems(
    StateFlush,
    (
        GameState::Playing.on_exit(CheckerboardSquare::disable),
        GameState::Playing.on_enter(CheckerboardSquare::enable),
        CheckerboardSquare::ANY.on_enter(compute_square_color),
    )
);
```

# Remaining tasks

- [ ] Unit tests
- [ ] Documentation
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
