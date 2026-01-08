//! Input source implementations for different protocols.
//!
//! Each protocol is conditionally compiled based on Cargo features:
//! - `proto-gamepad`: Text-based gamepad protocol (default)
//! - `proto-crsf`: CRSF/ExpressLRS protocol
//! - `proto-mavlink`: MAVLink protocol

#[cfg(feature = "proto-gamepad")]
pub mod gamepad;

#[cfg(feature = "proto-crsf")]
pub mod crsf;

#[cfg(feature = "proto-mavlink")]
pub mod mavlink;

// Re-export input sources for convenience
#[cfg(feature = "proto-gamepad")]
pub use gamepad::UartInputSource;

#[cfg(feature = "proto-crsf")]
pub use crsf::{CrsfBidirectionalSource, CrsfInputSource};

#[cfg(feature = "proto-mavlink")]
pub use mavlink::MavlinkInputSource;
