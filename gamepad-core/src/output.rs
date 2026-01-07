//! Output sink trait and error types.

use core::future::Future;
use gamepad_proto::GamepadState;

/// Error type for output operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputError {
    /// USB/communication I/O error.
    Io,
    /// Device not ready (e.g., USB not enumerated).
    NotReady,
    /// Report dropped (e.g., host not polling fast enough).
    Dropped,
    /// Endpoint busy.
    Busy,
}

/// Async trait for gamepad output sinks.
///
/// This trait abstracts the destination for gamepad data, enabling
/// different output methods (USB HID, BLE HID, serial debug, etc.).
///
/// # `no_std` Compatibility
///
/// All implementations must be `#![no_std]` compatible with no heap allocation.
pub trait OutputSink {
    /// Send a gamepad state to the output.
    ///
    /// May block until the previous report has been sent.
    fn send(&mut self, state: &GamepadState) -> impl Future<Output = Result<(), OutputError>>;

    /// Check if the output is ready to accept data.
    fn is_ready(&self) -> bool;
}
