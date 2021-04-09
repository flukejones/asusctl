/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod data;

pub use data::*;

/// Useful for specialised effects that required a grid of data
mod grid;
pub use grid::*;

/// Transform a PNG image for displaying on AniMe matrix display
mod image;
pub use image::*;

mod diagonal;
pub use diagonal::*;

mod gif;
pub use crate::gif::*;

mod sequencer;
pub use sequencer::*;

/// Base errors that are possible
pub mod error;
