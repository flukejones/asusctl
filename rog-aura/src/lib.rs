// TODO: Generic builtin modes
// TODO: Traits for finding device + writing generic modes
// TODO: Traits for writing aura_sync
// TODO: separate keyboard and laptop parts?

/// A container of images/grids/gifs/pauses which can be iterated over to
/// generate cool effects
pub mod effects;

mod builtin_modes;
pub use builtin_modes::*;

/// Helper for detecting what is available
pub mod aura_detection;
pub mod error;
pub mod usb;

pub mod keyboard;

pub const LED_MSG_LEN: usize = 17;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const RED: Colour = Colour {
    r: 0xff,
    g: 0x00,
    b: 0x00,
};
pub const GREEN: Colour = Colour {
    r: 0x00,
    g: 0xff,
    b: 0x00,
};
pub const BLUE: Colour = Colour {
    r: 0x00,
    g: 0x00,
    b: 0xff,
};
pub const VIOLET: Colour = Colour {
    r: 0x9b,
    g: 0x26,
    b: 0xb6,
};
pub const TEAL: Colour = Colour {
    r: 0x00,
    g: 0x7c,
    b: 0x80,
};
pub const YELLOW: Colour = Colour {
    r: 0xff,
    g: 0xef,
    b: 0x00,
};
pub const ORANGE: Colour = Colour {
    r: 0xff,
    g: 0xa4,
    b: 0x00,
};
pub const GRADIENT: [Colour; 7] = [RED, VIOLET, BLUE, TEAL, GREEN, YELLOW, ORANGE];
