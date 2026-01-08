//! Minimal MAVLink parser for MANUAL_CONTROL messages.
//!
//! This is a simplified MAVLink parser that only handles MANUAL_CONTROL (ID 69)
//! messages. It does not depend on external MAVLink crates to avoid atomic
//! limitations on Cortex-M0 targets.

/// MAVLink 1 start byte.
pub const MAVLINK_STX_V1: u8 = 0xFE;

/// MAVLink 2 start byte.
pub const MAVLINK_STX_V2: u8 = 0xFD;

/// MANUAL_CONTROL message ID.
pub const MSG_ID_MANUAL_CONTROL: u32 = 69;

/// HEARTBEAT message ID.
pub const MSG_ID_HEARTBEAT: u32 = 0;

/// Maximum MAVLink frame size.
pub const MAX_FRAME_SIZE: usize = 280;

/// Minimum frame size (MAVLink 1 with empty payload).
pub const MIN_FRAME_V1: usize = 8;

/// Minimum frame size (MAVLink 2 with empty payload).
pub const MIN_FRAME_V2: usize = 12;

/// CRC-16/MCRF4XX seed value.
const CRC_INIT: u16 = 0xFFFF;

/// MANUAL_CONTROL CRC_EXTRA value.
const CRC_EXTRA_MANUAL_CONTROL: u8 = 243;

/// HEARTBEAT CRC_EXTRA value.
const CRC_EXTRA_HEARTBEAT: u8 = 50;

/// Parsed MANUAL_CONTROL message.
#[derive(Debug, Clone, Copy, Default)]
pub struct ManualControl {
    /// Target system ID.
    pub target: u8,
    /// X-axis (-1000 to 1000).
    pub x: i16,
    /// Y-axis (-1000 to 1000).
    pub y: i16,
    /// Z-axis (0 to 1000 for thrust).
    pub z: i16,
    /// R-axis (-1000 to 1000 for yaw).
    pub r: i16,
    /// Buttons bitfield.
    pub buttons: u16,
    /// Extended buttons bitfield (MAVLink 2).
    pub buttons2: u16,
}

/// Parsed MAVLink message.
#[derive(Debug, Clone, Copy)]
pub enum MavMessage {
    ManualControl(ManualControl),
    Heartbeat,
    Unknown(u32),
}

/// Parser error.
#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    /// Not enough data.
    Incomplete,
    /// Invalid start byte.
    InvalidStart,
    /// CRC mismatch.
    CrcError,
    /// Unsupported message.
    Unsupported,
}

/// MAVLink frame parser.
pub struct MavlinkParser {
    buffer: [u8; MAX_FRAME_SIZE],
    pos: usize,
    state: ParserState,
}

#[derive(Clone, Copy)]
enum ParserState {
    WaitingForStart,
    ReadingHeader,
    ReadingPayload { expected_len: usize },
}

impl MavlinkParser {
    /// Create a new parser.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buffer: [0u8; MAX_FRAME_SIZE],
            pos: 0,
            state: ParserState::WaitingForStart,
        }
    }

    /// Reset parser state.
    pub fn reset(&mut self) {
        self.pos = 0;
        self.state = ParserState::WaitingForStart;
    }

    /// Feed a byte to the parser.
    ///
    /// Returns `Some(message)` if a complete valid message was parsed.
    pub fn push_byte(&mut self, byte: u8) -> Result<Option<MavMessage>, ParseError> {
        match self.state {
            ParserState::WaitingForStart => {
                if byte == MAVLINK_STX_V1 || byte == MAVLINK_STX_V2 {
                    self.buffer[0] = byte;
                    self.pos = 1;
                    self.state = ParserState::ReadingHeader;
                }
                Ok(None)
            }
            ParserState::ReadingHeader => {
                self.buffer[self.pos] = byte;
                self.pos += 1;

                let header_size = if self.buffer[0] == MAVLINK_STX_V2 { 10 } else { 6 };

                if self.pos >= header_size {
                    // Got full header, extract payload length
                    let payload_len = self.buffer[1] as usize;
                    let checksum_len = 2;
                    let expected_len = header_size + payload_len + checksum_len;

                    if expected_len > MAX_FRAME_SIZE {
                        self.reset();
                        return Err(ParseError::InvalidStart);
                    }

                    self.state = ParserState::ReadingPayload { expected_len };
                }
                Ok(None)
            }
            ParserState::ReadingPayload { expected_len } => {
                self.buffer[self.pos] = byte;
                self.pos += 1;

                if self.pos >= expected_len {
                    // Complete frame received
                    let result = self.parse_frame();
                    self.reset();
                    result
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Parse a complete frame.
    fn parse_frame(&self) -> Result<Option<MavMessage>, ParseError> {
        let is_v2 = self.buffer[0] == MAVLINK_STX_V2;
        let payload_len = self.buffer[1] as usize;

        let (msg_id, payload_start) = if is_v2 {
            // MAVLink 2: msgid is 3 bytes at offset 7-9
            let id = (self.buffer[7] as u32)
                | ((self.buffer[8] as u32) << 8)
                | ((self.buffer[9] as u32) << 16);
            (id, 10)
        } else {
            // MAVLink 1: msgid is 1 byte at offset 5
            (self.buffer[5] as u32, 6)
        };

        let payload = &self.buffer[payload_start..payload_start + payload_len];
        let crc_start = payload_start + payload_len;

        // Verify CRC
        let crc_extra = match msg_id {
            MSG_ID_MANUAL_CONTROL => CRC_EXTRA_MANUAL_CONTROL,
            MSG_ID_HEARTBEAT => CRC_EXTRA_HEARTBEAT,
            _ => return Ok(Some(MavMessage::Unknown(msg_id))),
        };

        // Calculate CRC over header (excluding STX) + payload + CRC_EXTRA
        let crc_data_end = if is_v2 { 10 + payload_len } else { 6 + payload_len };
        let calculated_crc = crc16_mcrf4xx(&self.buffer[1..crc_data_end], crc_extra);

        let received_crc = (self.buffer[crc_start] as u16)
            | ((self.buffer[crc_start + 1] as u16) << 8);

        if calculated_crc != received_crc {
            return Err(ParseError::CrcError);
        }

        // Parse message
        match msg_id {
            MSG_ID_MANUAL_CONTROL => {
                if payload_len < 11 {
                    return Err(ParseError::Incomplete);
                }
                let msg = ManualControl {
                    target: payload[0],
                    x: i16::from_le_bytes([payload[1], payload[2]]),
                    y: i16::from_le_bytes([payload[3], payload[4]]),
                    z: i16::from_le_bytes([payload[5], payload[6]]),
                    r: i16::from_le_bytes([payload[7], payload[8]]),
                    buttons: u16::from_le_bytes([payload[9], payload[10]]),
                    buttons2: if payload_len >= 13 {
                        u16::from_le_bytes([payload[11], payload[12]])
                    } else {
                        0
                    },
                };
                Ok(Some(MavMessage::ManualControl(msg)))
            }
            MSG_ID_HEARTBEAT => Ok(Some(MavMessage::Heartbeat)),
            _ => Ok(Some(MavMessage::Unknown(msg_id))),
        }
    }
}

impl Default for MavlinkParser {
    fn default() -> Self {
        Self::new()
    }
}

/// CRC-16/MCRF4XX calculation.
fn crc16_mcrf4xx(data: &[u8], crc_extra: u8) -> u16 {
    let mut crc = CRC_INIT;

    for &byte in data {
        crc = crc_accumulate(byte, crc);
    }
    // Include CRC_EXTRA
    crc = crc_accumulate(crc_extra, crc);

    crc
}

/// Accumulate one byte into CRC.
#[inline]
fn crc_accumulate(byte: u8, mut crc: u16) -> u16 {
    let tmp = (byte ^ (crc as u8)) as u16;
    let tmp = tmp ^ (tmp << 4);
    crc = (crc >> 8) ^ (tmp << 8) ^ (tmp << 3) ^ (tmp >> 4);
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_accumulate() {
        // Basic CRC test
        let crc = crc16_mcrf4xx(&[0x00], 0);
        assert_ne!(crc, CRC_INIT);
    }

    #[test]
    fn test_parser_rejects_invalid_start() {
        let mut parser = MavlinkParser::new();
        // Random byte should be ignored
        assert!(parser.push_byte(0x00).unwrap().is_none());
        assert!(parser.push_byte(0x42).unwrap().is_none());
    }
}
