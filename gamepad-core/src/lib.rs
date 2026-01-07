//! Platform-agnostic gamepad types, protocol parsing, and traits.
//!
//! This crate provides the core abstractions for gamepad handling without
//! any platform-specific dependencies. It can be used both in embedded
//! `no_std` environments and on host for testing.
//!
//! # Features
//!
//! - `std`: Enable standard library support (for testing)
//! - `defmt`: Enable defmt formatting for embedded logging

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
