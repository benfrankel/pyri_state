//! A split state works like a basic enum state that can be split between the modules of a crate.
//! It's a useful organizational tool for cross-cutting states in a plugin-based codebase.
//!
//! Newtype [`SplitState`] to define a split state type, and use [`add_to_split_state!`]
//! to extend it.

/// The internal value of a split state type.
///
/// # Example
///
/// Define your own split state type as a newtype:
///
/// ```rust
/// #[derive(State, Clone, PartialEq, Eq)]
/// pub struct MyState(pub SplitState);
/// ```
pub type SplitState = &'static str;

/// A macro for extending [`SplitState`] newtypes.
///
/// # Example
///
/// ```rust
/// add_to_split_state!(MyState, Foo, Bar);
/// add_to_split_state!(MyState, Quux);
/// ```
#[macro_export]
macro_rules! add_to_split_state {
    ($ty:ident, $($val:ident),* $(,)?) => {
        #[allow(non_upper_case_globals)]
        impl $ty {
            $(pub const $val: $ty = $ty(stringify!($val));)*
        }
    };
}
