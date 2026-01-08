//! MAVLink protocol parsing and gamepad mapping.
//!
//! This crate provides chip-agnostic MAVLink protocol parsing and conversion
//! to GamepadState. It is designed to be used with any async UART implementation.
//!
//! # Features
//!
//! - Minimal MAVLink parser for MANUAL_CONTROL (ID 69) and HEARTBEAT (ID 0)
//! - Configurable axis mapping
//! - No chip-specific dependencies - works on any platform
//! - Fully testable on host
//!
//! # Example
//!
//! ```ignore
//! use mavlink_proto::{MavlinkParser, MavMessage, manual_control_to_gamepad, DEFAULT_AXIS_MAPPING};
//!
//! let mut parser = MavlinkParser::new();
//!
//! // Feed bytes from UART
//! for byte in uart_bytes {
//!     if let Ok(Some(msg)) = parser.push_byte(byte) {
//!         if let MavMessage::ManualControl(mc) = msg {
//!             let state = manual_control_to_gamepad(
//!                 mc.x, mc.y, mc.z, mc.r,
//!                 mc.buttons, mc.buttons2,
//!                 &DEFAULT_AXIS_MAPPING,
//!             );
//!             // Use state...
//!         }
//!     }
//! }
//! ```
//!
//! # MAVLink Message Types
//!
//! This crate handles:
//! - **MANUAL_CONTROL** (ID 69): Primary joystick/gamepad control message
//! - **HEARTBEAT** (ID 0): Connection presence indicator
//!
//! # UART Configuration
//!
//! MAVLink commonly uses:
//! - 57600 baud for telemetry radios
//! - 115200 baud for direct serial connections
//! - 8N1 (8 data bits, no parity, 1 stop bit)

#![cfg_attr(not(feature = "std"), no_std)]

pub mod mapping;
pub mod parser;

// Re-export main types from parser
pub use parser::{
    ManualControl, MavMessage, MavlinkParser, ParseError,
    MAVLINK_STX_V1, MAVLINK_STX_V2, MAX_FRAME_SIZE,
    MSG_ID_HEARTBEAT, MSG_ID_MANUAL_CONTROL,
};

// Re-export main types from mapping
pub use mapping::{
    manual_control_to_gamepad, mavlink_to_buttons, mavlink_to_stick, mavlink_z_to_trigger,
    AxisMapping, DEFAULT_AXIS_MAPPING, MAVLINK_AXIS_MAX, MAVLINK_AXIS_MIN, MAVLINK_Z_MAX,
    MAVLINK_Z_MIN,
};

/// Common MAVLink baud rates.
pub const MAVLINK_BAUDRATE_TELEMETRY: u32 = 57_600;
pub const MAVLINK_BAUDRATE_SERIAL: u32 = 115_200;
