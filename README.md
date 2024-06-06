# Flexible game states

`pyri_state` is a flexible alternative to `bevy_state`. In `pyri_state`, states are double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

The next state is stored in a simple `StateBuffer` by default, but this can be swapped for a `StateStack`, a fixed `StateSequence`, or any custom data structure with a single line of code.

# Showcase

```rust
use bevy::prelude::*;
use pyri_state::{prelude::*, state};

#[derive(State, Clone, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

#[derive(State, Clone, PartialEq, Eq, Default)]
struct Level(usize);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StatePlugin))
        .init_state_::<GameState>()
        .add_state_::<Level>()
        .add_systems(
            StateFlush,
            (
                GameState::Playing.on_edge(Level::disable, Level::enable_default),
                Level::ANY.on_edge(tear_down_old_level, set_up_new_level),
                Level(10).on_enter(play_boss_music),
                state!(Level(4 | 7 | 10)).on_enter(save_progress),
                Level::with(|x| x.0 < 4).on_enter(spawn_tutorial_popup),
            ),
        );
}
```

# Features

Click a feature to see example code.

- **[Refresh](/examples/refresh.rs):** Trigger a transition from the current state to itself (e.g. to restart the current level).
- **[Disable, enable, toggle](/examples/disable_enable_toggle.rs):** Disable or enable any state on command (e.g. for simple toggle states and substates).
- **[Partial mutation](/examples/partial_mutation.rs):** Directly update the next state instead of setting an entirely new value.
- **[Custom storage](/examples/custom_storage.rs):** Swap out or define your own state storage type.
    - **Buffer:** Store a single state that can be mutated directly. This is the default storage type.
    - **[Stack](/examples/stack_storage.rs):** Keep track of a state's history in a stack (e.g. for a back button).
    - **[Sequence](/examples/sequence_storage.rs):** Navigate a fixed sequence of states by index (e.g. for phases in a turn-based game).
- **[Flexible scheduling](/examples/flexible_scheduling.rs):** Use state pattern matching and run conditions to schedule your systems.
- **[Computed & substates](/examples/computed_and_substates.rs):** Compute states from anything in the ECS world, including other states.
- **[Modular configuration](/examples/modular_configuration.rs):** Strip out or add plugins to your state type using the derive macro.
- **[Ecosystem compatibility](/examples/ecosystem_compatibility.rs):** Enable a `BevyState<S>` wrapper to interact with crates that expect it.
    
And some extra features:

- **[Split state](/examples/split_state.rs):** Split the definition of a basic state enum between the modules of your crate.

# Bevy version compatibility

| `bevy` version | `pyri_state` version |
| -------------- | -------------------- |
| 0.13           | 0.1                  |

# Remaining tasks

- [ ] Documentation
- [ ] Unit tests
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
