mod builtin_modes;
use advanced::LedCode;
pub use builtin_modes::*;

/// A container of images/grids/gifs/pauses which can be iterated over to
/// generate cool effects
pub mod effects;

/// All handling for `RgbAddress`ing.
pub mod advanced;
/// Convert the `RgbAddress` to `&str` labels
pub mod advanced_to_str;
/// Helper for detecting what is available
pub mod aura_detection;
pub mod error;
/// Helpers for consructing keyboard layouts for UI use and effects
pub mod layouts;
pub mod usb;

pub mod power;

mod deprecated;

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

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
pub enum AdvancedAuraType {
    /// A `None` will apply the effect to the whole keyboard via basic-static
    /// mode
    #[default]
    None,
    Zoned(Vec<LedCode>),
    PerKey,
}
