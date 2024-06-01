// Allow macros to refer to this crate as `pyri_state` internally.
extern crate self as pyri_state;

#[cfg(feature = "bevy_app")]
pub mod app;
#[cfg(feature = "extra")]
pub mod extra;
pub mod pattern;
pub mod schedule;
pub mod state;
pub mod storage;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::{AppExtPyriState, PyriStatePlugin};

    #[doc(hidden)]
    pub use crate::{
        pattern::{StatePattern, StatePatternExtGet, StatePatternExtGetAndEq},
        schedule::{StateFlush, StateFlushEvent},
        state::{
            CurrentState, GetState, NextStateMut, NextStateRef, RawState, SetState,
            SetStateExtClone, SetStateExtDefault, StateFlushMut, StateFlushRef,
        },
        storage::StateBuffer,
    };

    #[doc(hidden)]
    pub use pyri_state_derive::State;
}
