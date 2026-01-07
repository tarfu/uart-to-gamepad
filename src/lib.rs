#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

pub mod bridge;
pub mod gamepad;
pub mod input;
pub mod output;

pub use bridge::{BridgeError, GamepadBridge};
pub use gamepad::{GamepadFieldUpdate, GamepadState};
pub use input::{InputError, InputSource};
pub use output::{OutputError, OutputSink};
