//! Channel-to-gamepad mapping configuration.
//!
//! CRSF provides 16 RC channels (0-15) with 11-bit resolution (0-1984).
//! This module maps those channels to gamepad controls.

use gamepad_core::{AnalogStick, Buttons, GamepadState};

/// Channel mapping configuration for CRSF to gamepad conversion.
///
/// Customize this at compile-time by creating your own const.
#[derive(Debug, Clone, Copy)]
pub struct ChannelMapping {
    /// Channel index for right stick X axis (typically Roll/Aileron).
    pub right_stick_x: usize,
    /// Channel index for right stick Y axis (typically Pitch/Elevator).
    pub right_stick_y: usize,
    /// Channel index for left stick X axis (typically Yaw/Rudder).
    pub left_stick_x: usize,
    /// Channel index for left stick Y axis (optional, often unused).
    pub left_stick_y: usize,
    /// Channel index for left trigger (typically Throttle).
    pub left_trigger: usize,
    /// Channel index for right trigger (optional auxiliary).
    pub right_trigger: usize,
    /// Channel indices for button mapping (aux channels).
    /// Channels above threshold (992) are considered pressed.
    pub button_channels: [usize; 8],
    /// Invert right stick X axis.
    pub invert_right_x: bool,
    /// Invert right stick Y axis.
    pub invert_right_y: bool,
    /// Invert left stick X axis.
    pub invert_left_x: bool,
    /// Invert left stick Y axis.
    pub invert_left_y: bool,
}

/// Default RC channel mapping following standard conventions.
///
/// - CH1 (Roll) -> Right Stick X
/// - CH2 (Pitch) -> Right Stick Y
/// - CH3 (Throttle) -> Left Trigger
/// - CH4 (Yaw) -> Left Stick X
/// - CH5-CH12 -> Buttons (aux switches)
pub const DEFAULT_MAPPING: ChannelMapping = ChannelMapping {
    right_stick_x: 0,  // CH1 - Roll
    right_stick_y: 1,  // CH2 - Pitch
    left_stick_x: 3,   // CH4 - Yaw
    left_stick_y: 2,   // CH3 - Throttle (as stick, optional)
    left_trigger: 2,   // CH3 - Throttle (as trigger)
    right_trigger: 4,  // CH5 - Aux 1
    button_channels: [5, 6, 7, 8, 9, 10, 11, 12],
    invert_right_x: false,
    invert_right_y: false,
    invert_left_x: false,
    invert_left_y: false,
};

/// CRSF channel center value (11-bit).
pub const CRSF_CENTER: u16 = 992;

/// CRSF channel maximum value (11-bit).
pub const CRSF_MAX: u16 = 1984;

/// CRSF channel minimum value.
pub const CRSF_MIN: u16 = 0;

/// Button threshold - values above this are considered pressed.
pub const BUTTON_THRESHOLD: u16 = CRSF_CENTER;

/// Convert CRSF channel value (0-1984, center 992) to stick value (-32768 to 32767).
#[inline]
#[must_use]
pub fn crsf_to_stick(val: u16, invert: bool) -> i16 {
    // Center at 992, range 0-1984
    let centered = val as i32 - CRSF_CENTER as i32; // -992 to +992
    let scaled = (centered * 32767 / CRSF_CENTER as i32).clamp(-32768, 32767) as i16;
    if invert { -scaled } else { scaled }
}

/// Convert CRSF channel value (0-1984) to trigger value (0-255).
#[inline]
#[must_use]
pub fn crsf_to_trigger(val: u16) -> u8 {
    ((val as u32 * 255) / CRSF_MAX as u32).min(255) as u8
}

/// Check if a channel value represents a pressed button.
#[inline]
#[must_use]
pub fn crsf_to_button(val: u16) -> bool {
    val > BUTTON_THRESHOLD
}

/// Map CRSF channel data to GamepadState using the provided mapping.
#[must_use]
pub fn channels_to_gamepad(channels: &[u16; 16], mapping: &ChannelMapping) -> GamepadState {
    // Map analog sticks
    let left_stick = AnalogStick {
        x: crsf_to_stick(channels[mapping.left_stick_x], mapping.invert_left_x),
        y: crsf_to_stick(channels[mapping.left_stick_y], mapping.invert_left_y),
    };

    let right_stick = AnalogStick {
        x: crsf_to_stick(channels[mapping.right_stick_x], mapping.invert_right_x),
        y: crsf_to_stick(channels[mapping.right_stick_y], mapping.invert_right_y),
    };

    // Map triggers
    let left_trigger = crsf_to_trigger(channels[mapping.left_trigger]);
    let right_trigger = crsf_to_trigger(channels[mapping.right_trigger]);

    // Map buttons from aux channels
    let mut buttons = Buttons::NONE;
    let button_flags = [
        Buttons::A,
        Buttons::B,
        Buttons::X,
        Buttons::Y,
        Buttons::LB,  // Left bumper
        Buttons::RB,  // Right bumper
        Buttons::BACK,
        Buttons::START,
    ];

    for (i, &channel_idx) in mapping.button_channels.iter().enumerate() {
        if channel_idx < 16 && crsf_to_button(channels[channel_idx]) {
            if let Some(&button) = button_flags.get(i) {
                buttons |= button;
            }
        }
    }

    GamepadState {
        buttons,
        left_stick,
        right_stick,
        left_trigger,
        right_trigger,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crsf_to_stick_center() {
        assert_eq!(crsf_to_stick(CRSF_CENTER, false), 0);
    }

    #[test]
    fn test_crsf_to_stick_min() {
        assert_eq!(crsf_to_stick(CRSF_MIN, false), -32767);
    }

    #[test]
    fn test_crsf_to_stick_max() {
        assert_eq!(crsf_to_stick(CRSF_MAX, false), 32767);
    }

    #[test]
    fn test_crsf_to_stick_invert() {
        assert_eq!(crsf_to_stick(CRSF_MAX, true), -32767);
        assert_eq!(crsf_to_stick(CRSF_MIN, true), 32767);
    }

    #[test]
    fn test_crsf_to_trigger() {
        assert_eq!(crsf_to_trigger(CRSF_MIN), 0);
        assert_eq!(crsf_to_trigger(CRSF_MAX), 255);
        assert_eq!(crsf_to_trigger(CRSF_CENTER), 127); // ~half
    }

    #[test]
    fn test_crsf_to_button() {
        assert!(!crsf_to_button(CRSF_MIN));
        assert!(!crsf_to_button(CRSF_CENTER));
        assert!(crsf_to_button(CRSF_CENTER + 1));
        assert!(crsf_to_button(CRSF_MAX));
    }
}
