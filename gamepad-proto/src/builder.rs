//! Builder pattern API for constructing protocol messages.
//!
//! This module provides a fluent builder API for creating gamepad protocol messages
//! without needing to construct the full type structures first.
//!
//! # Example
//!
//! ```
//! use gamepad_proto::{MessageBuilder, Buttons};
//!
//! // Build a full state message
//! let mut buf = [0u8; 64];
//! let len = MessageBuilder::full_state()
//!     .buttons(Buttons::A | Buttons::B)
//!     .left_stick(100, -200)
//!     .right_stick(0, 0)
//!     .left_trigger(128)
//!     .right_trigger(64)
//!     .serialize(&mut buf)
//!     .unwrap();
//!
//! // Build an update message
//! let len = MessageBuilder::update()
//!     .buttons(Buttons::X)
//!     .serialize(&mut buf)
//!     .unwrap();
//! ```

use crate::serialize::SerializeError;
use crate::types::{AnalogStick, Buttons, GamepadFieldUpdate, GamepadState};

/// Entry point for building protocol messages.
///
/// Use [`MessageBuilder::full_state()`] to create a full state message,
/// or [`MessageBuilder::update()`] to create an incremental update message.
pub struct MessageBuilder;

impl MessageBuilder {
    /// Start building a full state message.
    ///
    /// The builder starts with neutral/default values for all fields.
    ///
    /// # Example
    ///
    /// ```
    /// use gamepad_proto::MessageBuilder;
    ///
    /// let mut buf = [0u8; 64];
    /// let len = MessageBuilder::full_state()
    ///     .left_stick(1000, -500)
    ///     .serialize(&mut buf)
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn full_state() -> FullStateBuilder {
        FullStateBuilder {
            state: GamepadState::neutral(),
        }
    }

    /// Start building an incremental update message.
    ///
    /// You must call exactly one of the field setter methods before serializing.
    ///
    /// # Example
    ///
    /// ```
    /// use gamepad_proto::{MessageBuilder, Buttons};
    ///
    /// let mut buf = [0u8; 32];
    /// let len = MessageBuilder::update()
    ///     .buttons(Buttons::START)
    ///     .serialize(&mut buf)
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn update() -> UpdateBuilder {
        UpdateBuilder { update: None }
    }
}

/// Builder for full state messages.
///
/// Created via [`MessageBuilder::full_state()`].
#[derive(Debug, Clone)]
pub struct FullStateBuilder {
    state: GamepadState,
}

impl FullStateBuilder {
    /// Set the button state.
    #[must_use]
    pub fn buttons(mut self, buttons: Buttons) -> Self {
        self.state.buttons = buttons;
        self
    }

    /// Set the left stick position.
    #[must_use]
    pub fn left_stick(mut self, x: i16, y: i16) -> Self {
        self.state.left_stick = AnalogStick::new(x, y);
        self
    }

    /// Set the right stick position.
    #[must_use]
    pub fn right_stick(mut self, x: i16, y: i16) -> Self {
        self.state.right_stick = AnalogStick::new(x, y);
        self
    }

    /// Set the left trigger value.
    #[must_use]
    pub fn left_trigger(mut self, value: u8) -> Self {
        self.state.left_trigger = value;
        self
    }

    /// Set the right trigger value.
    #[must_use]
    pub fn right_trigger(mut self, value: u8) -> Self {
        self.state.right_trigger = value;
        self
    }

    /// Set both triggers at once.
    #[must_use]
    pub fn triggers(mut self, left: u8, right: u8) -> Self {
        self.state.left_trigger = left;
        self.state.right_trigger = right;
        self
    }

    /// Get the built state without serializing.
    #[must_use]
    pub fn build(self) -> GamepadState {
        self.state
    }

    /// Serialize the message to the provided buffer.
    ///
    /// Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::BufferTooSmall`] if the buffer is too small.
    pub fn serialize(self, buf: &mut [u8]) -> Result<usize, SerializeError> {
        use crate::serialize::Serialize;
        self.state.serialize(buf)
    }

    /// Serialize to a `heapless::Vec`.
    #[cfg(feature = "heapless")]
    pub fn serialize_to_vec<const N: usize>(self) -> Result<heapless::Vec<u8, N>, SerializeError> {
        use crate::serialize::Serialize;
        self.state.serialize_to_vec()
    }

    /// Serialize to a `core::fmt::Write` implementation.
    pub fn serialize_fmt<W: core::fmt::Write>(self, writer: &mut W) -> Result<(), SerializeError> {
        use crate::serialize::Serialize;
        self.state.serialize_fmt(writer)
    }

    /// Serialize to an `embedded_io::Write` implementation.
    #[cfg(feature = "embedded-io")]
    pub fn serialize_io<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), SerializeError> {
        use crate::serialize::Serialize;
        self.state.serialize_io(writer)
    }
}

impl Default for FullStateBuilder {
    fn default() -> Self {
        MessageBuilder::full_state()
    }
}

/// Builder for incremental update messages.
///
/// Created via [`MessageBuilder::update()`].
///
/// You must call exactly one field setter method. Calling multiple setters
/// will overwrite the previous value - only the last one will be serialized.
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    update: Option<GamepadFieldUpdate>,
}

impl UpdateBuilder {
    /// Set the update to a buttons change.
    #[must_use]
    pub fn buttons(mut self, buttons: Buttons) -> Self {
        self.update = Some(GamepadFieldUpdate::Buttons(buttons));
        self
    }

    /// Set the update to a left stick X change.
    #[must_use]
    pub fn left_stick_x(mut self, value: i16) -> Self {
        self.update = Some(GamepadFieldUpdate::LeftStickX(value));
        self
    }

    /// Set the update to a left stick Y change.
    #[must_use]
    pub fn left_stick_y(mut self, value: i16) -> Self {
        self.update = Some(GamepadFieldUpdate::LeftStickY(value));
        self
    }

    /// Set the update to a right stick X change.
    #[must_use]
    pub fn right_stick_x(mut self, value: i16) -> Self {
        self.update = Some(GamepadFieldUpdate::RightStickX(value));
        self
    }

    /// Set the update to a right stick Y change.
    #[must_use]
    pub fn right_stick_y(mut self, value: i16) -> Self {
        self.update = Some(GamepadFieldUpdate::RightStickY(value));
        self
    }

    /// Set the update to a left trigger change.
    #[must_use]
    pub fn left_trigger(mut self, value: u8) -> Self {
        self.update = Some(GamepadFieldUpdate::LeftTrigger(value));
        self
    }

    /// Set the update to a right trigger change.
    #[must_use]
    pub fn right_trigger(mut self, value: u8) -> Self {
        self.update = Some(GamepadFieldUpdate::RightTrigger(value));
        self
    }

    /// Get the built update without serializing.
    ///
    /// Returns `None` if no field was set.
    #[must_use]
    pub fn build(self) -> Option<GamepadFieldUpdate> {
        self.update
    }

    /// Serialize the message to the provided buffer.
    ///
    /// Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError::BufferTooSmall`] if the buffer is too small,
    /// or if no field was set (nothing to serialize).
    pub fn serialize(self, buf: &mut [u8]) -> Result<usize, SerializeError> {
        use crate::serialize::Serialize;
        self.update
            .ok_or(SerializeError::BufferTooSmall)?
            .serialize(buf)
    }

    /// Serialize to a `heapless::Vec`.
    #[cfg(feature = "heapless")]
    pub fn serialize_to_vec<const N: usize>(self) -> Result<heapless::Vec<u8, N>, SerializeError> {
        use crate::serialize::Serialize;
        self.update
            .ok_or(SerializeError::BufferTooSmall)?
            .serialize_to_vec()
    }

    /// Serialize to a `core::fmt::Write` implementation.
    pub fn serialize_fmt<W: core::fmt::Write>(self, writer: &mut W) -> Result<(), SerializeError> {
        use crate::serialize::Serialize;
        self.update
            .ok_or(SerializeError::BufferTooSmall)?
            .serialize_fmt(writer)
    }

    /// Serialize to an `embedded_io::Write` implementation.
    #[cfg(feature = "embedded-io")]
    pub fn serialize_io<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), SerializeError> {
        use crate::serialize::Serialize;
        self.update
            .ok_or(SerializeError::BufferTooSmall)?
            .serialize_io(writer)
    }
}

impl Default for UpdateBuilder {
    fn default() -> Self {
        MessageBuilder::update()
    }
}

/// Convenience function to quickly serialize a full state to a buffer.
///
/// This is equivalent to `MessageBuilder::full_state()` with all the given values.
///
/// # Example
///
/// ```
/// use gamepad_proto::{serialize_full_state, Buttons};
///
/// let mut buf = [0u8; 64];
/// let len = serialize_full_state(
///     &mut buf,
///     Buttons::A,
///     1000, -500,  // left stick
///     0, 0,        // right stick
///     128, 64,     // triggers
/// ).unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn serialize_full_state(
    buf: &mut [u8],
    buttons: Buttons,
    lx: i16,
    ly: i16,
    rx: i16,
    ry: i16,
    lt: u8,
    rt: u8,
) -> Result<usize, SerializeError> {
    MessageBuilder::full_state()
        .buttons(buttons)
        .left_stick(lx, ly)
        .right_stick(rx, ry)
        .triggers(lt, rt)
        .serialize(buf)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::parser::{parse, parse_message, ParsedMessage};

    #[test]
    fn test_full_state_builder_default() {
        let state = MessageBuilder::full_state().build();
        assert_eq!(state, GamepadState::neutral());
    }

    #[test]
    fn test_full_state_builder_buttons() {
        let state = MessageBuilder::full_state()
            .buttons(Buttons::A | Buttons::B)
            .build();
        assert!(state.buttons.is_pressed(Buttons::A));
        assert!(state.buttons.is_pressed(Buttons::B));
    }

    #[test]
    fn test_full_state_builder_sticks() {
        let state = MessageBuilder::full_state()
            .left_stick(100, -200)
            .right_stick(-300, 400)
            .build();
        assert_eq!(state.left_stick.x, 100);
        assert_eq!(state.left_stick.y, -200);
        assert_eq!(state.right_stick.x, -300);
        assert_eq!(state.right_stick.y, 400);
    }

    #[test]
    fn test_full_state_builder_triggers() {
        let state = MessageBuilder::full_state()
            .left_trigger(128)
            .right_trigger(255)
            .build();
        assert_eq!(state.left_trigger, 128);
        assert_eq!(state.right_trigger, 255);
    }

    #[test]
    fn test_full_state_builder_triggers_combined() {
        let state = MessageBuilder::full_state().triggers(64, 192).build();
        assert_eq!(state.left_trigger, 64);
        assert_eq!(state.right_trigger, 192);
    }

    #[test]
    fn test_full_state_builder_serialize() {
        let mut buf = [0u8; 64];
        let len = MessageBuilder::full_state()
            .buttons(Buttons::X | Buttons::Y)
            .left_stick(1000, -1000)
            .serialize(&mut buf)
            .unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert!(parsed.buttons.is_pressed(Buttons::X));
        assert!(parsed.buttons.is_pressed(Buttons::Y));
        assert_eq!(parsed.left_stick.x, 1000);
        assert_eq!(parsed.left_stick.y, -1000);
    }

    #[test]
    fn test_full_state_builder_serialize_fmt() {
        let mut s = std::string::String::new();
        MessageBuilder::full_state()
            .left_trigger(100)
            .serialize_fmt(&mut s)
            .unwrap();

        assert!(s.starts_with("G"));
        assert!(s.ends_with('\n'));
    }

    #[test]
    fn test_update_builder_buttons() {
        let update = MessageBuilder::update()
            .buttons(Buttons::START)
            .build()
            .unwrap();
        assert_eq!(update, GamepadFieldUpdate::Buttons(Buttons::START));
    }

    #[test]
    fn test_update_builder_left_stick_x() {
        let update = MessageBuilder::update().left_stick_x(-500).build().unwrap();
        assert_eq!(update, GamepadFieldUpdate::LeftStickX(-500));
    }

    #[test]
    fn test_update_builder_left_stick_y() {
        let update = MessageBuilder::update().left_stick_y(1000).build().unwrap();
        assert_eq!(update, GamepadFieldUpdate::LeftStickY(1000));
    }

    #[test]
    fn test_update_builder_right_stick_x() {
        let update = MessageBuilder::update()
            .right_stick_x(2000)
            .build()
            .unwrap();
        assert_eq!(update, GamepadFieldUpdate::RightStickX(2000));
    }

    #[test]
    fn test_update_builder_right_stick_y() {
        let update = MessageBuilder::update()
            .right_stick_y(-3000)
            .build()
            .unwrap();
        assert_eq!(update, GamepadFieldUpdate::RightStickY(-3000));
    }

    #[test]
    fn test_update_builder_left_trigger() {
        let update = MessageBuilder::update().left_trigger(128).build().unwrap();
        assert_eq!(update, GamepadFieldUpdate::LeftTrigger(128));
    }

    #[test]
    fn test_update_builder_right_trigger() {
        let update = MessageBuilder::update().right_trigger(255).build().unwrap();
        assert_eq!(update, GamepadFieldUpdate::RightTrigger(255));
    }

    #[test]
    fn test_update_builder_no_field_set() {
        let update = MessageBuilder::update().build();
        assert!(update.is_none());
    }

    #[test]
    fn test_update_builder_serialize() {
        let mut buf = [0u8; 32];
        let len = MessageBuilder::update()
            .left_trigger(64)
            .serialize(&mut buf)
            .unwrap();

        let parsed = parse_message(&buf[..len]).unwrap();
        assert_eq!(
            parsed,
            ParsedMessage::Update(GamepadFieldUpdate::LeftTrigger(64))
        );
    }

    #[test]
    fn test_update_builder_serialize_no_field() {
        let mut buf = [0u8; 32];
        let result = MessageBuilder::update().serialize(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_builder_serialize_fmt() {
        let mut s = std::string::String::new();
        MessageBuilder::update()
            .right_trigger(200)
            .serialize_fmt(&mut s)
            .unwrap();

        assert!(s.starts_with("URT:200*"));
        assert!(s.ends_with('\n'));
    }

    #[test]
    fn test_serialize_full_state_function() {
        let mut buf = [0u8; 64];
        let len = serialize_full_state(
            &mut buf,
            Buttons::A | Buttons::B,
            100,
            -200,
            300,
            -400,
            50,
            100,
        )
        .unwrap();

        let parsed = parse(&buf[..len]).unwrap();
        assert!(parsed.buttons.is_pressed(Buttons::A));
        assert!(parsed.buttons.is_pressed(Buttons::B));
        assert_eq!(parsed.left_stick.x, 100);
        assert_eq!(parsed.left_stick.y, -200);
        assert_eq!(parsed.right_stick.x, 300);
        assert_eq!(parsed.right_stick.y, -400);
        assert_eq!(parsed.left_trigger, 50);
        assert_eq!(parsed.right_trigger, 100);
    }

    #[test]
    fn test_update_builder_overwrites_previous() {
        // Calling multiple setters should only keep the last one
        let update = MessageBuilder::update()
            .left_stick_x(100)
            .right_stick_y(200)
            .build()
            .unwrap();

        // Only the last setter should be preserved
        assert_eq!(update, GamepadFieldUpdate::RightStickY(200));
    }
}
