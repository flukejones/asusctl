/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod data;
pub use data::*;

/// Useful for specialised effects that require a grid of data
mod grid;
pub use grid::*;

/// Transform a PNG image for displaying on AniMe matrix display
mod image;
pub use image::*;

/// A grid of data that is intended to be read out and displayed on the ANiMe as
/// a diagonal
mod diagonal;
pub use diagonal::*;

/// A gif. Can be created from the ASUS gifs which are diagonal layout, or from
/// any standard gif
mod gif;
pub use crate::gif::*;

/// A container of images/grids/gifs/pauses which can be iterated over to generate
/// cool effects
mod sequencer;
pub use sequencer::*;

/// Base errors that are possible
pub mod error;

/// Provides const methods to create the USB HID control packets
pub mod usb;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");