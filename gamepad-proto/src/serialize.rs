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

use crate::crc::Crc8Digest;
use crate::fmt::{write_hex_u16, write_hex_u8, write_i16, write_u8};
use crate::types::{GamepadFieldUpdate, GamepadState};

/// Helper for buffer management with incremental CRC-8 checksum calculation.
///
/// Writes directly to the output buffer while accumulating the CRC-8 checksum,
/// eliminating the need for intermediate payload buffers.
struct SerializeBuf<'a> {
    buf: &'a mut [u8],
    pos: usize,
    crc: Crc8Digest,
}

impl<'a> SerializeBuf<'a> {
    /// Create a new serialization buffer.
    #[inline]
    fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            crc: Crc8Digest::new(),
        }
    }

    /// Write a byte without checksumming (for prefix, separator, newline).
    #[inline]
    fn write_raw(&mut self, byte: u8) {
        self.buf[self.pos] = byte;
        self.pos += 1;
    }

    /// Write a byte and accumulate into CRC-8 checksum.
    #[inline]
    fn write(&mut self, byte: u8) {
        self.buf[self.pos] = byte;
        self.crc.update(byte);
        self.pos += 1;
    }

    /// Write multiple bytes and accumulate into CRC-8 checksum.
    #[inline]
    fn write_slice(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write(b);
        }
    }

    /// Write hex u16 (4 bytes) with checksum.
    #[inline]
    fn write_hex_u16(&mut self, value: u16) {
        let mut tmp = [0u8; 4];
        write_hex_u16(&mut tmp, value);
        self.write_slice(&tmp);
    }

    /// Write i16 decimal with checksum.
    #[inline]
    fn write_i16(&mut self, value: i16) {
        let mut tmp = [0u8; 6];
        let len = write_i16(&mut tmp, value);
        self.write_slice(&tmp[..len]);
    }

    /// Write u8 decimal with checksum.
    #[inline]
    fn write_u8(&mut self, value: u8) {
        let mut tmp = [0u8; 3];
        let len = write_u8(&mut tmp, value);
        self.write_slice(&tmp[..len]);
    }

    /// Finalize by writing CRC-8 checksum and newline.
    #[inline]
    fn finalize(self) -> usize {
        let checksum = self.crc.finalize();
        let mut pos = self.pos;

        // Write directly to buffer since we've consumed the CRC digest
        self.buf[pos] = b'*';
        pos += 1;

        // Write checksum as 2 hex digits
        pos += write_hex_u8(&mut self.buf[pos..], checksum);

        self.buf[pos] = b'\n';
        pos += 1;

        pos
    }
}

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

        let mut sb = SerializeBuf::new(buf);

        // Prefix (not checksummed)
        sb.write_raw(b'G');

        // Payload (checksummed)
        sb.write_hex_u16(self.buttons.raw());
        sb.write(b':');
        sb.write_i16(self.left_stick.x);
        sb.write(b':');
        sb.write_i16(self.left_stick.y);
        sb.write(b':');
        sb.write_i16(self.right_stick.x);
        sb.write(b':');
        sb.write_i16(self.right_stick.y);
        sb.write(b':');
        sb.write_u8(self.left_trigger);
        sb.write(b':');
        sb.write_u8(self.right_trigger);

        // Finalize with checksum and newline
        Ok(sb.finalize())
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

        let mut sb = SerializeBuf::new(buf);

        // Prefix (not checksummed)
        sb.write_raw(b'U');

        // Field:value (checksummed)
        match self {
            Self::Buttons(b) => {
                sb.write_slice(b"B:");
                sb.write_hex_u16(b.raw());
            }
            Self::LeftStickX(v) => {
                sb.write_slice(b"LX:");
                sb.write_i16(*v);
            }
            Self::LeftStickY(v) => {
                sb.write_slice(b"LY:");
                sb.write_i16(*v);
            }
            Self::RightStickX(v) => {
                sb.write_slice(b"RX:");
                sb.write_i16(*v);
            }
            Self::RightStickY(v) => {
                sb.write_slice(b"RY:");
                sb.write_i16(*v);
            }
            Self::LeftTrigger(v) => {
                sb.write_slice(b"LT:");
                sb.write_u8(*v);
            }
            Self::RightTrigger(v) => {
                sb.write_slice(b"RT:");
                sb.write_u8(*v);
            }
        }

        // Finalize with checksum and newline
        Ok(sb.finalize())
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
