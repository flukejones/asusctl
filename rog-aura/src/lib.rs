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

pub const RED: Colour = Colour(0xff, 0x00, 0x00);
pub const GREEN: Colour = Colour(0x00, 0xff, 0x00);
pub const BLUE: Colour = Colour(0x00, 0x00, 0xff);
pub const VIOLET: Colour = Colour(0x9B, 0x26, 0xB6);
pub const TEAL: Colour = Colour(0x00, 0x7C, 0x80);
pub const YELLOW: Colour = Colour(0xff, 0xef, 0x00);
pub const ORANGE: Colour = Colour(0xff, 0xa4, 0x00);
pub const GRADIENT: [Colour; 7] = [RED, VIOLET, BLUE, TEAL, GREEN, YELLOW, ORANGE];
