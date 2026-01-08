//! Platform-agnostic gamepad types, traits, and bridge logic.
//!
//! This crate provides the core abstractions for gamepad handling without
//! any platform-specific dependencies. It can be used both in embedded
//! `no_std` environments and on host for testing.
//!
//! # Overview
//!
//! The crate is organized into several modules:
//!
//! - **Types** (re-exported from [`gamepad_proto`]): Core data structures
//!   ([`GamepadState`], [`Buttons`], [`AnalogStick`], [`GamepadFieldUpdate`])
//! - **Protocol** (re-exported from [`gamepad_proto`]): UART protocol parsing
//!   and serialization ([`parse`], [`parse_message`], [`Serialize`], [`MessageBuilder`])
//! - [`input`]: Input source trait ([`InputSource`])
//! - [`output`]: Output sink trait ([`OutputSink`])
//! - [`bridge`]: Orchestrates input-to-output flow ([`GamepadBridge`])
//! - [`telemetry`]: Bidirectional telemetry support ([`TelemetrySink`], [`TelemetrySource`])
//!
//! # Protocol
//!
//! The protocol supports two message types:
//!
//! **Full State** - Complete gamepad snapshot:
//! ```text
//! G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n
//! ```
//!
//! **Incremental Update** - Single field change:
//! ```text
//! U<field>:<value>*<checksum>\n
//! ```
//!
//! # Example
//!
//! ```rust
//! use gamepad_core::{parse_message, ParsedMessage, GamepadState};
//!
//! // Parse a full state message
//! let msg = b"G0001:1000:-1000:0:0:128:64*XX"; // checksum placeholder
//! // In real use, calculate proper checksum
//!
//! // Parse an incremental update
//! let mut state = GamepadState::neutral();
//! if let Ok(ParsedMessage::Update(update)) = parse_message(b"ULX:5000*29") {
//!     state.apply_update(update);
//!     assert_eq!(state.left_stick.x, 5000);
//! }
//! ```
//!
//! # Features
//!
//! - **`std`**: Enable standard library support (for host testing)
//! - **`defmt`**: Enable defmt formatting (for embedded logging)
//! - **`heapless`**: Enable `serialize_to_vec()` methods
//! - **`embedded-io`**: Enable `serialize_io()` methods for I/O peripherals
//!
//! # No-std Support
//!
//! This crate is `#![no_std]` by default and uses no heap allocations,
//! making it suitable for embedded systems with limited resources.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub mod bridge;
pub mod input;
pub mod output;
pub mod telemetry;

// Re-export all types and functions from gamepad-proto for convenience
pub use gamepad_proto::{
    // CRC-8 checksum
    calculate_crc8,
    Crc8Digest,
    // Parser
    parse,
    parse_message,
    // Serialization
    serialize_full_state,
    // Types
    AnalogStick,
    Buttons,
    FullStateBuilder,
    GamepadFieldUpdate,
    GamepadState,
    MessageBuilder,
    ParseError,
    ParsedMessage,
    Serialize,
    SerializeError,
    UpdateBuilder,
    MAX_FULL_STATE_SIZE,
    MAX_LINE_LENGTH,
    MAX_UPDATE_SIZE,
};

// Re-export local types
pub use bridge::{BridgeError, GamepadBridge};
pub use input::{InputError, InputSource};
pub use output::{OutputError, OutputSink};
pub use telemetry::{
    MockTelemetrySource, NullTelemetrySink, TelemetryData, TelemetryError, TelemetrySink,
    TelemetrySource,
};
