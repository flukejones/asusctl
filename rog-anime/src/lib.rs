/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod anime_data;
pub use anime_data::*;

/// Useful for specialised effects that required a grid of data
mod anime_grid;
pub use anime_grid::*;

/// Transform a PNG image for displaying on AniMe matrix display
mod anime_image;
pub use anime_image::*;

/// Base errors that are possible
pub mod error;