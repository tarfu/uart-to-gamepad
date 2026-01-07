//! UART to USB Gamepad bridge for RP2040.
//!
//! This crate provides the embedded implementation of a gamepad bridge
//! that reads gamepad state from UART and outputs it as USB HID.

#![no_std]

// Re-export core types for convenience
pub use gamepad_core::{
    parse, parse_message, AnalogStick, BridgeError, Buttons, GamepadBridge, GamepadFieldUpdate,
    GamepadState, InputError, InputSource, OutputError, OutputSink, ParsedMessage, MAX_LINE_LENGTH,
};

pub mod uart_input;
pub mod usb_output;

pub use uart_input::UartInputSource;
pub use usb_output::{configure_usb_hid, GamepadReport, GamepadRequestHandler, UsbHidOutput};
