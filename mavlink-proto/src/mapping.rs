//! MAVLink MANUAL_CONTROL to gamepad mapping.
//!
//! Maps MAVLink joystick axes and buttons to GamepadState.

use gamepad_core::{AnalogStick, Buttons, GamepadState};

/// Axis mapping configuration for MAVLink to gamepad conversion.
///
/// MAVLink MANUAL_CONTROL uses:
/// - x: pitch (forward/back)
/// - y: roll (left/right)
/// - z: thrust (0-1000, where 500 is neutral for some, 0 is min for others)
/// - r: yaw (rotation)
#[derive(Debug, Clone, Copy)]
pub struct AxisMapping {
    /// Invert X axis (pitch).
    pub invert_x: bool,
    /// Invert Y axis (roll).
    pub invert_y: bool,
    /// Invert Z axis (thrust).
    pub invert_z: bool,
    /// Invert R axis (yaw).
    pub invert_r: bool,
    /// Use Z axis as left trigger (true) or left stick Y (false).
    pub z_as_trigger: bool,
}

/// Default axis mapping.
pub const DEFAULT_AXIS_MAPPING: AxisMapping = AxisMapping {
    invert_x: false,
    invert_y: false,
    invert_z: false,
    invert_r: false,
    z_as_trigger: true,  // Use Z (thrust) as left trigger
};

/// MAVLink axis range.
pub const MAVLINK_AXIS_MIN: i16 = -1000;
pub const MAVLINK_AXIS_MAX: i16 = 1000;

/// MAVLink Z axis range (thrust is typically 0-1000).
pub const MAVLINK_Z_MIN: i16 = 0;
pub const MAVLINK_Z_MAX: i16 = 1000;

/// Convert MAVLink axis value (-1000 to 1000) to stick value (-32768 to 32767).
#[inline]
#[must_use]
pub fn mavlink_to_stick(val: i16, invert: bool) -> i16 {
    // Scale from -1000..1000 to -32768..32767
    let scaled = (val as i32 * 32767 / 1000).clamp(-32768, 32767) as i16;
    if invert { -scaled } else { scaled }
}

/// Convert MAVLink Z axis (0-1000) to trigger value (0-255).
#[inline]
#[must_use]
pub fn mavlink_z_to_trigger(z: i16) -> u8 {
    // Z is 0-1000 for thrust, map to 0-255
    let clamped = z.clamp(0, 1000) as u32;
    ((clamped * 255) / 1000) as u8
}

/// Convert MAVLink buttons bitfield to Buttons.
#[inline]
#[must_use]
pub fn mavlink_to_buttons(buttons: u16, buttons2: u16) -> Buttons {
    // MAVLink button bits map directly, but we may want to remap
    // For now, use lower 16 bits directly (buttons field)
    // buttons2 provides buttons 16-31 which we could map to other functions

    let mut result = Buttons::NONE;

    // Map first 8 buttons to standard gamepad buttons
    if buttons & (1 << 0) != 0 {
        result |= Buttons::A;
    }
    if buttons & (1 << 1) != 0 {
        result |= Buttons::B;
    }
    if buttons & (1 << 2) != 0 {
        result |= Buttons::X;
    }
    if buttons & (1 << 3) != 0 {
        result |= Buttons::Y;
    }
    if buttons & (1 << 4) != 0 {
        result |= Buttons::LB;
    }
    if buttons & (1 << 5) != 0 {
        result |= Buttons::RB;
    }
    if buttons & (1 << 6) != 0 {
        result |= Buttons::BACK;
    }
    if buttons & (1 << 7) != 0 {
        result |= Buttons::START;
    }

    // Map buttons 8-15 to d-pad and stick presses
    if buttons & (1 << 8) != 0 {
        result |= Buttons::GUIDE;
    }
    if buttons & (1 << 9) != 0 {
        result |= Buttons::LS;
    }
    if buttons & (1 << 10) != 0 {
        result |= Buttons::RS;
    }
    if buttons & (1 << 11) != 0 {
        result |= Buttons::DPAD_UP;
    }
    if buttons & (1 << 12) != 0 {
        result |= Buttons::DPAD_DOWN;
    }
    if buttons & (1 << 13) != 0 {
        result |= Buttons::DPAD_LEFT;
    }
    if buttons & (1 << 14) != 0 {
        result |= Buttons::DPAD_RIGHT;
    }

    // buttons2 could be used for additional mappings if needed
    let _ = buttons2;

    result
}

/// Convert MAVLink MANUAL_CONTROL fields to GamepadState.
#[must_use]
pub fn manual_control_to_gamepad(
    x: i16,
    y: i16,
    z: i16,
    r: i16,
    buttons: u16,
    buttons2: u16,
    mapping: &AxisMapping,
) -> GamepadState {
    // Standard mapping:
    // x (pitch) -> Right Stick Y (inverted: forward = up)
    // y (roll) -> Right Stick X
    // r (yaw) -> Left Stick X
    // z (thrust) -> Left Trigger or Left Stick Y

    let right_stick = AnalogStick {
        x: mavlink_to_stick(y, mapping.invert_y),      // roll -> X
        y: mavlink_to_stick(-x, mapping.invert_x),     // pitch -> Y (inverted)
    };

    let (left_stick, left_trigger) = if mapping.z_as_trigger {
        // Z as trigger, R as left stick X
        let stick = AnalogStick {
            x: mavlink_to_stick(r, mapping.invert_r),
            y: 0,  // No Y axis when Z is trigger
        };
        let trigger = mavlink_z_to_trigger(if mapping.invert_z { 1000 - z } else { z });
        (stick, trigger)
    } else {
        // Z as left stick Y
        let stick = AnalogStick {
            x: mavlink_to_stick(r, mapping.invert_r),
            y: mavlink_to_stick(z, mapping.invert_z),
        };
        (stick, 0u8)
    };

    GamepadState {
        buttons: mavlink_to_buttons(buttons, buttons2),
        left_stick,
        right_stick,
        left_trigger,
        right_trigger: 0,  // Could map from aux fields if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_to_stick_center() {
        assert_eq!(mavlink_to_stick(0, false), 0);
    }

    #[test]
    fn test_mavlink_to_stick_extremes() {
        assert_eq!(mavlink_to_stick(1000, false), 32767);
        assert_eq!(mavlink_to_stick(-1000, false), -32767);
    }

    #[test]
    fn test_mavlink_to_stick_invert() {
        assert_eq!(mavlink_to_stick(1000, true), -32767);
        assert_eq!(mavlink_to_stick(-1000, true), 32767);
    }

    #[test]
    fn test_mavlink_z_to_trigger() {
        assert_eq!(mavlink_z_to_trigger(0), 0);
        assert_eq!(mavlink_z_to_trigger(1000), 255);
        assert_eq!(mavlink_z_to_trigger(500), 127);
    }

    #[test]
    fn test_mavlink_to_buttons() {
        let buttons = mavlink_to_buttons(0b0000_0001, 0);
        assert!(buttons.contains(Buttons::A));

        let buttons = mavlink_to_buttons(0b0000_1111, 0);
        assert!(buttons.contains(Buttons::A));
        assert!(buttons.contains(Buttons::B));
        assert!(buttons.contains(Buttons::X));
        assert!(buttons.contains(Buttons::Y));
    }
}
