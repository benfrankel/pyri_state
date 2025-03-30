//! State debugging tools.
//!
//! Enable the `debug` feature flag to use this module.
//!
//! Insert the [`StateDebugSettings`] resource to enable debug tools.

pub mod log_flush;

#[cfg(feature = "bevy_reflect")]
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;

/// A resource that controls the behavior of [state debugging tools](crate::debug).
#[derive(Resource, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(bevy_reflect::Reflect),
    reflect(Resource)
)]
pub struct StateDebugSettings {
    /// Enable on-flush logs.
    pub log_flush: bool,
    /// Enable on-exit logs.
    pub log_exit: bool,
    /// Enable on-transition logs.
    pub log_trans: bool,
    /// Enable on-enter logs.
    pub log_enter: bool,
    /// Enable logging for local states.
    pub log_local: bool,
}
