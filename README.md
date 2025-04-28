# pyri_state

[![Crates.io](https://img.shields.io/crates/v/pyri_state.svg?style=for-the-badge)](https://crates.io/crates/pyri_state)
[![Docs](https://img.shields.io/docsrs/pyri_state/latest?style=for-the-badge)](https://docs.rs/pyri_state/latest/pyri_state/)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](https://github.com/benfrankel/pyri_state)

`pyri_state` is a `bevy_state` alternative offering flexible change detection & scheduling.

```rust
#[derive(State, Clone, PartialEq, Eq)]
struct Level(usize);

app.add_systems(StateFlush, state!(Level(4 | 7 | 10)).on_enter(save_progress));
```

See the [examples](/examples/) and [documentation](https://docs.rs/pyri_state/latest/pyri_state) for more information.

# Comparison to `bevy_state`

## State pattern-matching

In `pyri_state`, state pattern-matching is directly supported:

```rust
// Save progress when entering level 4, 7, or 10.
app.add_systems(StateFlush, state!(Level(4 | 7 | 10)).on_enter(save_progress));
```

There are a few ways to do this using `bevy_state`:

1. Add a system for every possible matching state.

```rust
for x in [4, 7, 10] {
    app.add_systems(OnEnter(Level(x)), save_progress);
}
```

2. Use a custom substate.

```rust
app.add_systems(OnEnter(SaveProgressLevel), save_progress);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct SaveProgressLevel;

impl SubStates for SaveProgressLevel {
    type SourceStates = Level;

    fn should_exist(sources: Level) -> Option<Self> {
        matches!(sources, Level(4 | 7 | 10)).then(Self)
    }
}
```

3. Use a custom schedule.

```rust
app.add_systems(OnSaveProgress, save_progress);

app.add_systems(
    StateTransition,
    last_transition::<Level>
        .pipe(run_save_progress)
        .in_set(EnterSchedules::<Level>::default()),
);

#[derive(ScheduleLabel, Clone, Eq, PartialEq, Hash, Debug)]
struct OnSaveProgress;

fn run_save_progress(transition: In<Option<StateTransitionEvent<S>>>, world: &mut World) {
    if matches!(transition.0, Some(StateTransitionEvent {
        entered: Some(Level(4 | 7 | 10)),
        ..
    })) {
        let _ = world.try_run_schedule(OnSaveProgress);
    }
}
```

Note that option 1 is prohibitively expensive when the pattern has too many matches, like `Level(x) if x % 2 == 0`.
Options 2 and 3 add a confusing layer of indirection and boilerplate, hiding the actual pattern-matching in
the `SubStates` implementation or the `run_my_schedule` exclusive system.

Even worse, option 2 is subtly broken: if you transition from state A to B where both states match the pattern,
`bevy_state` will silently discard the substate's transition because it's a same-state transition.

## State refreshing

In `pyri_state`, state refreshing is supported out-of-the-box:

```rust
// Restart game on R press.
app.add_systems(Update, Level::refresh.run_if(input_just_pressed(KeyCode::R)));
// Schedule a system for when any level restarts.
app.add_systems(StateFlush, Level::ANY.on_refresh(|| info!("Restarted level")));
// Refreshing a state will also reuse its exit, trans, and enter hooks.
app.add_systems(StateFlush, Level::ANY.on_exit(tear_down_level));
// You can explicitly check whether the state has changed, if you want.
app.add_systems(StateFlush, Level::ANY.on_enter(load_new_level.run_if(Level::will_change)));
```

The equivalent in `bevy_state` requires building your own custom schedules
(e.g. `OnReExit`, `OnReTransition`, `OnReEnter`, `OnChangeExit`, `OnChangeTransition`, `OnChangeEnter`, etc.)
and hooking them into the state transition internals,
[as in this example](https://github.com/bevyengine/bevy/blob/main/examples/state/custom_transitions.rs).
This is a seriously discouraging amount of boilerplate for something that should be a basic feature.

## And more

- **Custom storage:** In `pyri_state`, the next state can be stored in any custom data structure.
  For example, you can store the next state in a stack to implement a "back button" feature for a menu
  state as easily as `Menu::pop`.
  This is currently impossible in `bevy_state`, which only supports `enum NextState`.
- **Direct mutation:** In `pyri_state`, systems can mutate the next state value directly (e.g. `level.0 += 1`).
  In `bevy_state`, you have to clone the current state, mutate it, and set that as the next state.
  As a consequence, if multiple systems mutate the same state on the same frame, they'll completely overwrite each other,
  leading to rare, confusing bugs that direct mutation would often circumvent entirely.
- **Local states:** In `pyri_state`, states can be components.
  This is currently impossible in `bevy_state`, which only supports global states.

# Bevy version compatibility

| `bevy` version | `pyri_state` version |
| -------------- | -------------------- |
| 0.16           | 0.4                  |
| 0.15           | 0.3                  |
| 0.14           | 0.2                  |
| 0.13           | 0.1                  |

# License

This crate is available under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-Apache-2.0) at your choice.
