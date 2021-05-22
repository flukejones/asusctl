/// A container of images/grids/gifs/pauses which can be iterated over to generate
/// cool effects
mod sequencer;
pub use sequencer::*;

mod builtin_modes;
pub use builtin_modes::*;

mod per_key_rgb;
pub use per_key_rgb::*;

pub mod usb;

pub mod error;

pub const LED_MSG_LEN: usize = 17;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
