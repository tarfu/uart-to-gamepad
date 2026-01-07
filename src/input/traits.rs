use crate::gamepad::GamepadState;
use defmt::Format;

/// Error type for input operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
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

/// Async trait for gamepad input sources.
///
/// This trait abstracts the source of gamepad data, allowing different
/// implementations (UART, WiFi, BLE, I2C, SPI) to be used interchangeably.
///
/// # `no_std` Compatibility
///
/// All implementations must be `#![no_std]` compatible with no heap allocation.
pub trait InputSource {
    /// Wait for and receive the next gamepad state update.
    ///
    /// This is an async operation that yields when no data is available.
    /// Returns the new gamepad state or an error.
    ///
    /// On error, implementations should log via defmt and callers should
    /// typically fall back to `GamepadState::neutral()`.
    fn receive(&mut self) -> impl core::future::Future<Output = Result<GamepadState, InputError>>;

    /// Check if the input source is connected/ready.
    fn is_connected(&self) -> bool;
}
