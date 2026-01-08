//! CRSF input source implementation.
//!
//! Receives CRSF frames from UART and converts them to GamepadState.

use crsf_proto::{channels_to_gamepad, ChannelMapping, CrsfParser, Packet, DEFAULT_MAPPING};
use embassy_rp::uart::{Async, Uart, UartRx};
use gamepad_core::{GamepadState, InputError, InputSource};

/// CRSF input source for receiving RC channel data.
///
/// Parses CRSF frames from UART and converts channel data to GamepadState.
pub struct CrsfInputSource<'d> {
    /// UART receiver (RX only for basic input, full Uart for telemetry).
    rx: UartRx<'d, Async>,
    /// CRSF frame parser.
    parser: CrsfParser,
    /// Current gamepad state (updated on each RC packet).
    state: GamepadState,
    /// Channel-to-gamepad mapping configuration.
    mapping: ChannelMapping,
    /// Connection status (true if we've received valid packets recently).
    connected: bool,
}

impl<'d> CrsfInputSource<'d> {
    /// Create a new CRSF input source with default channel mapping.
    ///
    /// # Arguments
    /// * `rx` - UART receiver configured for 420000 baud
    #[must_use]
    pub fn new(rx: UartRx<'d, Async>) -> Self {
        Self::with_mapping(rx, DEFAULT_MAPPING)
    }

    /// Create a new CRSF input source with custom channel mapping.
    ///
    /// # Arguments
    /// * `rx` - UART receiver configured for 420000 baud
    /// * `mapping` - Custom channel-to-gamepad mapping
    #[must_use]
    pub fn with_mapping(rx: UartRx<'d, Async>, mapping: ChannelMapping) -> Self {
        Self {
            rx,
            parser: CrsfParser::new(),
            state: GamepadState::neutral(),
            mapping,
            connected: false,
        }
    }

    /// Process incoming bytes until we get an RC channels packet.
    async fn read_next_rc_packet(&mut self) -> Result<[u16; 16], InputError> {
        let mut byte_buf = [0u8; 1];

        loop {
            // Read one byte at a time
            self.rx
                .read(&mut byte_buf)
                .await
                .map_err(|_| InputError::Io)?;

            // Feed to parser
            match self.parser.push_byte(byte_buf[0]) {
                Ok(Some(packet)) => {
                    // Got a complete packet - check if it's RC channels
                    if let Packet::RCChannels(rc) = packet {
                        self.connected = true;
                        return Ok(rc.0);
                    }
                    // Other packet types are ignored for now
                    // (could be used for link statistics, etc.)
                }
                Ok(None) => {
                    // Incomplete packet, continue reading
                }
                Err(_) => {
                    // Parse error - reset parser and continue
                    self.parser.reset();
                    // Don't return error, just try again
                }
            }
        }
    }
}

impl InputSource for CrsfInputSource<'_> {
    async fn receive(&mut self) -> Result<GamepadState, InputError> {
        // Wait for next RC channels packet
        let channels = self.read_next_rc_packet().await?;

        // Convert to gamepad state
        self.state = channels_to_gamepad(&channels, &self.mapping);

        Ok(self.state)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// CRSF input source with full UART for bidirectional communication.
///
/// Use this when you need telemetry backchannel support.
pub struct CrsfBidirectionalSource<'d> {
    /// Full UART for TX and RX.
    uart: Uart<'d, Async>,
    /// CRSF frame parser.
    parser: CrsfParser,
    /// Current gamepad state.
    state: GamepadState,
    /// Channel mapping configuration.
    mapping: ChannelMapping,
    /// Connection status.
    connected: bool,
}

impl<'d> CrsfBidirectionalSource<'d> {
    /// Create a new bidirectional CRSF source with default mapping.
    #[must_use]
    pub fn new(uart: Uart<'d, Async>) -> Self {
        Self::with_mapping(uart, DEFAULT_MAPPING)
    }

    /// Create a new bidirectional CRSF source with custom mapping.
    #[must_use]
    pub fn with_mapping(uart: Uart<'d, Async>, mapping: ChannelMapping) -> Self {
        Self {
            uart,
            parser: CrsfParser::new(),
            state: GamepadState::neutral(),
            mapping,
            connected: false,
        }
    }

    /// Get mutable access to the UART for telemetry transmission.
    pub fn uart_mut(&mut self) -> &mut Uart<'d, Async> {
        &mut self.uart
    }

    /// Process incoming bytes until we get an RC channels packet.
    async fn read_next_rc_packet(&mut self) -> Result<[u16; 16], InputError> {
        let mut byte_buf = [0u8; 1];

        loop {
            self.uart
                .read(&mut byte_buf)
                .await
                .map_err(|_| InputError::Io)?;

            match self.parser.push_byte(byte_buf[0]) {
                Ok(Some(packet)) => {
                    if let Packet::RCChannels(rc) = packet {
                        self.connected = true;
                        return Ok(rc.0);
                    }
                }
                Ok(None) => {}
                Err(_) => {
                    self.parser.reset();
                }
            }
        }
    }
}

impl InputSource for CrsfBidirectionalSource<'_> {
    async fn receive(&mut self) -> Result<GamepadState, InputError> {
        let channels = self.read_next_rc_packet().await?;
        self.state = channels_to_gamepad(&channels, &self.mapping);
        Ok(self.state)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
