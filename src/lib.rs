// Allows derive macros in unit tests to refer to this crate as `pyri_state`.
extern crate self as pyri_state;

#[cfg(feature = "bevy_app")]
pub mod app;
pub mod buffer;
pub mod extra;
pub mod schedule;
pub mod state;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::{AppExtPyriState, PyriStatePlugin};

    #[doc(hidden)]
    pub use crate::{
        buffer::{CurrentState, NextState_, StateMut, StateRef},
        schedule::*,
        state::*,
    };

    #[doc(hidden)]
    pub use pyri_state_derive::{RawState, State};
}
