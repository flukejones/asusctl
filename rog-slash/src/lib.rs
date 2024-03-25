/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod data;
pub use data::*;

/// Base errors that are possible
pub mod error;

/// Provides const methods to create the USB HID control packets
pub mod usb;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");