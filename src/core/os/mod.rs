//! This module contains implementations of core functionalities,
//! each submodule must implement traits defined in [`core`](crate::core).
//! such as [`Preinstallation`](crate::core::Preinstallation).
//!
//! Note: If you add/remove sub mods here to add/remove support for certain OS,
//! make sure to update `build.rs` as well.

#[cfg(unix)]
pub(crate) mod unix;
