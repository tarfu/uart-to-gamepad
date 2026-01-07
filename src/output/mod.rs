mod traits;
pub mod usb_hid;

pub use traits::{OutputError, OutputSink};
pub use usb_hid::{GamepadReport, UsbHidOutput};
