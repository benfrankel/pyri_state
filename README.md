# Flexible game states

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/benfrankel/pyri_state)
[![Crates.io](https://img.shields.io/crates/v/pyri_state.svg)](https://crates.io/crates/pyri_state)
[![Downloads](https://img.shields.io/crates/d/pyri_state.svg)](https://crates.io/crates/pyri_state)
[![Docs](https://docs.rs/pyri_state/badge.svg)](https://docs.rs/pyri_state/latest/pyri_state/)

`pyri_state` is a `bevy_state` alternative offering flexible change detection & scheduling.

```rust
#[derive(State, Clone, PartialEq, Eq)]
struct Level(usize);

app.add_systems(StateFlush, state!(Level(4 | 7 | 10)).on_enter(save_progress));
```

Read the [documentation](https://docs.rs/pyri_state/latest/pyri_state) or check out the [examples folder](/examples/) for more information.

# Bevy version compatibility

| `bevy` version | `pyri_state` version |
| -------------- | -------------------- |
| 0.14           | 0.2                  |
| 0.13           | 0.1                  |

# License

This crate is available under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-Apache-2.0) at your choice.

# Remaining tasks

- [ ] Unit tests
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Component states?
