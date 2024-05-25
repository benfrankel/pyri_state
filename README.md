`pyri_state` is a flexible 3rd-party alternative to `bevy_state`. In `pyri_state`, states are simple double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

# Ergonomics

TODO: Sample code here.

# Features

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

- [ ] Unit tests
- [ ] Documentation
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
