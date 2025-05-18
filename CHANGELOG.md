# Next version

# Version 0.4.0

- **Updated to Bevy 0.16.0**
- **Added `no_std` support**
- **Extended reaction components:**
    - Removed `Single` variant from `DespawnOnExit` and `DespawnOnDisable` components
    - Renamed `DespawnOnExit` -> `DespawnOnExitState` component
    - Renamed `DespawnOnDisable` -> `DespawnOnDisableState` component
    - Renamed `VisibleWhileIn` -> `VisibleInState` component
    - Renamed `VisibleWhileEnabled` -> `VisibleInEnabledState` component
    - Added `EnabledInState` component
    - Added `EnabledInEnabledState` component
- Added `NextStateStackCommandsExt` extension trait
- Moved `StateFlush` to before `PreUpdate` schedule
- Renamed `ApplyFlushSet` -> `ApplyFlushSystems` system set
- Renamed `ResolveStateSet` -> `ResolveStateSystems` system set

# Version 0.3.0

- **Updated to Bevy 0.15.0**
- **Extended reaction components:**
    - Renamed `StateScope` -> `DespawnOnExit` component with `Single`, `Recursive`, and `Descendants` variants
    - Added `DespawnOnDisable` component
    - Added `VisibleWhileIn` component
    - Added `VisibleWhileEnabled` component
    - Added re-exports for reaction components to `prelude` module
- Renamed `relax` -> `reset_trigger` method on `FlushMut` system param
- Added `toggle_default`, `enable_default`, and `enter_default` methods to `FlushMut` system param
- Changed `toggle` and `enable` functions and systems to read the current state instead of the next state when possible

# Version 0.2.1

- Fixed 1-frame delay when using `BevyState` wrapper
- Added `StateExtBevy` extension trait

# Version 0.2.0

- **Updated to Bevy 0.14.0**
- **Wrote [documentation](https://docs.rs/pyri_state/latest/pyri_state/)**
- **Wrote [interactive examples](/examples/)**
- **Implemented state pattern-matching:**
    - Added `pattern` module
    - Added `state!` macro
    - Added `StatePattern` trait
    - Added `StatePatternExtClone` extension trait
    - Added `StatePatternExtEq` extension trait
    - Added `StateTransPattern` trait
    - Added `StateTransPatternExtClone` extension trait
    - Added `AnyStateTransPattern` type
    - Added `AnyStatePattern` type
    - Added `FnStatePattern` type
    - Added `FnStateTransPattern` type
    - Replaced `on_any_xyz` methods with `State::ANY` and `State::ANY_TO_ANY` constants
    - Replaced `on_xyz_and` methods with `State::with` and `State::when` methods
- **Implemented local states as components:**
    - Added `local` derive macro option
    - Required `Resource` for `State` trait
    - Added `LocalState` marker trait
    - Added `AppExtState::register_state` method
    - Added `CommandsExtState` extension trait
    - Added `EntityCommandsExtState` extension trait
    - Added local variants for state plugins:
        - Added `LocalDetectChangePlugin` plugin
        - Added `LocalFlushEventPlugin` plugin
        - Added `LocalApplyFlushPlugin` plugin
        - Added `LocalLogFlushPlugin` plugin
        - Added `schedule_local_detect_change` function
        - Added `schedule_local_flush_event` function
        - Added `schedule_local_apply_flush` function
        - Added `schedule_local_log_flush` function
- **Implemented custom next state storage:**
    - Added `next_state` module
    - Added `next(...)` derive macro option
    - Added `NextState` trait
    - Added `NextStateMut` trait
    - Split `NextState_` resource into `NextStateBuffer` and `TriggerStateFlush` resource / components
    - **Implemented next state stack:**
        - Added `stack` feature flag
        - Added `NextStateStack` resource / component
        - Added `NextStateStackMut` extension trait
        - Added `NextStateStackMutExtClone` extension trait
    - **Implemented next state sequence / index:**
        - Added `sequence` feature flag
        - Added `NextStateSequence` resource
        - Added `NextStateIndex` resource / component
        - Added `NextStateIndexMut` extension trait
- **Implemented state flush logging:**
    - Added `debug` module
    - Added `debug` feature flag
    - Added `StateDebugSettings` resource
    - Added `log_flush` derive macro option
    - Added `LogFlushPlugin` plugin
    - Added `schedule_log_flush` function
- Implemented some extra features:
    - Added `extra` module
    - Moved `BevyState` and related items into new `extra::bevy_state` module
    - **Implemented state scoping for entities:**
        - Added `entity_scope` feature flag
        - Added `StateScope` component
        - Added `schedule_entity_scope` function
        - Added `EntityScopePlugin` plugin
    - **Implemented split state helper:**
        - Added `split` feature flag
        - Added `SplitState` type alias
        - Added `add_to_split_state!` macro
- Adjusted state traits:
    - Removed `State_` trait
    - Renamed `RawState` -> `State` trait
    - Added `StateExtEq` extension trait
    - Added `StateMut` extension trait
    - Renamed `RawStateExtClone` -> `StateMutExtClone` extension trait
    - Renamed `RawStateExtDefault` -> `StateMutExtDefault` extension trait
    - Removed `RawStateExtEq` extension trait (see `StatePatternExtEq` instead)
- Adjusted `AppExtState`:
    - Renamed `app` -> `setup` module
    - Renamed `add_state_` -> `add_state` method
    - Renamed `init_state_` -> `init_state` method
    - Renamed `insert_state_` -> `insert_state` method
- Replaced configs with plugins:
    - Replaced `GetStateConfig` and `ConfigureState` with `RegisterState` trait
    - Renamed `StateConfigResolveState` -> `ResolveStatePlugin` plugin
    - Renamed `StateConfigDetectChange` -> `DetectChangePlugin` plugin
    - Renamed `StateConfigSendEvent` -> `FlushEventPlugin` plugin
    - Renamed `StateConfigBevyState` -> `BevyStatePlugin` plugin
    - Renamed `StateConfigApplyFlush` -> `ApplyFlushPlugin` plugin
- Adjusted scheduling:
    - Split out `resolve_state`, `detect_change`, `flush_event`, and `apply_flush` submodules
    - Renamed `StateFlushSet` -> `ResolveStateSet` system set
    - Adjusted `ResolveStateSet` variants:
        - Added `Compute` variant
        - Renamed `Transition` -> `Trans` variant
        - Added `AnyFlush` variant
        - Added `AnyExit` variant
        - Added `AnyTrans` variant
        - Added `AnyEnter` variant
    - Renamed `StateFlushEvent` fields: `before` -> `old` and `after` -> `new`
    - Renamed `schedule_send_event` -> `schedule_flush_event` function
    - Renamed `send_event` -> `flush_event` derive macro option
- Adjusted system params:
    - Added `access` module
    - Moved system params into new `access` module
    - Added `CurrentRef` system param
    - Added `CurrentMut` system param
    - Added `NextRef` system param
    - Added `NextMut` system param
    - Renamed `StateRef` -> `FlushRef` system param
    - Renamed `StateMut` -> `FlushMut` system param

# Version 0.1.0

- **Initial release**
