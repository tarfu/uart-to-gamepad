//! CRSF protocol parsing and gamepad mapping.
//!
//! This crate provides chip-agnostic CRSF protocol parsing and conversion
//! to GamepadState. It is designed to be used with any async UART implementation.
//!
//! # Features
//!
//! - Parse CRSF RC channel packets via `uf-crsf`
//! - Configurable channel-to-gamepad mapping
//! - Telemetry encoding for backchannel support
//! - No chip-specific dependencies - works on any platform
//! - Fully testable on host
//!
//! # Example
//!
//! ```ignore
//! use crsf_proto::{channels_to_gamepad, CrsfParser, Packet, DEFAULT_MAPPING};
//!
//! let mut parser = CrsfParser::new();
//!
//! // Feed bytes from UART
//! for byte in uart_bytes {
//!     if let Some(packet) = parser.push(byte) {
//!         if let Packet::RcChannels(channels) = packet {
//!             let state = channels_to_gamepad(&channels.0, &DEFAULT_MAPPING);
//!             // Use state...
//!         }
//!     }
//! }
//! ```
//!
//! # UART Configuration
//!
//! CRSF uses 420000 baud, 8N1:
//! - Baud rate: 420000 (ExpressLRS) or 416666 (TBS Crossfire)
//! - Data bits: 8
//! - Parity: None
//! - Stop bits: 1

#![cfg_attr(not(feature = "std"), no_std)]

pub mod mapping;
pub mod telemetry;

// Re-export main types from mapping
pub use mapping::{
    channels_to_gamepad, crsf_to_button, crsf_to_stick, crsf_to_trigger, ChannelMapping,
    BUTTON_THRESHOLD, CRSF_CENTER, CRSF_MAX, CRSF_MIN, DEFAULT_MAPPING,
};

// Re-export telemetry encoding
pub use telemetry::{encode_telemetry, MAX_TELEMETRY_FRAME_SIZE};

// Re-export uf_crsf types that users will need
pub use uf_crsf::packets::Packet;
pub use uf_crsf::parser::CrsfParser;

/// CRSF baud rate for ExpressLRS receivers.
pub const CRSF_BAUDRATE_ELRS: u32 = 420_000;

/// CRSF baud rate for TBS Crossfire receivers.
pub const CRSF_BAUDRATE_TBS: u32 = 416_666;
