//! Protocol serialization for gamepad messages.
//!
//! This module provides the [`Serialize`] trait for serializing [`GamepadState`]
//! and [`GamepadFieldUpdate`] to the UART protocol format.
//!
//! # Protocol Format
//!
//! ## Full State Message
//!
//! ```text
//! G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n
//! ```
//!
//! ## Incremental Update Message
//!
//! ```text
//! U<field>:<value>*<checksum>\n
//! ```
//!
//! # Example
//!
//! ```
//! use gamepad_proto::{GamepadState, Serialize, Buttons};
//!
//! let state = GamepadState::neutral();
//! let mut buf = [0u8; 64];
//! let len = state.serialize(&mut buf).unwrap();
//!
//! // The buffer now contains the serialized message
//! assert!(buf[..len].starts_with(b"G0000:0:0:0:0:0:0*"));
//! ```

use crate::fmt::{write_hex_u16, write_hex_u8, write_i16, write_u8};
use crate::parser::calculate_checksum;
use crate::types::{GamepadFieldUpdate, GamepadState};

/// Maximum size of a serialized full state message.
///
/// Breakdown: G(1) + buttons(4) + 6*colon(6) + lx(6) + ly(6) + rx(6) + ry(6) + lt(3) + rt(3) + *(1) + checksum(2) + \n(1) = 45
/// We use 48 for safety margin.
pub const MAX_FULL_STATE_SIZE: usize = 48;

/// Maximum size of a serialized update message.
///
/// Breakdown: U(1) + field(2) + colon(1) + value(6) + *(1) + checksum(2) + \n(1) = 14
/// We use 16 for safety margin.
pub const MAX_UPDATE_SIZE: usize = 16;

/// Error type for serialization operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SerializeError {
    /// The output buffer is too small to hold the serialized message.
    BufferTooSmall,
    /// A write operation failed (for I/O adapters).
    WriteError,
}

impl core::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "buffer too small"),
            Self::WriteError => write!(f, "write error"),
        }
    }
}

/// Extension trait for serializing protocol messages.
///
/// This trait is implemented for [`GamepadState`] and [`GamepadFieldUpdate`],
/// allowing them to be serialized to various output targets.
///
/// # Example
///
/// ```
/// use gamepad_proto::{GamepadState, Serialize};
///
/// let state = GamepadState::neutral();
/// let mut buf = [0u8; 64];
/// let len = state.serialize(&mut buf).unwrap();
/// ```
pub trait Serialize {
    /// Serialize to the provided buffer.
    ///
    /// Returns the number of bytes written on success.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::BufferTooSmall`] if the buffer is not large enough.
    fn serialize(&self, buf: &mut [u8]) -> Result<usize, SerializeError>;

    /// Serialize to a `heapless::Vec`.
    ///
    /// This is a convenience method that creates a new vector and serializes into it.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::BufferTooSmall`] if `N` is not large enough.
    #[cfg(feature = "heapless")]
    fn serialize_to_vec<const N: usize>(&self) -> Result<heapless::Vec<u8, N>, SerializeError> {
        let mut vec = heapless::Vec::new();
        // Resize to full capacity to allow serialize() to write
        vec.resize(N, 0)
            .map_err(|_| SerializeError::BufferTooSmall)?;
        let len = self.serialize(&mut vec)?;
        vec.truncate(len);
        Ok(vec)
    }

    /// Serialize to a `core::fmt::Write` implementation.
    ///
    /// This can be used with types like `heapless::String`.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::WriteError`] if the write fails.
    fn serialize_fmt<W: core::fmt::Write>(&self, writer: &mut W) -> Result<(), SerializeError>;

    /// Serialize to an `embedded_io::Write` implementation.
    ///
    /// This can be used with UART or other I/O peripherals.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::WriteError`] if the write fails.
    #[cfg(feature = "embedded-io")]
    fn serialize_io<W: embedded_io::Write>(&self, writer: &mut W) -> Result<(), SerializeError>;
}

impl Serialize for GamepadState {
    fn serialize(&self, buf: &mut [u8]) -> Result<usize, SerializeError> {
        if buf.len() < MAX_FULL_STATE_SIZE {
            return Err(SerializeError::BufferTooSmall);
        }

        // Build payload first to calculate checksum
        let mut payload_buf = [0u8; MAX_FULL_STATE_SIZE];
        let mut pos = 0;

        // Buttons (4 hex digits)
        pos += write_hex_u16(&mut payload_buf[pos..], self.buttons.raw());

        // Colon separator
        payload_buf[pos] = b':';
        pos += 1;

        // Left stick X
        pos += write_i16(&mut payload_buf[pos..], self.left_stick.x);

        payload_buf[pos] = b':';
        pos += 1;

        // Left stick Y
        pos += write_i16(&mut payload_buf[pos..], self.left_stick.y);

        payload_buf[pos] = b':';
        pos += 1;

        // Right stick X
        pos += write_i16(&mut payload_buf[pos..], self.right_stick.x);

        payload_buf[pos] = b':';
        pos += 1;

        // Right stick Y
        pos += write_i16(&mut payload_buf[pos..], self.right_stick.y);

        payload_buf[pos] = b':';
        pos += 1;

        // Left trigger
        pos += write_u8(&mut payload_buf[pos..], self.left_trigger);

        payload_buf[pos] = b':';
        pos += 1;

        // Right trigger
        pos += write_u8(&mut payload_buf[pos..], self.right_trigger);

        let payload_len = pos;
        let checksum = calculate_checksum(&payload_buf[..payload_len]);

        // Now write the complete message
        let mut out_pos = 0;

        // Prefix
        buf[out_pos] = b'G';
        out_pos += 1;

        // Payload
        buf[out_pos..out_pos + payload_len].copy_from_slice(&payload_buf[..payload_len]);
        out_pos += payload_len;

        // Checksum separator
        buf[out_pos] = b'*';
        out_pos += 1;

        // Checksum (2 hex digits)
        out_pos += write_hex_u8(&mut buf[out_pos..], checksum);

        // Line ending
        buf[out_pos] = b'\n';
        out_pos += 1;

        Ok(out_pos)
    }

    fn serialize_fmt<W: core::fmt::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
        let mut buf = [0u8; MAX_FULL_STATE_SIZE];
        let len = self.serialize(&mut buf)?;

        // Convert to str and write
        let s = core::str::from_utf8(&buf[..len]).map_err(|_| SerializeError::WriteError)?;
        writer.write_str(s).map_err(|_| SerializeError::WriteError)
    }

    #[cfg(feature = "embedded-io")]
    fn serialize_io<W: embedded_io::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
        let mut buf = [0u8; MAX_FULL_STATE_SIZE];
        let len = self.serialize(&mut buf)?;
        writer
            .write_all(&buf[..len])
            .map_err(|_| SerializeError::WriteError)
    }
}

impl Serialize for GamepadFieldUpdate {
    fn serialize(&self, buf: &mut [u8]) -> Result<usize, SerializeError> {
        if buf.len() < MAX_UPDATE_SIZE {
            return Err(SerializeError::BufferTooSmall);
        }

        // Build payload first to calculate checksum
        let mut payload_buf = [0u8; MAX_UPDATE_SIZE];
        let mut pos = 0;

        match self {
            Self::Buttons(b) => {
                payload_buf[pos] = b'B';
                pos += 1;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_hex_u16(&mut payload_buf[pos..], b.raw());
            }
            Self::LeftStickX(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"LX");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_i16(&mut payload_buf[pos..], *v);
            }
            Self::LeftStickY(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"LY");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_i16(&mut payload_buf[pos..], *v);
            }
            Self::RightStickX(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"RX");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_i16(&mut payload_buf[pos..], *v);
            }
            Self::RightStickY(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"RY");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_i16(&mut payload_buf[pos..], *v);
            }
            Self::LeftTrigger(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"LT");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_u8(&mut payload_buf[pos..], *v);
            }
            Self::RightTrigger(v) => {
                payload_buf[pos..pos + 2].copy_from_slice(b"RT");
                pos += 2;
                payload_buf[pos] = b':';
                pos += 1;
                pos += write_u8(&mut payload_buf[pos..], *v);
            }
        }

        let payload_len = pos;
        let checksum = calculate_checksum(&payload_buf[..payload_len]);

        // Now write the complete message
        let mut out_pos = 0;

        // Prefix
        buf[out_pos] = b'U';
        out_pos += 1;

        // Payload
        buf[out_pos..out_pos + payload_len].copy_from_slice(&payload_buf[..payload_len]);
        out_pos += payload_len;

        // Checksum separator
        buf[out_pos] = b'*';
        out_pos += 1;

        // Checksum (2 hex digits)
        out_pos += write_hex_u8(&mut buf[out_pos..], checksum);

        // Line ending
        buf[out_pos] = b'\n';
        out_pos += 1;

        Ok(out_pos)
    }

    fn serialize_fmt<W: core::fmt::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
        let mut buf = [0u8; MAX_UPDATE_SIZE];
        let len = self.serialize(&mut buf)?;

        let s = core::str::from_utf8(&buf[..len]).map_err(|_| SerializeError::WriteError)?;
        writer.write_str(s).map_err(|_| SerializeError::WriteError)
    }

    #[cfg(feature = "embedded-io")]
    fn serialize_io<W: embedded_io::Write>(&self, writer: &mut W) -> Result<(), SerializeError> {
        let mut buf = [0u8; MAX_UPDATE_SIZE];
        let len = self.serialize(&mut buf)?;
        writer
            .write_all(&buf[..len])
            .map_err(|_| SerializeError::WriteError)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::parser::{parse, parse_message, ParsedMessage};
    use crate::types::{AnalogStick, Buttons};

    #[test]
    fn test_serialize_neutral_state() {
        let state = GamepadState::neutral();
        let mut buf = [0u8; 64];
        let len = state.serialize(&mut buf).unwrap();

        // Should start with G and end with \n
        assert_eq!(buf[0], b'G');
        assert_eq!(buf[len - 1], b'\n');

        // Should be parseable back
        let parsed = parse(&buf[..len]).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_serialize_with_buttons() {
        let state = GamepadState {
            buttons: Buttons::A | Buttons::B | Buttons::X,
            ..GamepadState::neutral()
        };
        let mut buf = [0u8; 64];
        let len = state.serialize(&mut buf).unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert_eq!(parsed, state);
        assert!(parsed.buttons.is_pressed(Buttons::A));
        assert!(parsed.buttons.is_pressed(Buttons::B));
        assert!(parsed.buttons.is_pressed(Buttons::X));
    }

    #[test]
    fn test_serialize_with_sticks() {
        let state = GamepadState {
            left_stick: AnalogStick::new(1000, -2000),
            right_stick: AnalogStick::new(-3000, 4000),
            ..GamepadState::neutral()
        };
        let mut buf = [0u8; 64];
        let len = state.serialize(&mut buf).unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_serialize_with_triggers() {
        let state = GamepadState {
            left_trigger: 128,
            right_trigger: 255,
            ..GamepadState::neutral()
        };
        let mut buf = [0u8; 64];
        let len = state.serialize(&mut buf).unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_serialize_extreme_values() {
        let state = GamepadState {
            buttons: Buttons(0xFFFF),
            left_stick: AnalogStick::new(i16::MAX, i16::MIN),
            right_stick: AnalogStick::new(i16::MIN, i16::MAX),
            left_trigger: 255,
            right_trigger: 255,
        };
        let mut buf = [0u8; 64];
        let len = state.serialize(&mut buf).unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_serialize_buffer_too_small() {
        let state = GamepadState::neutral();
        let mut buf = [0u8; 10]; // Too small
        let result = state.serialize(&mut buf);
        assert_eq!(result, Err(SerializeError::BufferTooSmall));
    }

    #[test]
    fn test_serialize_update_buttons() {
        let update = GamepadFieldUpdate::Buttons(Buttons::A | Buttons::B);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        assert_eq!(buf[0], b'U');
        assert_eq!(buf[len - 1], b'\n');

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_left_stick_x() {
        let update = GamepadFieldUpdate::LeftStickX(-500);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_left_stick_y() {
        let update = GamepadFieldUpdate::LeftStickY(1000);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_right_stick_x() {
        let update = GamepadFieldUpdate::RightStickX(-32768);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_right_stick_y() {
        let update = GamepadFieldUpdate::RightStickY(32767);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_left_trigger() {
        let update = GamepadFieldUpdate::LeftTrigger(128);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_right_trigger() {
        let update = GamepadFieldUpdate::RightTrigger(255);
        let mut buf = [0u8; 32];
        let len = update.serialize(&mut buf).unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(parsed, ParsedMessage::Update(update));
    }

    #[test]
    fn test_serialize_update_buffer_too_small() {
        let update = GamepadFieldUpdate::LeftStickX(0);
        let mut buf = [0u8; 5]; // Too small
        let result = update.serialize(&mut buf);
        assert_eq!(result, Err(SerializeError::BufferTooSmall));
    }

    #[test]
    fn test_serialize_fmt_state() {
        let state = GamepadState::neutral();
        let mut s = std::string::String::new();
        state.serialize_fmt(&mut s).unwrap();

        assert!(s.starts_with("G0000:0:0:0:0:0:0*"));
        assert!(s.ends_with('\n'));
    }

    #[test]
    fn test_serialize_fmt_update() {
        let update = GamepadFieldUpdate::LeftTrigger(64);
        let mut s = std::string::String::new();
        update.serialize_fmt(&mut s).unwrap();

        assert!(s.starts_with("ULT:64*"));
        assert!(s.ends_with('\n'));
    }
}
