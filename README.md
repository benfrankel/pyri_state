`pyrious_state` is an experimental 3rd-party alternative to `bevy_state`.

# Thoughts

- States are just double buffers with a fixed flush point and some change detection and dependency tooling.

# Questions

- How does flushing states once per frame interact with `FixedUpdate`?
- Should settings be attached to the `State` trait via derive macro? Or is `app.add_state_with_settings(...)` better because it's not as magical?

# Remaining tasks

- [ ] Implement per-state settings.
- [ ] Provide at least one test / example for every listed feature below.
- [ ] Write documentation.

# Features

1. **Computed & sub states:** Roll your own computed and sub states with the full power of bevy systems.
2. **Conditional scheduling:** Configure systems to handle arbitrary state transitions via run conditions.
3. **Partial mutation:** Mutate the next state directly instead of replacing it with an entirely new value.
4. **Refresh transition:** Trigger a transition from the current state to itself.
5. **Remove & insert triggers:** Trigger a state to be removed or inserted next frame.
6. **Per-state settings:** Configure the state transition behavior per state type.
7. **Ergonomic wins:** See below.

# Example code

Using `bevy_state`:

```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
enum GameState { ... }

fn frobnicate(
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) { ... }

enum ShouldSpawnEasterEggState(bool);

impl ComputedState for ShouldSpawnEasterEggState {
    type SourceStates = GameState;
    
    // Can't access the previous state or use any system params in here, and there's no way
    // to trigger or handle same-state transitions (aka refresh transitions) on GameState.
    fn compute(sources: GameState) -> Option<Self> { ... }
}

app.add_systems(OnEnter(ShouldSpawnEasterEggState(true)), spawn_easter_egg);
```

Using `pyrious_state`:

```rust
#[derive(State, Clone, PartialEq, Eq)]
enum GameState { ... }

fn frobnicate(mut game_state: StateMut<GameState>) { ... }

fn should_spawn_easter_egg(game_state: StateRef<GameState>) -> bool {
    let (current, next) = game_state.unwrap();
    ...
}

app.add_systems(
    StateTransition,
    spawn_easter_egg.run_if(state_will_transition::<GameState>.and_then(should_spawn_easter_egg)),
);
```
