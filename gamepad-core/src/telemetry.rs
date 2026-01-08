//! Telemetry types and traits for bidirectional protocol support.
//!
//! This module provides abstractions for sending telemetry data back through
//! input protocols that support bidirectional communication (CRSF, MAVLink).

use core::future::Future;

/// Telemetry data that can be sent back to transmitter/GCS.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TelemetryData {
    /// Battery status telemetry.
    Battery {
        /// Voltage in millivolts.
        voltage_mv: u16,
        /// Current draw in milliamps.
        current_ma: u16,
        /// Remaining capacity percentage (0-100).
        remaining_pct: u8,
    },
    /// GPS position telemetry.
    Gps {
        /// Latitude in degrees * 1e7.
        lat: i32,
        /// Longitude in degrees * 1e7.
        lon: i32,
        /// Altitude in meters.
        alt_m: i16,
        /// Ground speed in m/s.
        speed_mps: u8,
        /// Number of satellites.
        sats: u8,
    },
    /// Attitude (orientation) telemetry.
    Attitude {
        /// Roll angle in degrees * 100.
        roll: i16,
        /// Pitch angle in degrees * 100.
        pitch: i16,
        /// Yaw angle in degrees * 100.
        yaw: i16,
    },
    /// RF link quality telemetry.
    LinkQuality {
        /// Received signal strength indicator (dBm, typically negative).
        rssi: i8,
        /// Signal-to-noise ratio in dB.
        snr: i8,
        /// Link quality percentage (0-100).
        lq: u8,
    },
}

/// Error type for telemetry operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TelemetryError {
    /// Telemetry not supported by this protocol/transport.
    NotSupported,
    /// I/O error during transmission.
    Io,
    /// Transmit buffer is full.
    BufferFull,
}

/// Trait for sending telemetry data back through the input channel.
///
/// Implement this for input sources that support bidirectional communication.
pub trait TelemetrySink {
    /// Send telemetry data.
    fn send_telemetry(
        &mut self,
        data: &TelemetryData,
    ) -> impl Future<Output = Result<(), TelemetryError>>;

    /// Check if this sink supports telemetry transmission.
    ///
    /// Returns `false` by default for protocols that don't support backchannel.
    fn supports_telemetry(&self) -> bool {
        false
    }
}

/// Trait for receiving telemetry data from an external source.
///
/// This allows telemetry to come from various places:
/// - USB host sending telemetry to forward
/// - Secondary UART connected to sensors
/// - Mock/test data
pub trait TelemetrySource {
    /// Try to receive the next telemetry data.
    ///
    /// Returns `None` if no telemetry is available.
    fn receive(&mut self) -> impl Future<Output = Option<TelemetryData>>;
}

/// Mock telemetry source that never produces data.
///
/// Use this as a placeholder until real telemetry sources are implemented.
pub struct MockTelemetrySource;

impl TelemetrySource for MockTelemetrySource {
    async fn receive(&mut self) -> Option<TelemetryData> {
        // Never produces telemetry
        core::future::pending().await
    }
}

/// Null telemetry sink that discards all data.
///
/// Use this for protocols that don't support telemetry backchannel.
pub struct NullTelemetrySink;

impl TelemetrySink for NullTelemetrySink {
    async fn send_telemetry(&mut self, _data: &TelemetryData) -> Result<(), TelemetryError> {
        Err(TelemetryError::NotSupported)
    }

    fn supports_telemetry(&self) -> bool {
        false
    }
}
