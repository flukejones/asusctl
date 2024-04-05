use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::keyboard::{LaptopAuraPower, LaptopOldAuraPower, LaptopTufAuraPower};

// Only these two packets must be 17 bytes
pub const LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5a, 0xba, 0xc5, 0xc4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[typeshare]
#[cfg_attr(
    feature = "dbus",
    derive(Type, Value, OwnedValue),
    zvariant(signature = "s")
)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AuraDevice {
    Tuf = 0,
    X1854 = 1,
    X1869 = 2,
    /// Pre-2020 laptops
    X1866 = 3,
    /// Z13 lightbar
    X18c6 = 4,
    /// Most modern laptops
    #[default]
    X19b6 = 5,
    X1a30 = 6,
    /// The ROG Ally
    X1abe = 7,
    Unknown = 99,
}

impl AuraDevice {
    pub fn is_tuf_style(&self) -> bool {
        matches!(self, AuraDevice::Tuf)
    }

    pub fn is_old_style(&self) -> bool {
        matches!(
            self,
            AuraDevice::X1854 | AuraDevice::X1869 | AuraDevice::X1866 | AuraDevice::X1abe
        )
    }

    pub fn is_new_style(&self) -> bool {
        !self.is_old_style() && !self.is_tuf_style()
    }
}

impl From<AuraDevice> for &str {
    fn from(a: AuraDevice) -> Self {
        match a {
            AuraDevice::Tuf => "tuf",
            AuraDevice::X1854 => "1854",
            AuraDevice::X1869 => "1869",
            AuraDevice::X1866 => "1866",
            AuraDevice::X18c6 => "18c6",
            AuraDevice::X19b6 => "19b6",
            AuraDevice::X1a30 => "1a30",
            AuraDevice::X1abe => "1abe",
            AuraDevice::Unknown => "unknown",
        }
    }
}

impl From<&str> for AuraDevice {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "tuf" => AuraDevice::Tuf,
            "1866" | "0x1866" => AuraDevice::X1866,
            "18c6" | "0x18c6" => AuraDevice::X18c6,
            "1869" | "0x1869" => AuraDevice::X1869,
            "1854" | "0x1854" => AuraDevice::X1854,
            "19b6" | "0x19b6" => AuraDevice::X19b6,
            "1a30" | "0x1a30" => AuraDevice::X1a30,
            "1abe" | "0x1abe" => AuraDevice::X1abe,
            _ => AuraDevice::Unknown,
        }
    }
}

impl Debug for AuraDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tuf => write!(f, "Tuf"),
            Self::X1854 => write!(f, "0x1854"),
            Self::X1869 => write!(f, "0x1869"),
            Self::X1866 => write!(f, "0x1866"),
            Self::X18c6 => write!(f, "0x18c6"),
            Self::X19b6 => write!(f, "0x19B6"),
            Self::X1a30 => write!(f, "0x1A30"),
            Self::X1abe => write!(f, "0x1ABE"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// This struct is intended as a helper to pass args to generic dbus interface
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AuraPowerDev {
    /// TUF laptops use a similar style of control to the older ROG devices but
    /// through WMI
    pub tuf: Vec<LaptopTufAuraPower>,
    /// Pre-0x19b6 devices use a different smaller scheme to the newer ROG
    /// devices
    pub old_rog: Vec<LaptopOldAuraPower>,
    /// ASUS standardised control scheme from 2020 onwards
    pub rog: LaptopAuraPower,
}
