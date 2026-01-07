//! Core gamepad types: Buttons, AnalogStick, GamepadState, GamepadFieldUpdate.

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// Button state represented as a bitfield for efficiency.
///
/// Supports up to 16 buttons, with common gamepad buttons pre-defined.
/// Implements bitwise operators for ergonomic button manipulation.
///
/// # Example
///
/// ```
/// use gamepad_core::Buttons;
///
/// let buttons = Buttons::A | Buttons::B;
/// assert!(buttons.contains(Buttons::A));
/// assert!(buttons.contains(Buttons::B));
/// assert!(!buttons.contains(Buttons::X));
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Buttons(pub u16);

impl Buttons {
    // Button constants as Buttons type for type safety
    pub const A: Self = Self(1 << 0);
    pub const B: Self = Self(1 << 1);
    pub const X: Self = Self(1 << 2);
    pub const Y: Self = Self(1 << 3);
    pub const LB: Self = Self(1 << 4); // Left bumper
    pub const RB: Self = Self(1 << 5); // Right bumper
    pub const BACK: Self = Self(1 << 6); // Select/Back
    pub const START: Self = Self(1 << 7);
    pub const GUIDE: Self = Self(1 << 8); // Xbox/Home button
    pub const LS: Self = Self(1 << 9); // Left stick press
    pub const RS: Self = Self(1 << 10); // Right stick press
    pub const DPAD_UP: Self = Self(1 << 11);
    pub const DPAD_DOWN: Self = Self(1 << 12);
    pub const DPAD_LEFT: Self = Self(1 << 13);
    pub const DPAD_RIGHT: Self = Self(1 << 14);

    /// No buttons pressed.
    pub const NONE: Self = Self(0);

    /// Check if the given button(s) are pressed.
    #[inline]
    #[must_use]
    pub const fn contains(self, button: Buttons) -> bool {
        (self.0 & button.0) == button.0
    }

    /// Check if the given button is pressed (alias for contains).
    #[inline]
    #[must_use]
    pub const fn is_pressed(self, button: Buttons) -> bool {
        self.contains(button)
    }

    /// Set or clear button(s).
    #[inline]
    pub fn set(&mut self, button: Buttons, pressed: bool) {
        if pressed {
            self.0 |= button.0;
        } else {
            self.0 &= !button.0;
        }
    }

    /// Get the raw u16 value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Check if no buttons are pressed.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for Buttons {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Buttons {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Buttons {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Buttons {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for Buttons {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Analog stick with X/Y axes.
///
/// Range: [-32768, 32767] for full precision.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AnalogStick {
    pub x: i16,
    pub y: i16,
}

impl AnalogStick {
    #[must_use]
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const NEUTRAL: Self = Self { x: 0, y: 0 };
}

/// Complete gamepad state snapshot.
///
/// Contains all inputs for a standard gamepad:
/// - 16 buttons (bitfield)
/// - 2 analog sticks (left/right, each with X/Y)
/// - 2 triggers (left/right, 0-255)
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GamepadState {
    pub buttons: Buttons,
    pub left_stick: AnalogStick,
    pub right_stick: AnalogStick,
    pub left_trigger: u8,
    pub right_trigger: u8,
}

impl GamepadState {
    /// Create a zeroed/neutral gamepad state (no buttons pressed, sticks centered).
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            buttons: Buttons::NONE,
            left_stick: AnalogStick::NEUTRAL,
            right_stick: AnalogStick::NEUTRAL,
            left_trigger: 0,
            right_trigger: 0,
        }
    }

    /// Apply a single field update to this state.
    #[inline]
    pub fn apply_update(&mut self, update: GamepadFieldUpdate) {
        match update {
            GamepadFieldUpdate::Buttons(b) => self.buttons = b,
            GamepadFieldUpdate::LeftStickX(x) => self.left_stick.x = x,
            GamepadFieldUpdate::LeftStickY(y) => self.left_stick.y = y,
            GamepadFieldUpdate::RightStickX(x) => self.right_stick.x = x,
            GamepadFieldUpdate::RightStickY(y) => self.right_stick.y = y,
            GamepadFieldUpdate::LeftTrigger(t) => self.left_trigger = t,
            GamepadFieldUpdate::RightTrigger(t) => self.right_trigger = t,
        }
    }
}

/// Represents a single field update for incremental protocol messages.
///
/// Used with the "U" prefix protocol messages to update individual fields
/// without sending the full gamepad state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[must_use]
pub enum GamepadFieldUpdate {
    /// Update buttons (B field)
    Buttons(Buttons),
    /// Update left stick X axis (LX field)
    LeftStickX(i16),
    /// Update left stick Y axis (LY field)
    LeftStickY(i16),
    /// Update right stick X axis (RX field)
    RightStickX(i16),
    /// Update right stick Y axis (RY field)
    RightStickY(i16),
    /// Update left trigger (LT field)
    LeftTrigger(u8),
    /// Update right trigger (RT field)
    RightTrigger(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buttons_bitwise_or() {
        let buttons = Buttons::A | Buttons::B;
        assert!(buttons.contains(Buttons::A));
        assert!(buttons.contains(Buttons::B));
        assert!(!buttons.contains(Buttons::X));
    }

    #[test]
    fn test_buttons_set_clear() {
        let mut buttons = Buttons::NONE;
        buttons.set(Buttons::A, true);
        assert!(buttons.is_pressed(Buttons::A));
        buttons.set(Buttons::A, false);
        assert!(!buttons.is_pressed(Buttons::A));
    }

    #[test]
    fn test_gamepad_state_apply_update() {
        let mut state = GamepadState::neutral();
        state.apply_update(GamepadFieldUpdate::Buttons(Buttons::A | Buttons::B));
        assert!(state.buttons.is_pressed(Buttons::A));
        assert!(state.buttons.is_pressed(Buttons::B));

        state.apply_update(GamepadFieldUpdate::LeftStickX(-1000));
        assert_eq!(state.left_stick.x, -1000);

        state.apply_update(GamepadFieldUpdate::LeftTrigger(128));
        assert_eq!(state.left_trigger, 128);
    }

    #[test]
    fn test_analog_stick_neutral() {
        let stick = AnalogStick::NEUTRAL;
        assert_eq!(stick.x, 0);
        assert_eq!(stick.y, 0);
    }
}
