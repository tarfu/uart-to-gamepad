//! CRSF telemetry encoding.
//!
//! Converts TelemetryData to CRSF packet format for transmission.

use gamepad_core::{TelemetryData, TelemetryError};
use uf_crsf::packets::{write_packet_to_buffer, Attitude, Battery, Gps, PacketAddress};

/// Convert TelemetryData to CRSF packets and write to buffer.
///
/// This is a chip-agnostic function that encodes telemetry data into
/// CRSF packet format. The caller is responsible for transmitting the
/// resulting bytes via UART.
///
/// # Arguments
///
/// * `data` - The telemetry data to encode
/// * `buf` - Buffer to write the encoded packet into (must be at least 64 bytes)
///
/// # Returns
///
/// The number of bytes written to the buffer, or an error.
pub fn encode_telemetry(data: &TelemetryData, buf: &mut [u8]) -> Result<usize, TelemetryError> {
    match data {
        TelemetryData::Battery {
            voltage_mv,
            current_ma,
            remaining_pct,
        } => {
            // Convert from our units to CRSF units
            // voltage: mV -> 10mV (divide by 10)
            // current: mA -> 10mA (divide by 10)
            let voltage = (*voltage_mv / 10) as i16;
            let current = (*current_ma / 10) as i16;
            let packet =
                Battery::new(voltage, current, 0, *remaining_pct).map_err(|_| TelemetryError::Io)?;
            write_packet_to_buffer(buf, PacketAddress::FlightController, &packet)
                .map_err(|_| TelemetryError::BufferFull)
        }

        TelemetryData::Gps {
            lat,
            lon,
            alt_m,
            speed_mps,
            sats,
        } => {
            // Convert from our units to CRSF units
            // lat/lon: already in 1e7 degrees
            // speed: m/s -> 0.01 km/h (multiply by 360)
            // alt: meters -> with 1000m offset
            let groundspeed = (*speed_mps as u16) * 360;
            let altitude = (*alt_m as u16).saturating_add(1000);
            let packet = Gps::new(*lat, *lon, groundspeed, 0, altitude, *sats)
                .map_err(|_| TelemetryError::Io)?;
            write_packet_to_buffer(buf, PacketAddress::FlightController, &packet)
                .map_err(|_| TelemetryError::BufferFull)
        }

        TelemetryData::Attitude { roll, pitch, yaw } => {
            // Convert from degrees*100 to radians*10000
            // degrees*100 -> radians*10000: multiply by (pi/180) * 100 = ~1.745
            let roll_rad = ((*roll as i32) * 1745 / 1000) as i16;
            let pitch_rad = ((*pitch as i32) * 1745 / 1000) as i16;
            let yaw_rad = ((*yaw as i32) * 1745 / 1000) as i16;
            let packet = Attitude::new(roll_rad, pitch_rad, yaw_rad).map_err(|_| TelemetryError::Io)?;
            write_packet_to_buffer(buf, PacketAddress::FlightController, &packet)
                .map_err(|_| TelemetryError::BufferFull)
        }

        TelemetryData::LinkQuality { .. } => Err(TelemetryError::NotSupported),
    }
}

/// Maximum size for a CRSF telemetry frame.
pub const MAX_TELEMETRY_FRAME_SIZE: usize = 64;
