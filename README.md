# Flexible game states

`pyri_state` is a flexible alternative to `bevy_state`. In `pyri_state`, states are double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

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
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .init_state_::<GameState>()
        .add_state_::<Level>()
        .add_systems(
            StateFlush,
            (
                GameState::Playing.on_exit(Level::disable),
                GameState::Playing.on_enter(Level::enable_default),
                Level::ANY.on_exit(tear_down_old_level),
                Level::ANY.on_enter(set_up_new_level),
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
- **[Disable, enable, toggle](/examples/disable_enable_toggle.rs):** Disable or enable any state on command (great for toggle states and substates).
- **[Partial mutation](/examples/partial_mutation.rs):** Directly update the next state instead of setting an entirely new value.
- **[Computed & substates](/examples/computed_and_substates.rs):** Roll your own computed and substates with the full power of Bevy ECS.
- **[Flexible scheduling](/examples/flexible_scheduling.rs):** Harness the full power of Bevy ECS to schedule your state transitions.
- **[Custom storage](/examples/custom_storage.rs):** Swap out or define your own state storage type.
    - **Slot:** Only store the next state. This is the default storage type.
    - **[Stack](/examples/stack_storage.rs):** Keep track of a state's history in a stack (e.g. back button).
    - **[Sequence](/examples/sequence_storage.rs):** Navigate a fixed sequence of states by index (e.g. pages).
- **[Ecosystem compatibility](/examples/ecosystem_compatibility.rs):** Enable a `BevyState<S>` wrapper to interact with crates that expect it.
- **[Modular configuration](/examples/modular_configuration.rs):** Strip out or add plugins to your state type using the derive macro.
    
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
