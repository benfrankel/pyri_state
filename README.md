`pyri_state` is an experimental 3rd-party alternative to `bevy_state`. In `pyri_state`, states are simple double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

# Features

## Ergonomics

See example code below.

## Partial mutation

Mutate the next state directly instead of fully replacing it with a new value:

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
    ColorMode::on_any_update(
        // These systems might run on the same frame sometimes.
        // With partial mutation, that's totally fine and expected.
        enable_red.run_if(dealt_damage),
        disable_red.run_if(took_damage),
        toggle_green.run_if(input_just_pressed(KeyCode::Space)),
        toggle_blue.run_if(on_timer(Duration::from_secs(5))),
    ),
)
```

## Flexible scheduling

Configure systems to handle arbitrary state transitions using run conditions:

```rust
#[derive(State, Clone, PartialEq, Eq)]
struct LevelIdx(usize);

.add_state_::<LevelIdx>()
.add_systems(
    StateFlush,
    LevelIdx::on_change_and(
        |old, new| matches!(
            (old, new),
            (Some(LevelIdx(x @ 2 | 5..7)), Some(LevelIdx(y))) if x * x > y,
        ),
        spawn_easter_egg,
    ),
);
```

## Modular configuration

Configure the state flush behavior per state type:

```rust
#[derive(State, PartialEq, Eq, Clone, Hash, Debug)]
#[state(
    // Disable default configs: detect_change, send_event, apply_flush.
    no_defaults, 
    // Enable change detection to trigger a flush on any state change.
    detect_change, 
    // Send an event on flush.
    send_event,
    // Add a BevyState wrapper (see the next example).
    bevy_state,
    // Clone the next state into the current state on flush.
    apply_flush,
    // Run the on flush systems after some other states.
    after(FooState, BarState<i32>),
    // Run the on flush systems before some other states.
    before(QuuxState)
)]
struct MyCustomState(i32);

// Extra traits can be omitted if they won't be used:
#[derive(State)]
#[state(no_defaults)]
struct MyRawState(i32);

.add_state_::<MyCustomState>()
.add_state_::<MyRawState>()
```

## Ecosystem compatibility

Opt in to a `BevyState<S>` wrapper for compatibility with ecosystem crates:

```rust
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;

#[derive(State, Clone, PartialEq, Eq, Hash, Debug, Default)]
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
        GameState::LoadingGame.enter.run_if(input_just_pressed(KeyCode::Enter)),
    ),
)
```

## Refresh

Trigger a transition from the current state to itself:

```rust
#[derive(State, Clone, PartialEq, Eq, Default)]
struct LevelState(usize);

fn tear_down_old_level(level: Res<CurrentState<LevelState>>) { ... }
fn set_up_new_level(level: Res<NextState_<LevelState>>) { ... }

.init_state_::<LevelState>()
.add_systems(
    StateFlush,
    (
        LevelState::on_any_exit(tear_down_old_level),
        LevelState::on_any_enter(set_up_new_level),
    )
)
.add_systems(
    Update,
    // Restart the current level on R press:
    LevelState::refresh.run_if(input_just_pressed(KeyCode::KeyR)),
);
```

## Disable & enable

States can be disabled, enabled, and even toggled easily:

```rust
#[derive(State, Clone, PartialEq, Eq, Default)]
struct Paused;

fn toggle_pause() { ... }

.add_state_::<Paused>()
.add_systems(
    StateFlush,
    Paused.on_any_change(toggle_pause),
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

Roll your own computed and substates with the full power of bevy systems:

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
    x: u8,
    y: u8,    
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
        if board.x + board.y % 2 == 0 {
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
        CheckerboardSquare::on_any_enter(compute_square_color),
    )
);
```

# Remaining tasks

- [ ] Unit tests
- [ ] Documentation
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
