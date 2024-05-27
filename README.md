# Flexible game states

`pyri_state` is a flexible alternative to `bevy_state`. In `pyri_state`, states are simple double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

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
                GameState::Playing.on_enter(Level::enable),
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

Click on a feature to view example code:

- [Refresh](/examples/refresh.rs)
- [Disable, enable, toggle](/examples/disable_enable_toggle.rs)
- [Computed & substates](/examples/computed_and_substates.rs)
- [Flexible scheduling](/examples/flexible_scheduling.rs)
- [Partial mutation](/examples/partial_mutation.rs)
- [Modular configuration](/examples/modular_configuration.rs)
- [Ecosystem compatibility](/examples/ecosystem_compatibility.rs)
    
And more:

- [State stack](/examples/state_stack.rs)
- [Split state](/examples/split_state.rs)

# Bevy version compatibility

| `bevy` version | `pyri_state` version |
| -------------- | -------------------- |
| 0.13           | 0.1                  |

# Remaining tasks

- [ ] Documentation
- [ ] Unit tests
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
