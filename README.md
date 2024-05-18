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

1. **Computed and substates:** Roll your own computed and substates with the full power of bevy systems and reasonable ergonomics.
3. **Conditional scheduling:** Configure systems to handle arbitrary state transitions using run conditions.
4. **Partial mutation:** Mutate the next state directly instead of replacing it with an entirely new value.
5. **Refresh transition:** Trigger a transition from the current state to itself.
6. **Remove and insert transitions:** Trigger a state to be removed or inserted next frame.
7. **Per-state settings:** Configure the state transition behavior per state type.
