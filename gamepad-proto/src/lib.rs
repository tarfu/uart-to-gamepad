//! UART protocol types, parsing, and serialization for the gamepad bridge.
//!
//! This crate provides everything needed to work with the gamepad bridge protocol:
//!
//! - **Types**: Core data structures for representing gamepad state
//!   - [`Buttons`] - Button state bitfield
//!   - [`AnalogStick`] - Analog stick X/Y position
//!   - [`GamepadState`] - Complete gamepad snapshot
//!   - [`GamepadFieldUpdate`] - Single field update for incremental messages
//!
//! - **Parsing**: Parse incoming protocol messages
//!   - [`parse()`] - Parse a full state message
//!   - [`parse_message()`] - Parse any message type
//!   - [`ParsedMessage`] - Result of parsing
//!
//! - **Serialization**: Serialize outgoing protocol messages
//!   - [`Serialize`] trait - Extension trait for serialization
//!   - [`MessageBuilder`] - Fluent builder API
//!
//! # Protocol Format
//!
//! The protocol uses ASCII text messages with CRC-8/SMBUS checksums.
//!
//! ## Full State Message
//!
//! ```text
//! G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n
//! ```
//!
//! - `G` - Message prefix
//! - `buttons` - 4 hex digits (16-bit button bitfield)
//! - `lx,ly` - Left stick X/Y as signed decimal i16 (-32768 to 32767)
//! - `rx,ry` - Right stick X/Y as signed decimal i16
//! - `lt,rt` - Triggers as unsigned decimal u8 (0-255)
//! - `checksum` - 2 hex digits (CRC-8/SMBUS of payload bytes)
//!
//! ## Incremental Update Message
//!
//! ```text
//! U<field>:<value>*<checksum>\n
//! ```
//!
//! Fields: `B` (buttons hex), `LX`, `LY`, `RX`, `RY` (i16), `LT`, `RT` (u8)
//!
//! # Examples
//!
//! ## Parsing Messages
//!
//! ```
//! use gamepad_proto::{parse_message, ParsedMessage, GamepadState};
//!
//! // Parse a full state message (with valid checksum)
//! let msg = b"G0001:100:-100:0:0:64:32*54\n";
//! if let Ok(ParsedMessage::FullState(state)) = parse_message(msg) {
//!     assert!(state.buttons.is_pressed(gamepad_proto::Buttons::A));
//!     assert_eq!(state.left_stick.x, 100);
//! }
//! ```
//!
//! ## Serializing with the Serialize Trait
//!
//! ```
//! use gamepad_proto::{GamepadState, Serialize, Buttons};
//!
//! let state = GamepadState {
//!     buttons: Buttons::A | Buttons::B,
//!     ..GamepadState::neutral()
//! };
//!
//! let mut buf = [0u8; 64];
//! let len = state.serialize(&mut buf).unwrap();
//! assert!(buf[..len].starts_with(b"G0003:"));
//! ```
//!
//! ## Serializing with the Builder API
//!
//! ```
//! use gamepad_proto::{MessageBuilder, Buttons};
//!
//! let mut buf = [0u8; 64];
//!
//! // Full state message
//! let len = MessageBuilder::full_state()
//!     .buttons(Buttons::X | Buttons::Y)
//!     .left_stick(1000, -500)
//!     .left_trigger(128)
//!     .serialize(&mut buf)
//!     .unwrap();
//!
//! // Incremental update message
//! let len = MessageBuilder::update()
//!     .right_trigger(255)
//!     .serialize(&mut buf)
//!     .unwrap();
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

pub mod builder;
pub mod crc;
mod fmt;
pub mod parser;
pub mod serialize;
pub mod types;

// Re-export types at crate root for convenience
pub use builder::{serialize_full_state, FullStateBuilder, MessageBuilder, UpdateBuilder};
pub use crc::{calculate_crc8, Crc8Digest};
pub use parser::{parse, parse_message, ParseError, ParsedMessage, MAX_LINE_LENGTH};
pub use serialize::{Serialize, SerializeError, MAX_FULL_STATE_SIZE, MAX_UPDATE_SIZE};
pub use types::{AnalogStick, Buttons, GamepadFieldUpdate, GamepadState};
