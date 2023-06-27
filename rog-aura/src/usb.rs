use std::fmt::Debug;
use std::ops::{BitAnd, BitOr};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
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

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AuraPowerDev {
    pub tuf: Vec<AuraDevTuf>,
    pub x1866: Vec<AuraDevRog1>,
    pub x19b6: Vec<AuraDevRog2>,
}

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// # Bits for older 0x1866 keyboard model
///
/// Keybord and Lightbar require Awake, Boot and Sleep apply to both
/// Keybord and Lightbar regardless of if either are enabled (or Awake is
/// enabled)
///
/// |   Byte 1   |   Byte 2   |   Byte 3   | function |   hex    |
/// |------------|------------|------------|----------|----------|
/// | 0000, 0000 | 0000, 0000 | 0000, 0010 | Awake    | 00,00,02 |
/// | 0000, 1000 | 0000, 0000 | 0000, 0000 | Keyboard | 08,00,00 |
/// | 0000, 0100 | 0000, 0101 | 0000, 0000 | Lightbar | 04,05,00 |
/// | 1100, 0011 | 0001, 0010 | 0000, 1001 | Boot/Sht | c3,12,09 |
/// | 0011, 0000 | 0000, 1000 | 0000, 0100 | Sleep    | 30,08,04 |
/// | 1111, 1111 | 0001, 1111 | 0000, 1111 | all on   |          |
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
/// | Byte 1 | Byte 2  | Byte 3  | Byte 4  | Label    |
/// |--------|---------|---------|---------|----------|
/// |00000001| 00000000| 00000000| 00000000|boot_logo_|
/// |00000010| 00000000| 00000000| 00000000|boot_keyb_|
/// |00000100| 00000000| 00000000| 00000000|awake_logo|
/// |00001000| 00000000| 00000000| 00000000|awake_keyb|
/// |00010000| 00000000| 00000000| 00000000|sleep_logo|
/// |00100000| 00000000| 00000000| 00000000|sleep_keyb|
/// |01000000| 00000000| 00000000| 00000000|shut_logo_|
/// |10000000| 00000000| 00000000| 00000000|shut_keyb_|
/// |00000000| 00000010| 00000000| 00000000|boot_bar__|
/// |00000000| 00000100| 00000000| 00000000|awake_bar_|
/// |00000000| 00001000| 00000000| 00000000|sleep_bar_|
/// |00000000| 00010000| 00000000| 00000000|shut_bar__|
/// |00000000| 00000000| 00000001| 00000000|boot_lid__|
/// |00000000| 00000000| 00000010| 00000000|awkae_lid_|
/// |00000000| 00000000| 00000100| 00000000|sleep_lid_|
/// |00000000| 00000000| 00001000| 00000000|shut_lid__|
/// |00000000| 00000000| 00000000| 00000001|boot_rear_|
/// |00000000| 00000000| 00000000| 00000010|awake_rear|
/// |00000000| 00000000| 00000000| 00000100|sleep_rear|
/// |00000000| 00000000| 00000000| 00001000|shut_rear_|
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    fn check_0x19b6_control_bytes_binary_rep() {
        fn to_binary_string(bytes: &[AuraDevRog2]) -> String {
            let bytes = AuraDevRog2::to_bytes(bytes);
            format!(
                "{:08b}, {:08b}, {:08b}, {:08b}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            )
        }

        let boot_logo_ = to_binary_string(&[AuraDevRog2::BootLogo]);
        let boot_keyb_ = to_binary_string(&[AuraDevRog2::BootKeyb]);
        let sleep_logo = to_binary_string(&[AuraDevRog2::SleepLogo]);
        let sleep_keyb = to_binary_string(&[AuraDevRog2::SleepKeyb]);
        let awake_logo = to_binary_string(&[AuraDevRog2::AwakeLogo]);
        let awake_keyb = to_binary_string(&[AuraDevRog2::AwakeKeyb]);
        let shut_logo_ = to_binary_string(&[AuraDevRog2::ShutdownLogo]);
        let shut_keyb_ = to_binary_string(&[AuraDevRog2::ShutdownKeyb]);
        let boot_bar__ = to_binary_string(&[AuraDevRog2::BootBar]);
        let awake_bar_ = to_binary_string(&[AuraDevRog2::AwakeBar]);
        let sleep_bar_ = to_binary_string(&[AuraDevRog2::SleepBar]);
        let shut_bar__ = to_binary_string(&[AuraDevRog2::ShutdownBar]);
        let boot_lid__ = to_binary_string(&[AuraDevRog2::BootLid]);
        let awkae_lid_ = to_binary_string(&[AuraDevRog2::AwakeLid]);
        let sleep_lid_ = to_binary_string(&[AuraDevRog2::SleepLid]);
        let shut_lid__ = to_binary_string(&[AuraDevRog2::ShutdownLid]);
        let boot_rear_ = to_binary_string(&[AuraDevRog2::BootRearGlow]);
        let awake_rear = to_binary_string(&[AuraDevRog2::AwakeRearGlow]);
        let sleep_rear = to_binary_string(&[AuraDevRog2::SleepRearGlow]);
        let shut_rear_ = to_binary_string(&[AuraDevRog2::ShutdownRearGlow]);

        assert_eq!(boot_logo_, "00000001, 00000000, 00000000, 00000000");
        assert_eq!(boot_keyb_, "00000010, 00000000, 00000000, 00000000");
        assert_eq!(awake_logo, "00000100, 00000000, 00000000, 00000000");
        assert_eq!(awake_keyb, "00001000, 00000000, 00000000, 00000000");
        assert_eq!(sleep_logo, "00010000, 00000000, 00000000, 00000000");
        assert_eq!(sleep_keyb, "00100000, 00000000, 00000000, 00000000");
        assert_eq!(shut_logo_, "01000000, 00000000, 00000000, 00000000");
        assert_eq!(shut_keyb_, "10000000, 00000000, 00000000, 00000000");
        //
        assert_eq!(boot_bar__, "00000000, 00000010, 00000000, 00000000");
        assert_eq!(awake_bar_, "00000000, 00000100, 00000000, 00000000");
        assert_eq!(sleep_bar_, "00000000, 00001000, 00000000, 00000000");
        assert_eq!(shut_bar__, "00000000, 00010000, 00000000, 00000000");
        //
        assert_eq!(boot_lid__, "00000000, 00000000, 00000001, 00000000");
        assert_eq!(awkae_lid_, "00000000, 00000000, 00000010, 00000000");
        assert_eq!(sleep_lid_, "00000000, 00000000, 00000100, 00000000");
        assert_eq!(shut_lid__, "00000000, 00000000, 00001000, 00000000");
        //
        assert_eq!(boot_rear_, "00000000, 00000000, 00000000, 00000001");
        assert_eq!(awake_rear, "00000000, 00000000, 00000000, 00000010");
        assert_eq!(sleep_rear, "00000000, 00000000, 00000000, 00000100");
        assert_eq!(shut_rear_, "00000000, 00000000, 00000000, 00001000");

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
            AuraDevRog2::BootBar,
            AuraDevRog2::AwakeBar,
            AuraDevRog2::SleepBar,
            AuraDevRog2::ShutdownBar,
            AuraDevRog2::AwakeLid,
            AuraDevRog2::BootLid,
            AuraDevRog2::SleepLid,
            AuraDevRog2::ShutdownLid,
            AuraDevRog2::AwakeRearGlow,
            AuraDevRog2::BootRearGlow,
            AuraDevRog2::SleepRearGlow,
            AuraDevRog2::ShutdownRearGlow,
        ];
        let out = to_binary_string(&byte1);
        assert_eq!(out, "11111111, 00011110, 00001111, 00001111");
    }
}
