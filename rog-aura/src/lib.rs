/// The main data conversion for transfering in shortform over dbus or other,
/// or writing directly to the USB device
mod data;
pub use data::*;

/// A container of images/grids/gifs/pauses which can be iterated over to generate
/// cool effects
mod sequencer;
pub use sequencer::*;

mod aura_modes;
pub use aura_modes::*;

mod aura_perkey;
pub use aura_perkey::*;

pub mod usb;

pub mod error;

pub const LED_MSG_LEN: usize = 17;