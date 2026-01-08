//! MAVLink input source implementation.
//!
//! Receives MAVLink MANUAL_CONTROL messages from UART and converts to GamepadState.

use embassy_rp::uart::{Async, UartRx};
use embassy_time::{Duration, Instant};
use gamepad_core::{GamepadState, InputError, InputSource};
use mavlink_proto::{
    manual_control_to_gamepad, AxisMapping, MavlinkParser, MavMessage, DEFAULT_AXIS_MAPPING,
};

/// MAVLink system ID for this device.
pub const DEFAULT_SYSTEM_ID: u8 = 1;

/// MAVLink component ID for this device.
pub const DEFAULT_COMPONENT_ID: u8 = 1;

/// Heartbeat interval.
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);

/// Connection timeout (if no messages received).
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// MAVLink input source (RX only).
///
/// Receives MANUAL_CONTROL messages and converts them to GamepadState.
/// Uses a minimal built-in MAVLink parser to avoid atomic limitations
/// on Cortex-M0 targets.
///
/// Note: Heartbeat sending is not implemented. The ground station should
/// continue sending even without heartbeat responses from this device.
pub struct MavlinkInputSource<'d> {
    /// UART receiver.
    rx: UartRx<'d, Async>,
    /// MAVLink parser.
    parser: MavlinkParser,
    /// Current gamepad state.
    state: GamepadState,
    /// Axis mapping configuration.
    mapping: AxisMapping,
    /// Last received message time.
    last_message: Option<Instant>,
    /// Target system ID to accept messages from (0 = any).
    target_system: u8,
}

impl<'d> MavlinkInputSource<'d> {
    /// Create a new MAVLink input source with default mapping.
    #[must_use]
    pub fn new(rx: UartRx<'d, Async>) -> Self {
        Self::with_mapping(rx, DEFAULT_AXIS_MAPPING)
    }

    /// Create a new MAVLink input source with custom axis mapping.
    #[must_use]
    pub fn with_mapping(rx: UartRx<'d, Async>, mapping: AxisMapping) -> Self {
        Self {
            rx,
            parser: MavlinkParser::new(),
            state: GamepadState::neutral(),
            mapping,
            last_message: None,
            target_system: 0, // Accept from any system
        }
    }

    /// Set target system ID to filter messages (0 = accept all).
    pub fn set_target_system(&mut self, system_id: u8) {
        self.target_system = system_id;
    }

    /// Read and process bytes until we get a MANUAL_CONTROL message.
    async fn read_next_manual_control(&mut self) -> Result<GamepadState, InputError> {
        let mut byte_buf = [0u8; 1];

        loop {
            // Read one byte
            self.rx
                .read(&mut byte_buf)
                .await
                .map_err(|_| InputError::Io)?;

            // Feed to parser
            match self.parser.push_byte(byte_buf[0]) {
                Ok(Some(message)) => {
                    self.last_message = Some(Instant::now());

                    match message {
                        MavMessage::ManualControl(msg) => {
                            // Check if message is for us (target = 0 means broadcast)
                            if self.target_system == 0 || msg.target == self.target_system {
                                self.state = manual_control_to_gamepad(
                                    msg.x,
                                    msg.y,
                                    msg.z,
                                    msg.r,
                                    msg.buttons,
                                    msg.buttons2,
                                    &self.mapping,
                                );
                                return Ok(self.state);
                            }
                        }
                        MavMessage::Heartbeat => {
                            // Heartbeat received - connection is alive
                            // Continue waiting for MANUAL_CONTROL
                        }
                        MavMessage::Unknown(_) => {
                            // Ignore unknown messages
                        }
                    }
                }
                Ok(None) => {
                    // Incomplete message, continue reading
                }
                Err(_) => {
                    // Parse error, parser has been reset, continue
                }
            }
        }
    }
}

impl InputSource for MavlinkInputSource<'_> {
    async fn receive(&mut self) -> Result<GamepadState, InputError> {
        self.read_next_manual_control().await
    }

    fn is_connected(&self) -> bool {
        if let Some(last) = self.last_message {
            Instant::now().duration_since(last) < CONNECTION_TIMEOUT
        } else {
            false
        }
    }
}
