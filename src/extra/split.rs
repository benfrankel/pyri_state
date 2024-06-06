//! TODO: Module-level documentation

/// Define your own split state type by newtyping `SplitState`:
///
/// ```rust
/// use pyri_state::prelude::*;
/// use pyri_state::extra::split::SplitState;
/// use pyri_state::add_to_split_state;
///
/// #[derive(State, Clone, PartialEq, Eq)]
/// pub struct MyState(pub SplitState);
///
/// add_to_split_state!(MyState, Foo, Bar);
/// add_to_split_state!(MyState, Quux);
/// ```
pub type SplitState = &'static str;

#[macro_export]
macro_rules! add_to_split_state {
    ($ty:ident, $($val:ident),* $(,)?) => {
        #[allow(non_upper_case_globals)]
        impl $ty {
            $(pub const $val: $ty = $ty(stringify!($val));)*
        }
    };
}
