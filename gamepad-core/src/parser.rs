//! UART protocol parser for gamepad messages.
//!
//! Supports two message types:
//! - Full state (G prefix): `G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n`
//! - Update (U prefix): `U<field>:<value>*<checksum>\n`

use crate::input::InputError;
use crate::types::{AnalogStick, Buttons, GamepadFieldUpdate, GamepadState};

/// Maximum line length for the protocol (including newline).
pub const MAX_LINE_LENGTH: usize = 64;

/// Minimum valid full state message length: G0000:0:0:0:0:0:0*XX = 20 chars
const MIN_FULL_STATE_LEN: usize = 20;

/// Minimum valid update message length: UB:0*XX = 7 chars
const MIN_UPDATE_LEN: usize = 7;

/// Parsed message - either a full gamepad state or an incremental update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum ParsedMessage {
    /// Full gamepad state (G prefix)
    FullState(GamepadState),
    /// Single field update (U prefix)
    Update(GamepadFieldUpdate),
}

/// Parse a complete line into a GamepadState.
///
/// # Protocol Format
///
/// ```text
/// G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n
/// ```
///
/// - `G` - Message prefix
/// - `buttons` - 4 hex digits (16-bit button bitfield)
/// - `lx,ly` - Left stick X/Y as signed decimal i16
/// - `rx,ry` - Right stick X/Y as signed decimal i16
/// - `lt,rt` - Triggers as unsigned decimal u8 (0-255)
/// - `checksum` - 2 hex digits (XOR of bytes between G and *)
/// - `\n` - Line terminator (CR ignored if present)
///
/// # Example
///
/// ```text
/// G0001:0:0:0:0:0:0*30\n
/// ```
///
/// This represents: A button pressed, sticks centered, triggers at 0.
#[inline]
pub fn parse(line: &[u8]) -> Result<GamepadState, InputError> {
    parse_full_state(strip_line_ending(line))
}

/// Internal parser for full gamepad state (assumes line endings already stripped).
fn parse_full_state(line: &[u8]) -> Result<GamepadState, InputError> {
    // Must start with 'G'
    if line.first() != Some(&b'G') {
        return Err(InputError::Parse);
    }

    // Extract and verify checksum
    let payload = extract_verified_payload(line, MIN_FULL_STATE_LEN)?;

    // Parse payload: buttons:lx:ly:rx:ry:lt:rt
    let mut parts = payload.split(|&b| b == b':');

    let buttons_str = parts.next().ok_or(InputError::Parse)?;
    let lx_str = parts.next().ok_or(InputError::Parse)?;
    let ly_str = parts.next().ok_or(InputError::Parse)?;
    let rx_str = parts.next().ok_or(InputError::Parse)?;
    let ry_str = parts.next().ok_or(InputError::Parse)?;
    let lt_str = parts.next().ok_or(InputError::Parse)?;
    let rt_str = parts.next().ok_or(InputError::Parse)?;

    // Should have no more parts
    if parts.next().is_some() {
        return Err(InputError::Parse);
    }

    let buttons = parse_hex_u16(buttons_str)?;
    let lx = parse_i16(lx_str)?;
    let ly = parse_i16(ly_str)?;
    let rx = parse_i16(rx_str)?;
    let ry = parse_i16(ry_str)?;
    let lt = parse_u8(lt_str)?;
    let rt = parse_u8(rt_str)?;

    Ok(GamepadState {
        buttons: Buttons(buttons),
        left_stick: AnalogStick::new(lx, ly),
        right_stick: AnalogStick::new(rx, ry),
        left_trigger: lt,
        right_trigger: rt,
    })
}

/// Parse any protocol message (full state or update).
///
/// Dispatches based on the message prefix:
/// - `G` - Full gamepad state
/// - `U` - Single field update
///
/// # Example
///
/// ```text
/// G0001:0:0:0:0:0:0*31\n  -> ParsedMessage::FullState(...)
/// UB:0001*31\n            -> ParsedMessage::Update(Buttons(...))
/// ULX:-500*XX\n           -> ParsedMessage::Update(LeftStickX(-500))
/// ```
pub fn parse_message(line: &[u8]) -> Result<ParsedMessage, InputError> {
    let line = strip_line_ending(line);

    if line.is_empty() {
        return Err(InputError::Parse);
    }

    match line[0] {
        b'G' => parse_full_state(line).map(ParsedMessage::FullState),
        b'U' => parse_update(line).map(ParsedMessage::Update),
        _ => Err(InputError::Parse),
    }
}

/// Parse an update message (U prefix).
///
/// # Protocol Format
///
/// ```text
/// U<field>:<value>*<checksum>\n
/// ```
///
/// Field identifiers:
/// - `B` - Buttons (4 hex digits)
/// - `LX` - Left stick X (signed i16)
/// - `LY` - Left stick Y (signed i16)
/// - `RX` - Right stick X (signed i16)
/// - `RY` - Right stick Y (signed i16)
/// - `LT` - Left trigger (unsigned u8)
/// - `RT` - Right trigger (unsigned u8)
fn parse_update(line: &[u8]) -> Result<GamepadFieldUpdate, InputError> {
    // Must start with 'U'
    if line.first() != Some(&b'U') {
        return Err(InputError::Parse);
    }

    // Extract and verify checksum
    let payload = extract_verified_payload(line, MIN_UPDATE_LEN)?;

    // Find the colon separator between field and value
    let colon_pos = payload
        .iter()
        .position(|&b| b == b':')
        .ok_or(InputError::Parse)?;

    let field = &payload[..colon_pos];
    let value = &payload[colon_pos + 1..];

    // Parse based on field identifier
    Ok(match field {
        b"B" => GamepadFieldUpdate::Buttons(Buttons(parse_hex_u16(value)?)),
        b"LX" => GamepadFieldUpdate::LeftStickX(parse_i16(value)?),
        b"LY" => GamepadFieldUpdate::LeftStickY(parse_i16(value)?),
        b"RX" => GamepadFieldUpdate::RightStickX(parse_i16(value)?),
        b"RY" => GamepadFieldUpdate::RightStickY(parse_i16(value)?),
        b"LT" => GamepadFieldUpdate::LeftTrigger(parse_u8(value)?),
        b"RT" => GamepadFieldUpdate::RightTrigger(parse_u8(value)?),
        _ => return Err(InputError::Parse),
    })
}

/// Calculate XOR checksum of the payload bytes.
#[inline]
fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

/// Strip trailing CR and/or LF from a line.
#[inline]
fn strip_line_ending(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    if end > 0 && line[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && line[end - 1] == b'\r' {
        end -= 1;
    }
    &line[..end]
}

/// Extract and verify checksum, returning the payload slice.
///
/// The `min_len` parameter is the minimum valid message length.
/// The input line should have line endings already stripped.
#[inline]
fn extract_verified_payload(line: &[u8], min_len: usize) -> Result<&[u8], InputError> {
    if line.len() < min_len {
        return Err(InputError::Parse);
    }

    let checksum_pos = line
        .iter()
        .rposition(|&b| b == b'*')
        .ok_or(InputError::Parse)?;

    if checksum_pos + 3 > line.len() {
        return Err(InputError::Parse);
    }

    let payload = &line[1..checksum_pos];
    let checksum_str = &line[checksum_pos + 1..];
    let expected_checksum = calculate_checksum(payload);
    let received_checksum = parse_hex_u8(checksum_str)?;

    if expected_checksum != received_checksum {
        return Err(InputError::Checksum);
    }

    Ok(payload)
}

/// Parse a 4-character hex string as u16.
#[inline]
fn parse_hex_u16(s: &[u8]) -> Result<u16, InputError> {
    if s.len() != 4 {
        return Err(InputError::Parse);
    }
    let mut value: u16 = 0;
    for &b in s {
        let digit = hex_digit(b)?;
        // Shift can never overflow: max 4 iterations shifting by 0, 4, 8, 12
        value = (value << 4) | digit as u16;
    }
    Ok(value)
}

/// Parse a 2-character hex string as u8.
#[inline]
fn parse_hex_u8(s: &[u8]) -> Result<u8, InputError> {
    if s.len() != 2 {
        return Err(InputError::Parse);
    }
    let high = hex_digit(s[0])?;
    let low = hex_digit(s[1])?;
    Ok((high << 4) | low)
}

/// Convert a hex character to its value.
#[inline]
fn hex_digit(b: u8) -> Result<u8, InputError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(InputError::Parse),
    }
}

/// Parse a decimal string as i16 (with optional leading whitespace and sign).
#[inline]
fn parse_i16(s: &[u8]) -> Result<i16, InputError> {
    let s = trim_leading_whitespace(s);
    if s.is_empty() {
        return Err(InputError::Parse);
    }

    let (negative, s) = if s[0] == b'-' {
        (true, &s[1..])
    } else if s[0] == b'+' {
        (false, &s[1..])
    } else {
        (false, s)
    };

    if s.is_empty() {
        return Err(InputError::Parse);
    }

    let mut value: i32 = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return Err(InputError::Parse);
        }
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add((b - b'0') as i32))
            .ok_or(InputError::Parse)?;
    }

    if negative {
        value = -value;
    }

    if value < i16::MIN as i32 || value > i16::MAX as i32 {
        return Err(InputError::Parse);
    }

    Ok(value as i16)
}

/// Parse a decimal string as u8 (with optional leading whitespace).
#[inline]
fn parse_u8(s: &[u8]) -> Result<u8, InputError> {
    let s = trim_leading_whitespace(s);
    if s.is_empty() {
        return Err(InputError::Parse);
    }

    let mut value: u16 = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return Err(InputError::Parse);
        }
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add((b - b'0') as u16))
            .ok_or(InputError::Parse)?;
    }

    if value > u8::MAX as u16 {
        return Err(InputError::Parse);
    }

    Ok(value as u8)
}

/// Trim leading ASCII whitespace (spaces).
#[inline]
fn trim_leading_whitespace(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&b| b != b' ').unwrap_or(s.len());
    &s[start..]
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::format;

    use super::*;

    #[test]
    fn test_parse_neutral() {
        let payload = b"0000:0:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:0:0:0:0:0:0*{:02X}\n", checksum);
        let state = parse(line.as_bytes()).unwrap();
        assert_eq!(state, GamepadState::neutral());
    }

    #[test]
    fn test_parse_button_a() {
        let payload = b"0001:0:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        let line = format!("G0001:0:0:0:0:0:0*{:02X}\n", checksum);
        let state = parse(line.as_bytes()).unwrap();
        assert!(state.buttons.is_pressed(Buttons::A));
    }

    #[test]
    fn test_parse_sticks() {
        let payload = b"0000:1000:-2000:3000:-4000:128:64";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:1000:-2000:3000:-4000:128:64*{:02X}\n", checksum);
        let state = parse(line.as_bytes()).unwrap();
        assert_eq!(state.left_stick.x, 1000);
        assert_eq!(state.left_stick.y, -2000);
        assert_eq!(state.right_stick.x, 3000);
        assert_eq!(state.right_stick.y, -4000);
        assert_eq!(state.left_trigger, 128);
        assert_eq!(state.right_trigger, 64);
    }

    #[test]
    fn test_checksum_mismatch() {
        // Use *FF which is definitely wrong (correct checksum for this payload is 00)
        let line = b"G0000:0:0:0:0:0:0*FF\n";
        assert_eq!(parse(line), Err(InputError::Checksum));
    }

    #[test]
    fn test_invalid_prefix() {
        let line = b"X0000:0:0:0:0:0:0*30\n";
        assert_eq!(parse(line), Err(InputError::Parse));
    }

    // --- Update message tests ---

    #[test]
    fn test_parse_update_buttons() {
        let payload = b"B:0003";
        let checksum = calculate_checksum(payload);
        let line = format!("UB:0003*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::Buttons(Buttons::A | Buttons::B))
        );
    }

    #[test]
    fn test_parse_update_left_stick_x() {
        let payload = b"LX:-500";
        let checksum = calculate_checksum(payload);
        let line = format!("ULX:-500*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::LeftStickX(-500))
        );
    }

    #[test]
    fn test_parse_update_left_stick_y() {
        let payload = b"LY:1000";
        let checksum = calculate_checksum(payload);
        let line = format!("ULY:1000*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::LeftStickY(1000))
        );
    }

    #[test]
    fn test_parse_update_right_stick_x() {
        let payload = b"RX:2000";
        let checksum = calculate_checksum(payload);
        let line = format!("URX:2000*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::RightStickX(2000))
        );
    }

    #[test]
    fn test_parse_update_right_stick_y() {
        let payload = b"RY:-100";
        let checksum = calculate_checksum(payload);
        let line = format!("URY:-100*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::RightStickY(-100))
        );
    }

    #[test]
    fn test_parse_update_left_trigger() {
        let payload = b"LT:128";
        let checksum = calculate_checksum(payload);
        let line = format!("ULT:128*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::LeftTrigger(128))
        );
    }

    #[test]
    fn test_parse_update_right_trigger() {
        let payload = b"RT:255";
        let checksum = calculate_checksum(payload);
        let line = format!("URT:255*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::RightTrigger(255))
        );
    }

    #[test]
    fn test_parse_update_checksum_mismatch() {
        let line = b"UB:0001*00\n";
        assert_eq!(parse_message(line), Err(InputError::Checksum));
    }

    #[test]
    fn test_parse_update_invalid_field() {
        let payload = b"XX:100";
        let checksum = calculate_checksum(payload);
        let line = format!("UXX:100*{:02X}\n", checksum);
        assert_eq!(parse_message(line.as_bytes()), Err(InputError::Parse));
    }

    #[test]
    fn test_parse_message_dispatches_g() {
        let payload = b"0000:0:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:0:0:0:0:0:0*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(result, ParsedMessage::FullState(GamepadState::neutral()));
    }

    #[test]
    fn test_apply_update() {
        let mut state = GamepadState::neutral();

        // Apply button update
        state.apply_update(GamepadFieldUpdate::Buttons(Buttons::A | Buttons::B));
        assert!(state.buttons.is_pressed(Buttons::A));
        assert!(state.buttons.is_pressed(Buttons::B));

        // Apply stick updates
        state.apply_update(GamepadFieldUpdate::LeftStickX(-1000));
        state.apply_update(GamepadFieldUpdate::RightStickY(2000));
        assert_eq!(state.left_stick.x, -1000);
        assert_eq!(state.right_stick.y, 2000);

        // Apply trigger update
        state.apply_update(GamepadFieldUpdate::LeftTrigger(128));
        assert_eq!(state.left_trigger, 128);
    }

    // --- Edge case tests ---

    #[test]
    fn test_parse_message_empty() {
        assert_eq!(parse_message(b""), Err(InputError::Parse));
        assert_eq!(parse_message(b"\n"), Err(InputError::Parse));
        assert_eq!(parse_message(b"\r\n"), Err(InputError::Parse));
    }

    #[test]
    fn test_parse_i16_max() {
        let payload = b"LX:32767";
        let checksum = calculate_checksum(payload);
        let line = format!("ULX:32767*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::LeftStickX(i16::MAX))
        );
    }

    #[test]
    fn test_parse_i16_min() {
        let payload = b"LX:-32768";
        let checksum = calculate_checksum(payload);
        let line = format!("ULX:-32768*{:02X}\n", checksum);
        let result = parse_message(line.as_bytes()).unwrap();
        assert_eq!(
            result,
            ParsedMessage::Update(GamepadFieldUpdate::LeftStickX(i16::MIN))
        );
    }

    #[test]
    fn test_parse_i16_overflow() {
        let payload = b"LX:32768";
        let checksum = calculate_checksum(payload);
        let line = format!("ULX:32768*{:02X}\n", checksum);
        assert_eq!(parse_message(line.as_bytes()), Err(InputError::Parse));
    }

    #[test]
    fn test_parse_i16_underflow() {
        let payload = b"LX:-32769";
        let checksum = calculate_checksum(payload);
        let line = format!("ULX:-32769*{:02X}\n", checksum);
        assert_eq!(parse_message(line.as_bytes()), Err(InputError::Parse));
    }

    #[test]
    fn test_parse_cr_only_line_ending() {
        // CR-only line ending should be stripped
        let payload = b"0000:0:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:0:0:0:0:0:0*{:02X}\r", checksum);
        let state = parse(line.as_bytes()).unwrap();
        assert_eq!(state, GamepadState::neutral());
    }

    #[test]
    fn test_parse_extra_parts_rejected() {
        // Message with extra colon-separated part should fail
        let payload = b"0000:0:0:0:0:0:0:99";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:0:0:0:0:0:0:99*{:02X}\n", checksum);
        assert_eq!(parse(line.as_bytes()), Err(InputError::Parse));
    }

    #[test]
    fn test_parse_missing_parts_rejected() {
        // Message with missing parts should fail
        let payload = b"0000:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        let line = format!("G0000:0:0:0:0:0*{:02X}\n", checksum);
        assert_eq!(parse(line.as_bytes()), Err(InputError::Parse));
    }
}
