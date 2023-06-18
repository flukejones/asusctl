use std::fmt::Debug;
use std::ops::{BitAnd, BitOr};

use serde::{Deserialize, Serialize};
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;

pub const LED_INIT1: [u8; 2] = [0x5d, 0xb9];
pub const LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
pub const LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
pub const LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
pub const LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

// Only these two packets must be 17 bytes
pub const LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5a, 0xba, 0xc5, 0xc4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Default)]
pub enum AuraDevice {
    Tuf,
    X1854,
    X1869,
    X1866,
    X18c6,
    #[default]
    X19b6,
    X1a30,
    Unknown,
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
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// This struct is intended as a helper to pass args to generic dbus interface
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AuraPowerDev {
    pub tuf: Vec<AuraDevTuf>,
    pub x1866: Vec<AuraDevRog1>,
    pub x19b6: Vec<AuraDevRog2>,
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AuraDevTuf {
    Boot,
    Awake,
    Sleep,
    Keyboard,
}

impl AuraDevTuf {
    pub const fn dev_id() -> &'static str {
        "tuf"
    }
}

/// # Bits for older 0x1866, 0x1869, 0x1854 keyboard models
///
/// Keybord and Lightbar require Awake, Boot and Sleep apply to both
/// Keybord and Lightbar regardless of if either are enabled (or Awake is
/// enabled)
///
/// |   Byte 1   |   Byte 2   |   Byte 3   |   Byte 4   | function  |   hex
/// |
/// |------------|------------|------------|------------|-----------|-------------|
/// | 0000, 0000 | 0000, 0000 | 0000, 0010 | 0000, 0000 | Awake     |
/// 00,00,02,00 | | 0000, 1000 | 0000, 0000 | 0000, 0000 | 0000, 0000 | Keyboard
/// | 08,00,00,00 | | 0000, 0100 | 0000, 0101 | 0000, 0000 | 0000, 0000 |
/// Lightbar  | 04,05,00,00 | | 1100, 0011 | 0001, 0010 | 0000, 1001 | 0000,
/// 0000 | Boot/Sht  | c3,12,09,00 | | 0011, 0000 | 0000, 1000 | 0000, 0100 |
/// 0000, 0000 | Sleep     | 30,08,04,00 | | 1111, 1111 | 0001, 1111 | 0000,
/// 1111 | 0000, 0000 | all on    |             | | 0000, 0000 | 0000, 0000 |
/// 0000, 0000 | 0000, 0010 | Rear Glow | 00,00,00,02 |
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AuraDevRog1 {
    Awake = 0x000002,
    Keyboard = 0x080000,
    Lightbar = 0x040500,
    Boot = 0xc31209,
    Sleep = 0x300804,
}

impl From<AuraDevRog1> for u32 {
    fn from(a: AuraDevRog1) -> Self {
        a as u32
    }
}

impl AuraDevRog1 {
    pub fn to_bytes(control: &[Self]) -> [u8; 4] {
        let mut a: u32 = 0;
        for n in control {
            a |= *n as u32;
        }
        [
            ((a & 0xff0000) >> 16) as u8,
            ((a & 0xff00) >> 8) as u8,
            (a & 0xff) as u8,
            0x00,
        ]
    }

    pub const fn dev_id() -> &'static str {
        "0x1866"
    }
}

impl BitOr<AuraDevRog1> for AuraDevRog1 {
    type Output = u32;

    fn bitor(self, rhs: AuraDevRog1) -> Self::Output {
        self as u32 | rhs as u32
    }
}

impl BitAnd<AuraDevRog1> for AuraDevRog1 {
    type Output = u32;

    fn bitand(self, rhs: AuraDevRog1) -> Self::Output {
        self as u32 & rhs as u32
    }
}

/// # Bits for newer 0x18c6, 0x19B6, 0x1a30, keyboard models
///
/// byte 4 in the USB packet is for keyboard + logo power states
/// default is on, `ff`
/// Keyboard and logo use the full range of bits (almost)
///
/// | n1   | n2   | hex | action                | bit   |
/// |------|------|-----|-----------------------|-------|
/// | 0000 | 0000 | 00  | all off               |       |
/// | 0000 | 0001 | 01  | logo boot             | bit 1 |
/// | 0000 | 0010 | 02  | keyboard boot         | bit 2 |
/// | 0000 | 0100 | 04  | logo awake            | bit 3 |
/// | 0000 | 1000 | 08  | keyboard awake        | bit 4 |
/// | 0001 | 0000 | 10  | logo sleep off        | bit 5 |
/// | 0010 | 0000 | 20  | keyboard sleep        | bit 6 |
/// | 0100 | 0000 | 40  | logo shutdown off     | bit 7 |
/// | 1000 | 0000 | 80  | keyboard shutdown off | bit 8 |
///
/// byte 5 = lightbar
///
/// |  1   |  2   | hex | action               | bit   |
/// |------|------|-----|----------------------|-------|
/// | 0000 | 0010 | 02  | lightbar off boot    | bit 2 |
/// | 0000 | 0100 | 04  | lightbar on          | bit 3 |
/// | 0000 | 1000 | 08  | lightbar off sleep   | bit 4 |
/// | 0001 | 0000 | 10  | lightbar shtdn off   | bit 5 |
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AuraDevRog2 {
    BootLogo = 1,
    BootKeyb = 1 << 1,
    AwakeLogo = 1 << 2,
    AwakeKeyb = 1 << 3,
    SleepLogo = 1 << 4,
    SleepKeyb = 1 << 5,
    ShutdownLogo = 1 << 6,
    ShutdownKeyb = 1 << 7,
    BootBar = 1 << (7 + 2),
    AwakeBar = 1 << (7 + 3),
    SleepBar = 1 << (7 + 4),
    ShutdownBar = 1 << (7 + 5),
    BootLid = 1 << (15 + 1),
    AwakeLid = 1 << (15 + 2),
    SleepLid = 1 << (15 + 3),
    ShutdownLid = 1 << (15 + 4),
    BootRearGlow = 1 << (23 + 1),
    AwakeRearGlow = 1 << (23 + 2),
    SleepRearGlow = 1 << (23 + 3),
    ShutdownRearGlow = 1 << (23 + 4),
}

impl From<AuraDevRog2> for u32 {
    fn from(a: AuraDevRog2) -> Self {
        a as u32
    }
}

impl AuraDevRog2 {
    pub fn to_bytes(control: &[Self]) -> [u8; 4] {
        let mut a: u32 = 0;
        for n in control {
            a |= *n as u32;
        }
        [
            (a & 0xff) as u8,
            ((a & 0xff00) >> 8) as u8,
            ((a & 0xff0000) >> 16) as u8,
            ((a & 0xff000000) >> 24) as u8,
        ]
    }

    pub const fn dev_id() -> &'static str {
        "0x196b"
    }
}

impl BitOr<AuraDevRog2> for AuraDevRog2 {
    type Output = u16;

    fn bitor(self, rhs: AuraDevRog2) -> Self::Output {
        self as u16 | rhs as u16
    }
}

impl BitAnd<AuraDevRog2> for AuraDevRog2 {
    type Output = u16;

    fn bitand(self, rhs: AuraDevRog2) -> Self::Output {
        self as u16 & rhs as u16
    }
}

#[cfg(test)]
mod tests {
    use super::AuraDevRog1;
    use crate::usb::AuraDevRog2;

    #[test]
    fn check_0x1866_control_bytes() {
        let bytes = [AuraDevRog1::Keyboard, AuraDevRog1::Awake];
        let bytes = AuraDevRog1::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x08, 0x00, 0x02, 0x00]);

        let bytes = [AuraDevRog1::Lightbar, AuraDevRog1::Awake];
        let bytes = AuraDevRog1::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x04, 0x05, 0x02, 0x00]);

        let bytes = [AuraDevRog1::Sleep];
        let bytes = AuraDevRog1::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x30, 0x08, 0x04, 0x00]);

        let bytes = [AuraDevRog1::Boot];
        let bytes = AuraDevRog1::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xc3, 0x12, 0x09, 0x00]);

        let bytes = [
            AuraDevRog1::Keyboard,
            AuraDevRog1::Lightbar,
            AuraDevRog1::Awake,
            AuraDevRog1::Sleep,
            AuraDevRog1::Boot,
        ];

        let bytes = AuraDevRog1::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xff, 0x1f, 0x000f, 0x00]);
    }

    #[test]
    fn check_0x19b6_control_bytes() {
        // All on
        let byte1 = [
            AuraDevRog2::BootLogo,
            AuraDevRog2::BootKeyb,
            AuraDevRog2::SleepLogo,
            AuraDevRog2::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte1);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[0], 0xff);

        //
        let byte1 = [
            // AuraControl::BootLogo,
            AuraDevRog2::BootKeyb,
            AuraDevRog2::SleepLogo,
            AuraDevRog2::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfe);

        let byte1 = [
            AuraDevRog2::BootLogo,
            // AuraControl::BootKeyb,
            AuraDevRog2::SleepLogo,
            AuraDevRog2::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfd);

        let byte1 = [
            AuraDevRog2::BootLogo,
            AuraDevRog2::BootKeyb,
            // AuraControl::SleepLogo,
            AuraDevRog2::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xef);

        let byte1 = [
            AuraDevRog2::BootLogo,
            AuraDevRog2::BootKeyb,
            AuraDevRog2::SleepLogo,
            // AuraControl::SleepKeyb,
            AuraDevRog2::AwakeLogo,
            AuraDevRog2::AwakeKeyb,
            AuraDevRog2::ShutdownLogo,
            AuraDevRog2::ShutdownKeyb,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xdf);

        let byte2 = [
            AuraDevRog2::BootBar,
            AuraDevRog2::AwakeBar,
            AuraDevRog2::SleepBar,
            AuraDevRog2::ShutdownBar,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte2);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[1], 0x1e);

        let byte2 = [
            AuraDevRog2::BootBar,
            AuraDevRog2::AwakeBar,
            // AuraControl::SleepBar,
            AuraDevRog2::ShutdownBar,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte2);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[1], 0x16);

        let byte3 = [
            AuraDevRog2::AwakeLid,
            AuraDevRog2::BootLid,
            AuraDevRog2::SleepLid,
            AuraDevRog2::ShutdownLid,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte3);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[2], 0x0f);

        let byte3 = [
            // AuraDev19b6::AwakeLid,
            AuraDevRog2::BootLid,
            AuraDevRog2::SleepLid,
            AuraDevRog2::ShutdownLid,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte3);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[2], 0x0d);

        let byte3 = [
            AuraDevRog2::AwakeLid,
            AuraDevRog2::BootLid,
            // AuraControl::SleepLid,
            AuraDevRog2::ShutdownLid,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte3);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[2], 0x0b);

        let byte3 = [
            AuraDevRog2::AwakeLid,
            AuraDevRog2::BootLid,
            AuraDevRog2::SleepLid,
            // AuraDev19b6::ShutdownLid,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte3);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[2], 0x07);

        let byte3 = [
            AuraDevRog2::AwakeLid,
            // AuraDev19b6::BootLid,
            AuraDevRog2::SleepLid,
            // AuraDev19b6::ShutdownLid,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte3);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes[2], 0x06);

        let byte4 = [
            // AuraDev19b6::AwakeRearBar,
            AuraDevRog2::BootRearGlow,
            AuraDevRog2::SleepRearGlow,
            AuraDevRog2::ShutdownRearGlow,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte4);
        println!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        assert_eq!(bytes[3], 0x0d);

        let byte4 = [
            AuraDevRog2::AwakeRearGlow,
            AuraDevRog2::BootRearGlow,
            // AuraDevRog2::SleepRearBar,
            AuraDevRog2::ShutdownRearGlow,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte4);
        println!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        assert_eq!(bytes[3], 0x0b);

        let byte4 = [
            AuraDevRog2::AwakeRearGlow,
            AuraDevRog2::BootRearGlow,
            AuraDevRog2::SleepRearGlow,
            // AuraDevRog2::ShutdownRearBar,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte4);
        println!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        assert_eq!(bytes[3], 0x07);

        let byte4 = [
            AuraDevRog2::AwakeRearGlow,
            // AuraDevRog2::BootRearBar,
            AuraDevRog2::SleepRearGlow,
            // AuraDevRog2::ShutdownRearBar,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte4);
        println!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        assert_eq!(bytes[3], 0x06);

        let byte4 = [
            AuraDevRog2::AwakeRearGlow,
            // AuraDevRog2::BootRearBar,
            // AuraDevRog2::SleepRearBar,
            // AuraDevRog2::ShutdownRearBar,
        ];
        let bytes = AuraDevRog2::to_bytes(&byte4);
        println!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        );
        assert_eq!(bytes[3], 0x02);
    }
}
