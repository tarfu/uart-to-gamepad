//! GamepadBridge: connects input sources to output sinks.

use crate::input::{InputError, InputSource};
use crate::output::{OutputError, OutputSink};
use gamepad_proto::GamepadState;

/// A bridge that forwards gamepad state from an input source to an output sink.
///
/// This abstraction decouples the input and output implementations,
/// making the system more testable and flexible.
///
/// # Error Handling
///
/// On input errors, the bridge sends a neutral gamepad state to prevent
/// stale inputs from persisting.
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
                self.output
                    .send(&state)
                    .await
                    .map_err(BridgeError::Output)?;
                Ok(())
            }
            Err(e) => {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BridgeError {
    /// Error from the input source.
    Input(InputError),
    /// Error from the output sink.
    Output(OutputError),
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use gamepad_proto::Buttons;
    use std::sync::{Arc, Mutex};
    use std::vec;
    use std::vec::Vec;

    // Simple mock input source
    struct MockInput {
        states: Vec<Result<GamepadState, InputError>>,
        index: usize,
    }

    impl MockInput {
        fn new(states: Vec<Result<GamepadState, InputError>>) -> Self {
            Self { states, index: 0 }
        }
    }

    impl InputSource for MockInput {
        fn receive(&mut self) -> impl Future<Output = Result<GamepadState, InputError>> {
            let result = if self.index < self.states.len() {
                let r = self.states[self.index].clone();
                self.index += 1;
                r
            } else {
                Err(InputError::Disconnected)
            };
            core::future::ready(result)
        }

        fn is_connected(&self) -> bool {
            self.index < self.states.len()
        }
    }

    // Simple mock output sink
    struct MockOutput {
        sent: Arc<Mutex<Vec<GamepadState>>>,
    }

    impl MockOutput {
        fn new() -> Self {
            Self {
                sent: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl OutputSink for MockOutput {
        fn send(&mut self, state: &GamepadState) -> impl Future<Output = Result<(), OutputError>> {
            self.sent.lock().unwrap().push(*state);
            core::future::ready(Ok(()))
        }

        fn is_ready(&self) -> bool {
            true
        }
    }

    // Helper to run a future to completion (simple blocking executor)
    fn block_on<F: Future>(mut f: F) -> F::Output {
        fn noop_raw_waker() -> RawWaker {
            fn noop(_: *const ()) {}
            fn clone(_: *const ()) -> RawWaker {
                noop_raw_waker()
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            RawWaker::new(core::ptr::null(), &VTABLE)
        }

        let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
        let mut cx = Context::from_waker(&waker);

        // SAFETY: We don't move f after pinning
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        loop {
            match f.as_mut().poll(&mut cx) {
                Poll::Ready(result) => return result,
                Poll::Pending => {
                    panic!("Mock future returned Pending unexpectedly");
                }
            }
        }
    }

    #[test]
    fn test_bridge_forwards_state() {
        let mut state = GamepadState::neutral();
        state.buttons = Buttons::A | Buttons::B;
        state.left_stick.x = 1000;

        let input = MockInput::new(vec![Ok(state)]);
        let output = MockOutput::new();
        let sent_ref = output.sent.clone();

        let mut bridge = GamepadBridge::new(input, output);

        let result = block_on(bridge.process_one());
        assert!(result.is_ok());

        let sent = sent_ref.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], state);
    }

    #[test]
    fn test_bridge_sends_neutral_on_error() {
        let input = MockInput::new(vec![Err(InputError::Parse)]);
        let output = MockOutput::new();
        let sent_ref = output.sent.clone();

        let mut bridge = GamepadBridge::new(input, output);

        let result = block_on(bridge.process_one());
        assert!(matches!(result, Err(BridgeError::Input(InputError::Parse))));

        let sent = sent_ref.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], GamepadState::neutral());
    }
}
