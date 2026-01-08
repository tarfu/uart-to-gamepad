//! CRC-8 checksum for protocol messages.
//!
//! Uses CRC-8/SMBUS algorithm with a 256-byte lookup table for fast calculation.

use crc::{Crc, CRC_8_SMBUS};

/// CRC-8/SMBUS calculator with 256-byte lookup table.
const CRC8: Crc<u8> = Crc::<u8>::new(&CRC_8_SMBUS);

/// Calculate CRC-8 checksum of a byte slice.
#[inline]
#[must_use]
pub fn calculate_crc8(data: &[u8]) -> u8 {
    CRC8.checksum(data)
}

/// CRC-8 digest for incremental calculation.
///
/// Use this when building a message byte-by-byte (e.g., during serialization).
pub struct Crc8Digest {
    digest: crc::Digest<'static, u8>,
}

impl Crc8Digest {
    /// Create a new CRC-8 digest.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            digest: CRC8.digest(),
        }
    }

    /// Update the digest with a single byte.
    #[inline]
    pub fn update(&mut self, byte: u8) {
        self.digest.update(&[byte]);
    }

    /// Update the digest with a byte slice.
    #[inline]
    pub fn update_slice(&mut self, data: &[u8]) {
        self.digest.update(data);
    }

    /// Finalize and return the checksum value.
    #[inline]
    #[must_use]
    pub fn finalize(self) -> u8 {
        self.digest.finalize()
    }
}

impl Default for Crc8Digest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc8_empty() {
        assert_eq!(calculate_crc8(&[]), 0x00);
    }

    #[test]
    fn test_crc8_single_byte() {
        // CRC-8/SMBUS of [0x00] should be 0x00
        assert_eq!(calculate_crc8(&[0x00]), 0x00);
    }

    #[test]
    fn test_crc8_digest_matches_batch() {
        let data = b"0000:0:0:0:0:0:0";
        let batch_crc = calculate_crc8(data);

        let mut digest = Crc8Digest::new();
        for &b in data {
            digest.update(b);
        }
        let incremental_crc = digest.finalize();

        assert_eq!(batch_crc, incremental_crc);
    }

    #[test]
    fn test_crc8_digest_slice() {
        let data = b"test data";
        let batch_crc = calculate_crc8(data);

        let mut digest = Crc8Digest::new();
        digest.update_slice(data);
        let slice_crc = digest.finalize();

        assert_eq!(batch_crc, slice_crc);
    }
}
