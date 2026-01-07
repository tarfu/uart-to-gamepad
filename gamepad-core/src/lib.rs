//! Platform-agnostic gamepad types, protocol parsing, and traits.
//!
//! This crate provides the core abstractions for gamepad handling without
//! any platform-specific dependencies. It can be used both in embedded
//! `no_std` environments and on host for testing.
//!
//! # Overview
//!
//! The crate is organized into several modules:
//!
//! - [`types`]: Core data structures ([`GamepadState`], [`Buttons`], [`AnalogStick`])
//! - [`parser`]: UART protocol parsing ([`parse`], [`parse_message`])
//! - [`input`]: Input source trait ([`InputSource`])
//! - [`output`]: Output sink trait ([`OutputSink`])
//! - [`bridge`]: Orchestrates input-to-output flow ([`GamepadBridge`])
//!
//! # Protocol
//!
//! The parser supports two message types:
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
pub mod parser;
pub mod types;

// Re-export main types at crate root
pub use bridge::{BridgeError, GamepadBridge};
pub use input::{InputError, InputSource};
pub use output::{OutputError, OutputSink};
pub use parser::{parse, parse_message, ParsedMessage, MAX_LINE_LENGTH};
pub use types::{AnalogStick, Buttons, GamepadFieldUpdate, GamepadState};
