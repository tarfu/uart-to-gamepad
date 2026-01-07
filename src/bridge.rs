use crate::gamepad::GamepadState;
use crate::input::{InputError, InputSource};
use crate::output::{OutputError, OutputSink};
use defmt::{error, trace};

/// A bridge that forwards gamepad state from an input source to an output sink.
///
/// This abstraction decouples the input and output implementations,
/// making the system more testable and flexible.
///
/// # Error Handling
///
/// On input errors, the bridge sends a neutral gamepad state to prevent
/// stale inputs from persisting. Errors are logged via defmt.
pub struct GamepadBridge<I, O> {
    input: I,
    output: O,
}

impl<I: InputSource, O: OutputSink> GamepadBridge<I, O> {
    /// Create a new bridge from an input source and output sink.
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }

    /// Run the bridge, forwarding gamepad state indefinitely.
    ///
    /// This method never returns under normal operation.
    pub async fn run(&mut self) -> ! {
        loop {
            let _ = self.process_one().await;
        }
    }

    /// Process a single input and forward it to the output.
    ///
    /// Returns the result of the operation for testing purposes.
    pub async fn process_one(&mut self) -> Result<(), BridgeError> {
        match self.input.receive().await {
            Ok(state) => {
                trace!("Received gamepad state: {:?}", state);
                self.output
                    .send(&state)
                    .await
                    .map_err(BridgeError::Output)?;
                Ok(())
            }
            Err(e) => {
                error!("Input error: {:?}", e);
                // Send neutral state to prevent stale inputs
                let _ = self.output.send(&GamepadState::neutral()).await;
                Err(BridgeError::Input(e))
            }
        }
    }

    /// Get a reference to the input source.
    pub fn input(&self) -> &I {
        &self.input
    }

    /// Get a mutable reference to the input source.
    pub fn input_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Get a reference to the output sink.
    pub fn output(&self) -> &O {
        &self.output
    }

    /// Get a mutable reference to the output sink.
    pub fn output_mut(&mut self) -> &mut O {
        &mut self.output
    }

    /// Decompose the bridge into its input and output components.
    pub fn into_parts(self) -> (I, O) {
        (self.input, self.output)
    }
}

/// Error type for bridge operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum BridgeError {
    /// Error from the input source.
    Input(InputError),
    /// Error from the output sink.
    Output(OutputError),
}
