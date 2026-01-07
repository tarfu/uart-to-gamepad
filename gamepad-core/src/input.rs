//! Input source trait and error types.

use core::future::Future;
use gamepad_proto::GamepadState;

/// Error type for input operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InputError {
    /// UART/communication I/O error.
    Io,
    /// Protocol parsing error (invalid format).
    Parse,
    /// Checksum mismatch.
    Checksum,
    /// Connection lost / timeout.
    Disconnected,
    /// Buffer overflow (line too long).
    BufferOverflow,
    /// UART framing error.
    Framing,
}

impl From<gamepad_proto::ParseError> for InputError {
    fn from(err: gamepad_proto::ParseError) -> Self {
        match err {
            gamepad_proto::ParseError::Parse => InputError::Parse,
            gamepad_proto::ParseError::Checksum => InputError::Checksum,
        }
    }
}

/// Async trait for gamepad input sources.
///
/// This trait abstracts the source of gamepad data, allowing different
/// implementations (UART, Wi-Fi, BLE, I2C, SPI) to be used interchangeably.
///
/// # `no_std` Compatibility
///
/// All implementations must be `#![no_std]` compatible with no heap allocation.
pub trait InputSource {
    /// Wait for and receive the next gamepad state update.
    ///
    /// This is an async operation that yields when no data is available.
    /// Returns the new gamepad state or an error.
    fn receive(&mut self) -> impl Future<Output = Result<GamepadState, InputError>>;

    /// Check if the input source is connected/ready.
    fn is_connected(&self) -> bool;
}
