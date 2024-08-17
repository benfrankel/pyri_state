//! Split the definition of a simple enum-like [`State`](crate::state::State) between
//! the modules of your crate.
//!
//! Enable the `split` feature flag to use this module.
//!
//! Newtype [`SplitState`] to define a new split state type, and use
//! [`add_to_split_state!`](crate::add_to_split_state!) to extend it.
//!
//! This can be a useful organizational tool for cross-cutting states in a plugin-based
//! codebase.

/// The internal value of a split state type.
///
/// # Example
///
/// Define your own split state type as a newtype:
///
/// ```ignore
/// #[derive(State, Clone, PartialEq, Eq)]
/// pub struct MyState(pub SplitState);
/// ```
pub type SplitState = &'static str;

/// A macro for extending [`SplitState`] newtypes.
///
/// # Example
///
/// ```ignore
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
