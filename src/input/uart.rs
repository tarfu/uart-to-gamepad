use crate::gamepad::GamepadState;
use crate::input::parser::{parse_message, ParsedMessage, MAX_LINE_LENGTH};
use crate::input::traits::{InputError, InputSource};
use embassy_rp::uart::{Async, Error as UartError, UartRx};
use heapless::Vec;

/// UART-based input source for receiving gamepad state.
///
/// Reads line-based protocol messages from UART and parses them into
/// [`GamepadState`] values. Supports both full state messages (G prefix)
/// and incremental update messages (U prefix).
///
/// # Protocol
///
/// Full state: `G<buttons>:<lx>:<ly>:<rx>:<ry>:<lt>:<rt>*<checksum>\n`
/// Update: `U<field>:<value>*<checksum>\n`
///
/// See [`parse_message`] for full protocol specification.
///
/// # Pins
///
/// Uses UART1 by default:
/// - GPIO 8: TX
/// - GPIO 9: RX
/// - GPIO 10: CTS (optional, with `uart-flow-control` feature)
/// - GPIO 11: RTS (optional, with `uart-flow-control` feature)
pub struct UartInputSource<'d> {
    rx: UartRx<'d, Async>,
    buffer: Vec<u8, MAX_LINE_LENGTH>,
    /// Current gamepad state (updated incrementally or replaced fully)
    state: GamepadState,
}

impl<'d> UartInputSource<'d> {
    /// Create a new UART input source from the given UART receiver.
    pub fn new(rx: UartRx<'d, Async>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            state: GamepadState::neutral(),
        }
    }

    /// Get the current gamepad state.
    #[inline]
    #[must_use]
    pub fn current_state(&self) -> &GamepadState {
        &self.state
    }

    /// Read bytes until a newline is found or buffer is full.
    ///
    /// If a line exceeds the buffer capacity, the rest of the line is
    /// discarded to prevent cascading parse errors on subsequent reads.
    async fn read_line(&mut self) -> Result<(), InputError> {
        self.buffer.clear();

        loop {
            let mut byte = [0u8; 1];
            self.rx.read(&mut byte).await?;

            if byte[0] == b'\n' {
                return Ok(());
            }

            if self.buffer.push(byte[0]).is_err() {
                // Buffer overflow - discard rest of line until newline
                loop {
                    self.rx.read(&mut byte).await?;
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                return Err(InputError::BufferOverflow);
            }
        }
    }
}

impl<'d> InputSource for UartInputSource<'d> {
    async fn receive(&mut self) -> Result<GamepadState, InputError> {
        self.read_line().await?;

        match parse_message(&self.buffer)? {
            ParsedMessage::FullState(state) => {
                self.state = state;
            }
            ParsedMessage::Update(update) => {
                self.state.apply_update(update);
            }
        }

        Ok(self.state)
    }

    fn is_connected(&self) -> bool {
        // UART is always "connected" if we have the peripheral
        true
    }
}

/// Convert UART errors to InputError using the From trait.
impl From<UartError> for InputError {
    fn from(e: UartError) -> Self {
        match e {
            UartError::Framing => InputError::Framing,
            UartError::Break => InputError::Io,
            UartError::Overrun => InputError::BufferOverflow,
            UartError::Parity => InputError::Io,
            _ => InputError::Io,
        }
    }
}
