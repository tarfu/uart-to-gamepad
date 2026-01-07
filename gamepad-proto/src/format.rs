//! No-std compatible number formatting utilities for protocol serialization.
//!
//! These functions write formatted numbers directly to byte buffers without
//! requiring heap allocation or the standard library.

/// Hex digits lookup table for fast conversion.
const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Write a u16 as 4 uppercase hex digits.
///
/// Returns the number of bytes written (always 4).
///
/// # Panics
///
/// Panics if `buf.len() < 4`.
#[inline]
pub fn write_hex_u16(buf: &mut [u8], value: u16) -> usize {
    debug_assert!(buf.len() >= 4, "buffer too small for hex u16");
    buf[0] = HEX_DIGITS[((value >> 12) & 0xF) as usize];
    buf[1] = HEX_DIGITS[((value >> 8) & 0xF) as usize];
    buf[2] = HEX_DIGITS[((value >> 4) & 0xF) as usize];
    buf[3] = HEX_DIGITS[(value & 0xF) as usize];
    4
}

/// Write a u8 as 2 uppercase hex digits.
///
/// Returns the number of bytes written (always 2).
///
/// # Panics
///
/// Panics if `buf.len() < 2`.
#[inline]
pub fn write_hex_u8(buf: &mut [u8], value: u8) -> usize {
    debug_assert!(buf.len() >= 2, "buffer too small for hex u8");
    buf[0] = HEX_DIGITS[(value >> 4) as usize];
    buf[1] = HEX_DIGITS[(value & 0xF) as usize];
    2
}

/// Write an i16 as a signed decimal string.
///
/// Returns the number of bytes written (1-6 bytes).
///
/// # Panics
///
/// Panics if `buf.len() < 6` (max size: "-32768").
#[inline]
pub fn write_i16(buf: &mut [u8], value: i16) -> usize {
    debug_assert!(buf.len() >= 6, "buffer too small for i16");

    if value == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut pos = 0;
    let (abs_value, is_negative) = if value < 0 {
        buf[0] = b'-';
        pos = 1;
        // Handle i16::MIN specially to avoid overflow
        if value == i16::MIN {
            // -32768 is a special case
            buf[1..6].copy_from_slice(b"32768");
            return 6;
        }
        ((-value) as u16, true)
    } else {
        (value as u16, false)
    };

    // Write digits in reverse order to temporary buffer
    let mut temp = [0u8; 5];
    let mut n = abs_value;
    let mut len = 0;
    while n > 0 {
        temp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }

    // Copy digits in correct order
    for i in (0..len).rev() {
        buf[pos] = temp[i];
        pos += 1;
    }

    if is_negative {
        1 + len
    } else {
        len
    }
}

/// Write a u8 as an unsigned decimal string.
///
/// Returns the number of bytes written (1-3 bytes).
///
/// # Panics
///
/// Panics if `buf.len() < 3` (max size: "255").
#[inline]
pub fn write_u8(buf: &mut [u8], value: u8) -> usize {
    debug_assert!(buf.len() >= 3, "buffer too small for u8");

    if value == 0 {
        buf[0] = b'0';
        return 1;
    }

    // Write digits in reverse order to temporary buffer
    let mut temp = [0u8; 3];
    let mut n = value;
    let mut len = 0;
    while n > 0 {
        temp[len] = b'0' + (n % 10);
        n /= 10;
        len += 1;
    }

    // Copy digits in correct order
    for i in (0..len).rev() {
        buf[len - 1 - i] = temp[i];
    }

    len
}

/// Calculate XOR checksum of the given bytes.
///
/// This is the same algorithm used by the parser.
#[inline]
pub fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_hex_u16() {
        let mut buf = [0u8; 4];

        write_hex_u16(&mut buf, 0x0000);
        assert_eq!(&buf, b"0000");

        write_hex_u16(&mut buf, 0xFFFF);
        assert_eq!(&buf, b"FFFF");

        write_hex_u16(&mut buf, 0x1234);
        assert_eq!(&buf, b"1234");

        write_hex_u16(&mut buf, 0xABCD);
        assert_eq!(&buf, b"ABCD");

        write_hex_u16(&mut buf, 0x0001);
        assert_eq!(&buf, b"0001");
    }

    #[test]
    fn test_write_hex_u8() {
        let mut buf = [0u8; 2];

        write_hex_u8(&mut buf, 0x00);
        assert_eq!(&buf, b"00");

        write_hex_u8(&mut buf, 0xFF);
        assert_eq!(&buf, b"FF");

        write_hex_u8(&mut buf, 0x1A);
        assert_eq!(&buf, b"1A");
    }

    #[test]
    fn test_write_i16() {
        let mut buf = [0u8; 6];

        let len = write_i16(&mut buf, 0);
        assert_eq!(&buf[..len], b"0");

        let len = write_i16(&mut buf, 1);
        assert_eq!(&buf[..len], b"1");

        let len = write_i16(&mut buf, -1);
        assert_eq!(&buf[..len], b"-1");

        let len = write_i16(&mut buf, 32767);
        assert_eq!(&buf[..len], b"32767");

        let len = write_i16(&mut buf, -32768);
        assert_eq!(&buf[..len], b"-32768");

        let len = write_i16(&mut buf, 1000);
        assert_eq!(&buf[..len], b"1000");

        let len = write_i16(&mut buf, -1000);
        assert_eq!(&buf[..len], b"-1000");
    }

    #[test]
    fn test_write_u8() {
        let mut buf = [0u8; 3];

        let len = write_u8(&mut buf, 0);
        assert_eq!(&buf[..len], b"0");

        let len = write_u8(&mut buf, 1);
        assert_eq!(&buf[..len], b"1");

        let len = write_u8(&mut buf, 255);
        assert_eq!(&buf[..len], b"255");

        let len = write_u8(&mut buf, 128);
        assert_eq!(&buf[..len], b"128");

        let len = write_u8(&mut buf, 64);
        assert_eq!(&buf[..len], b"64");
    }

    #[test]
    fn test_calculate_checksum() {
        // Empty payload
        assert_eq!(calculate_checksum(b""), 0);

        // Single byte
        assert_eq!(calculate_checksum(b"A"), b'A');

        // XOR of all bytes
        assert_eq!(calculate_checksum(b"AB"), b'A' ^ b'B');

        // Neutral state payload
        let payload = b"0000:0:0:0:0:0:0";
        let checksum = calculate_checksum(payload);
        // Verify by XORing manually
        let expected = payload.iter().fold(0u8, |acc, &b| acc ^ b);
        assert_eq!(checksum, expected);
    }
}
