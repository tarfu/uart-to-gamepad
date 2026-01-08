//! UART to USB Gamepad bridge for RP2040.
//!
//! This crate provides the embedded implementation of a gamepad bridge
//! that reads gamepad state from UART and outputs it as USB HID.
//!
//! # Overview
//!
//! The firmware runs on a Raspberry Pi Pico (RP2040) and:
//! 1. Receives gamepad state over UART (115200 baud, 8N1)
//! 2. Parses the protocol messages (full state or incremental updates)
//! 3. Outputs the state as a USB HID gamepad
//!
//! # Hardware Configuration
//!
//! | Function | GPIO | Description |
//! |----------|------|-------------|
//! | UART1 TX | 8    | Serial transmit |
//! | UART1 RX | 9    | Serial receive (gamepad data input) |
//! | LED      | 25   | On-board LED (error indicator) |
//!
//! # Architecture
//!
//! The firmware uses the Embassy async runtime with three concurrent tasks:
//!
//! - **USB Task**: Manages the USB device stack
//! - **Input Task**: Reads UART data, parses protocol, signals state changes
//! - **Output Task**: Receives state signals, formats and sends USB HID reports
//!
//! Communication between tasks uses Embassy's [`Signal`](embassy_sync::signal::Signal)
//! with "latest value wins" semantics, ensuring the USB output always reflects
//! the most recent gamepad state.
//!
//! # Modules
//!
//! - [`uart_input`]: UART-based input source ([`UartInputSource`])
//! - [`usb_output`]: USB HID output ([`UsbHidOutput`], [`GamepadReport`])
//!
//! # Features
//!
//! - **`dev-panic`** (default): Use `panic-probe` for development (prints panic info via RTT)
//! - **`prod-panic`**: Use `panic-reset` for production (silent watchdog reset)
//! - **`standard-hid`** (default): Standard HID gamepad descriptor (cross-platform)
//! - **`xinput-compat`**: Xbox-style HID descriptor (better Windows game support)
//! - **`uart-flow-control`**: Enable hardware flow control (CTS/RTS on GPIO 10/11)
//!
//! # Re-exports
//!
//! This crate re-exports all public items from [`gamepad_core`] for convenience,
//! so consumers only need to depend on this crate.

#![no_std]

// Ensure mutually exclusive HID descriptor features
#[cfg(all(feature = "standard-hid", feature = "xinput-compat"))]
compile_error!("Cannot enable both `standard-hid` and `xinput-compat` features - they define conflicting HID descriptors");

// Re-export core types for convenience
pub use gamepad_core::{
    parse, parse_message, AnalogStick, BridgeError, Buttons, GamepadBridge, GamepadFieldUpdate,
    GamepadState, InputError, InputSource, OutputError, OutputSink, ParsedMessage, MAX_LINE_LENGTH,
};

pub mod input;
pub mod usb_output;

// Re-export input sources based on selected protocol
#[cfg(feature = "proto-gamepad")]
pub use input::UartInputSource;

#[cfg(feature = "proto-crsf")]
pub use input::{CrsfBidirectionalSource, CrsfInputSource};

#[cfg(feature = "proto-mavlink")]
pub use input::MavlinkInputSource;

pub use usb_output::{configure_usb_hid, GamepadReport, GamepadRequestHandler, UsbHidOutput};
