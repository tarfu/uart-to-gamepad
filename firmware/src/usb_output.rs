//! USB HID gamepad output implementation.

use gamepad_core::{GamepadState, OutputError, OutputSink};
use defmt::Format;
use embassy_usb::class::hid::{HidWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::Builder;

/// USB HID Gamepad report structure.
///
/// This matches the HID report descriptor defined below.
/// Total size: 8 bytes (buttons: 2, sticks: 4x1, triggers: 2x1)
///
/// Note: Stick values are scaled from i16 to i8 for HID compatibility.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Format)]
#[repr(C)]
pub struct GamepadReport {
    /// Button bitfield (16 buttons)
    pub buttons: u16,
    /// Left stick X (-128 to 127)
    pub left_stick_x: i8,
    /// Left stick Y (-128 to 127)
    pub left_stick_y: i8,
    /// Right stick X (-128 to 127)
    pub right_stick_x: i8,
    /// Right stick Y (-128 to 127)
    pub right_stick_y: i8,
    /// Left trigger (0-255)
    pub left_trigger: u8,
    /// Right trigger (0-255)
    pub right_trigger: u8,
}

impl GamepadReport {
    /// Size of the report in bytes.
    pub const SIZE: usize = 8;

    /// Convert the report to bytes.
    #[must_use]
    pub fn as_bytes(&self) -> [u8; Self::SIZE] {
        let buttons_bytes = self.buttons.to_le_bytes();
        [
            buttons_bytes[0],
            buttons_bytes[1],
            self.left_stick_x as u8,
            self.left_stick_y as u8,
            self.right_stick_x as u8,
            self.right_stick_y as u8,
            self.left_trigger,
            self.right_trigger,
        ]
    }

    /// Neutral/zero report.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            buttons: 0,
            left_stick_x: 0,
            left_stick_y: 0,
            right_stick_x: 0,
            right_stick_y: 0,
            left_trigger: 0,
            right_trigger: 0,
        }
    }
}

impl From<&GamepadState> for GamepadReport {
    fn from(state: &GamepadState) -> Self {
        Self {
            buttons: state.buttons.raw(),
            // Scale i16 to i8 by taking the high byte
            left_stick_x: (state.left_stick.x >> 8) as i8,
            left_stick_y: (state.left_stick.y >> 8) as i8,
            right_stick_x: (state.right_stick.x >> 8) as i8,
            right_stick_y: (state.right_stick.y >> 8) as i8,
            left_trigger: state.left_trigger,
            right_trigger: state.right_trigger,
        }
    }
}

/// Standard HID Gamepad Report Descriptor.
///
/// This descriptor defines a gamepad with:
/// - 16 buttons
/// - 2 analog sticks (X/Y each, signed 8-bit)
/// - 2 triggers (unsigned 8-bit)
#[cfg(feature = "standard-hid")]
pub const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    //
    // --- Buttons (16 buttons) ---
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (Button 1)
    0x29, 0x10, //   Usage Maximum (Button 16)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x95, 0x10, //   Report Count (16)
    0x75, 0x01, //   Report Size (1)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Left Stick ---
    0x05, 0x01, //   Usage Page (Generic Desktop)
    0x09, 0x30, //   Usage (X)
    0x09, 0x31, //   Usage (Y)
    0x15, 0x81, //   Logical Minimum (-127)
    0x25, 0x7F, //   Logical Maximum (127)
    0x95, 0x02, //   Report Count (2)
    0x75, 0x08, //   Report Size (8)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Right Stick ---
    0x09, 0x32, //   Usage (Z)
    0x09, 0x35, //   Usage (Rz)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Triggers ---
    0x09, 0x33, //   Usage (Rx) - Left trigger
    0x09, 0x34, //   Usage (Ry) - Right trigger
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    0xC0, // End Collection
];

/// XInput-compatible HID Report Descriptor.
///
/// This descriptor attempts to be recognized as an Xbox controller
/// for better compatibility with Windows games.
#[cfg(feature = "xinput-compat")]
pub const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    0xA1, 0x00, //   Collection (Physical)
    //
    // --- Buttons (16 buttons) ---
    0x05, 0x09, //     Usage Page (Button)
    0x19, 0x01, //     Usage Minimum (Button 1)
    0x29, 0x10, //     Usage Maximum (Button 16)
    0x15, 0x00, //     Logical Minimum (0)
    0x25, 0x01, //     Logical Maximum (1)
    0x95, 0x10, //     Report Count (16)
    0x75, 0x01, //     Report Size (1)
    0x81, 0x02, //     Input (Data, Variable, Absolute)
    //
    // --- Left Stick ---
    0x05, 0x01, //     Usage Page (Generic Desktop)
    0x09, 0x30, //     Usage (X)
    0x09, 0x31, //     Usage (Y)
    0x16, 0x01, 0x80, // Logical Minimum (-32767)
    0x26, 0xFF, 0x7F, // Logical Maximum (32767)
    0x95, 0x02, //     Report Count (2)
    0x75, 0x10, //     Report Size (16) - Full 16-bit for XInput
    0x81, 0x02, //     Input (Data, Variable, Absolute)
    //
    // --- Right Stick ---
    0x09, 0x32, //     Usage (Z)
    0x09, 0x35, //     Usage (Rz)
    0x95, 0x02, //     Report Count (2)
    0x81, 0x02, //     Input (Data, Variable, Absolute)
    //
    // --- Triggers ---
    0x09, 0x33, //     Usage (Rx)
    0x09, 0x34, //     Usage (Ry)
    0x15, 0x00, //     Logical Minimum (0)
    0x26, 0xFF, 0x00, // Logical Maximum (255)
    0x95, 0x02, //     Report Count (2)
    0x75, 0x08, //     Report Size (8)
    0x81, 0x02, //     Input (Data, Variable, Absolute)
    //
    0xC0, //   End Collection
    0xC0, // End Collection
];

/// Default report descriptor (standard HID).
#[cfg(all(not(feature = "standard-hid"), not(feature = "xinput-compat")))]
pub const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x05, // Usage (Gamepad)
    0xA1, 0x01, // Collection (Application)
    //
    // --- Buttons (16 buttons) ---
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (Button 1)
    0x29, 0x10, //   Usage Maximum (Button 16)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x95, 0x10, //   Report Count (16)
    0x75, 0x01, //   Report Size (1)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Left Stick ---
    0x05, 0x01, //   Usage Page (Generic Desktop)
    0x09, 0x30, //   Usage (X)
    0x09, 0x31, //   Usage (Y)
    0x15, 0x81, //   Logical Minimum (-127)
    0x25, 0x7F, //   Logical Maximum (127)
    0x95, 0x02, //   Report Count (2)
    0x75, 0x08, //   Report Size (8)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Right Stick ---
    0x09, 0x32, //   Usage (Z)
    0x09, 0x35, //   Usage (Rz)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    // --- Triggers ---
    0x09, 0x33, //   Usage (Rx)
    0x09, 0x34, //   Usage (Ry)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x02, //   Input (Data, Variable, Absolute)
    //
    0xC0, // End Collection
];

/// USB HID gamepad output.
///
/// Wraps an embassy-usb HID writer to send gamepad reports.
pub struct UsbHidOutput<'d> {
    writer: HidWriter<'d, embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>, 8>,
    ready: bool,
}

impl<'d> UsbHidOutput<'d> {
    /// Create a new USB HID output from the given HID writer.
    pub fn new(
        writer: HidWriter<'d, embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>, 8>,
    ) -> Self {
        Self {
            writer,
            ready: false,
        }
    }

    /// Wait until the device is ready (USB enumerated).
    pub async fn wait_ready(&mut self) {
        self.writer.ready().await;
        self.ready = true;
    }
}

impl<'d> OutputSink for UsbHidOutput<'d> {
    async fn send(&mut self, state: &GamepadState) -> Result<(), OutputError> {
        let report = GamepadReport::from(state);
        self.writer
            .write(&report.as_bytes())
            .await
            .map_err(|_| OutputError::Io)
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

/// HID request handler (handles SET_REPORT, etc.).
///
/// Currently a no-op handler since we don't handle output reports.
pub struct GamepadRequestHandler;

impl RequestHandler for GamepadRequestHandler {
    fn get_report(&mut self, _id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        None
    }

    fn set_report(&mut self, _id: ReportId, _data: &[u8]) -> OutResponse {
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, _id: Option<ReportId>, _duration_ms: u32) {}

    fn get_idle_ms(&mut self, _id: Option<ReportId>) -> Option<u32> {
        None
    }
}

/// Configure the USB HID class in the USB builder.
///
/// Returns the HID writer for use by the application.
pub fn configure_usb_hid<'d>(
    builder: &mut Builder<'d, embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>>,
    state: &'d mut State<'d>,
) -> HidWriter<'d, embassy_rp::usb::Driver<'d, embassy_rp::peripherals::USB>, 8> {
    let config = embassy_usb::class::hid::Config {
        report_descriptor: REPORT_DESCRIPTOR,
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 8,
        hid_subclass: embassy_usb::class::hid::HidSubclass::No,
        hid_boot_protocol: embassy_usb::class::hid::HidBootProtocol::None,
    };

    embassy_usb::class::hid::HidWriter::new(builder, state, config)
}
