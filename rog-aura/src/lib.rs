// TODO: Generic builtin modes
// TODO: Traits for finding device + writing generic modes
// TODO: Traits for writing aura_sync
// TODO: separate keyboard and laptop parts?

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::{OwnedValue, Type, Value};

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

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuraDeviceType {
    /// Most new laptops
    #[default]
    LaptopPost2021 = 0,
    LaptopPre2021 = 1,
    LaptopTuf = 2,
    ScsiExtDisk = 3,
    Unknown = 255,
}

impl AuraDeviceType {
    pub fn is_old_laptop(&self) -> bool {
        *self == Self::LaptopPre2021
    }

    pub fn is_tuf_laptop(&self) -> bool {
        *self == Self::LaptopTuf
    }

    pub fn is_new_laptop(&self) -> bool {
        *self == Self::LaptopPost2021
    }

    pub fn is_scsi(&self) -> bool {
        *self == Self::ScsiExtDisk
    }
}

impl From<&str> for AuraDeviceType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim_start_matches("0x") {
            "tuf" => AuraDeviceType::LaptopTuf,
            "1932" => AuraDeviceType::ScsiExtDisk,
            "1866" | "18c6" | "1869" | "1854" => Self::LaptopPre2021,
            _ => Self::LaptopPost2021,
        }
    }
}

/// The powerr zones this laptop supports
#[typeshare]
#[cfg_attr(
    feature = "dbus",
    derive(Type, Value, OwnedValue),
    zvariant(signature = "u")
)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Default, Copy, Clone)]
pub enum PowerZones {
    /// The logo on some laptop lids
    Logo = 0,
    /// The full keyboard (not zones)
    #[default]
    Keyboard = 1,
    /// The lightbar, typically on the front of the laptop
    Lightbar = 2,
    /// The leds that may be placed around the edge of the laptop lid
    Lid = 3,
    /// The led strip on the rear of some laptops
    RearGlow = 4,
    /// On pre-2021 laptops there is either 1 or 2 zones used
    KeyboardAndLightbar = 5,
    None = 255,
}
