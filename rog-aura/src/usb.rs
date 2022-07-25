use serde::{Deserialize, Serialize};
use std::ops::{BitAnd, BitOr};
#[cfg(feature = "dbus")]
use zvariant::Type;

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
        0x5A, 0xBA, 0xC5, 0xC4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub enum AuraDevice {
    X1854,
    X1869,
    X1866,
    #[default]
    X19B6,
    Unknown,
}

impl From<&str> for AuraDevice {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "1866" => AuraDevice::X1866,
            "1869" => AuraDevice::X1869,
            "1854" => AuraDevice::X1854,
            "19b6" => AuraDevice::X19B6,
            "0x1866" => AuraDevice::X1866,
            "0x1869" => AuraDevice::X1869,
            "0x1854" => AuraDevice::X1854,
            "0x19b6" => AuraDevice::X19B6,
            _ => AuraDevice::Unknown,
        }
    }
}

/// This struct is intended as a helper to pass args to generic dbus interface
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuraPowerDev {
    pub x1866: Vec<AuraDev1866>,
    pub x19b6: Vec<AuraDev19b6>,
}

/// # Bits for older 0x1866 keyboard model
///
/// Keybord and Lightbar require Awake, Boot and Sleep apply to both
/// Keybord and Lightbar regardless of if either are enabled (or Awake is enabled)
///
/// |   Byte 1   |   Byte 2   |   Byte 3   | function |   hex    |
/// |------------|------------|------------|----------|----------|
/// | 0000, 0000 | 0000, 0000 | 0000, 0010 | Awake    | 00,00,02 |
/// | 0000, 1000 | 0000, 0000 | 0000, 0000 | Keyboard | 08,00,00 |
/// | 0000, 0100 | 0000, 0101 | 0000, 0000 | Lightbar | 04,05,00 |
/// | 1100, 0011 | 0001, 0010 | 0000, 1001 | Boot/Sht | c3,12,09 |
/// | 0011, 0000 | 0000, 1000 | 0000, 0100 | Sleep    | 30,08,04 |
/// | 1111, 1111 | 0001, 1111 | 0000, 1111 | all on   |          |
///
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum AuraDev1866 {
    Awake = 0x000002,
    Keyboard = 0x080000,
    Lightbar = 0x040500,
    Boot = 0xc31209,
    Sleep = 0x300804,
}

impl From<AuraDev1866> for u32 {
    fn from(a: AuraDev1866) -> Self {
        a as u32
    }
}

impl AuraDev1866 {
    pub fn to_bytes(control: &[Self]) -> [u8; 3] {
        let mut a: u32 = 0;
        control.iter().for_each(|n| {
            a |= *n as u32;
        });
        [
            ((a & 0xff0000) >> 16) as u8,
            ((a & 0xff00) >> 8) as u8,
            (a & 0xff) as u8,
        ]
    }

    pub const fn dev_id() -> &'static str {
        "0x1866"
    }
}

impl BitOr<AuraDev1866> for AuraDev1866 {
    type Output = u32;

    fn bitor(self, rhs: AuraDev1866) -> Self::Output {
        return self as u32 | rhs as u32;
    }
}

impl BitAnd<AuraDev1866> for AuraDev1866 {
    type Output = u32;

    fn bitand(self, rhs: AuraDev1866) -> Self::Output {
        return self as u32 & rhs as u32;
    }
}

/// # Bits for 0x19b6 keyboard model
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
///
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum AuraDev19b6 {
    BootLogo = 1,
    BootKeyb = 1 << 1,
    AwakeLogo = 1 << 2,
    AwakeKeyb = 1 << 3,
    SleepLogo = 1 << 4,
    SleepKeyb = 1 << 5,
    ShutdownLogo = 1 << 6,
    ShutdownKeyb = 1 << 7,
    AwakeBar = 1 << 7 + 2,
    BootBar = 1 << 7 + 3,
    SleepBar = 1 << 7 + 4,
    ShutdownBar = 1 << 7 + 5,
}

impl From<AuraDev19b6> for u16 {
    fn from(a: AuraDev19b6) -> Self {
        a as u16
    }
}

impl AuraDev19b6 {
    pub fn to_bytes(control: &[Self]) -> [u8; 3] {
        let mut a: u16 = 0;
        control.iter().for_each(|n| {
            a |= *n as u16;
        });
        [(a & 0xff) as u8, ((a & 0xff00) >> 8) as u8, 0x00]
    }

    pub const fn dev_id() -> &'static str {
        "0x196b"
    }
}

impl BitOr<AuraDev19b6> for AuraDev19b6 {
    type Output = u16;

    fn bitor(self, rhs: AuraDev19b6) -> Self::Output {
        return self as u16 | rhs as u16;
    }
}

impl BitAnd<AuraDev19b6> for AuraDev19b6 {
    type Output = u16;

    fn bitand(self, rhs: AuraDev19b6) -> Self::Output {
        return self as u16 & rhs as u16;
    }
}

#[cfg(test)]
mod tests {
    use crate::usb::AuraDev19b6;

    use super::AuraDev1866;

    #[test]
    fn check_0x1866_control_bytes() {
        let bytes = [AuraDev1866::Keyboard, AuraDev1866::Awake];
        let bytes = AuraDev1866::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x08, 0x00, 0x02]);

        let bytes = [AuraDev1866::Lightbar, AuraDev1866::Awake];
        let bytes = AuraDev1866::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x04, 0x05, 0x02]);

        let bytes = [AuraDev1866::Sleep];
        let bytes = AuraDev1866::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x30, 0x08, 0x04]);

        let bytes = [AuraDev1866::Boot];
        let bytes = AuraDev1866::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xc3, 0x12, 0x09]);

        let bytes = [
            AuraDev1866::Keyboard,
            AuraDev1866::Lightbar,
            AuraDev1866::Awake,
            AuraDev1866::Sleep,
            AuraDev1866::Boot,
        ];

        let bytes = AuraDev1866::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xff, 0x1f, 0x000f]);
    }

    #[test]
    fn check_0x19b6_control_bytes() {
        // All on
        let byte1 = [
            AuraDev19b6::BootLogo,
            AuraDev19b6::BootKeyb,
            AuraDev19b6::SleepLogo,
            AuraDev19b6::SleepKeyb,
            AuraDev19b6::AwakeLogo,
            AuraDev19b6::AwakeKeyb,
            AuraDev19b6::ShutdownLogo,
            AuraDev19b6::ShutdownKeyb,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xff);

        //
        let byte1 = [
            // AuraControl::BootLogo,
            AuraDev19b6::BootKeyb,
            AuraDev19b6::SleepLogo,
            AuraDev19b6::SleepKeyb,
            AuraDev19b6::AwakeLogo,
            AuraDev19b6::AwakeKeyb,
            AuraDev19b6::ShutdownLogo,
            AuraDev19b6::ShutdownKeyb,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfe);

        let byte1 = [
            AuraDev19b6::BootLogo,
            // AuraControl::BootKeyb,
            AuraDev19b6::SleepLogo,
            AuraDev19b6::SleepKeyb,
            AuraDev19b6::AwakeLogo,
            AuraDev19b6::AwakeKeyb,
            AuraDev19b6::ShutdownLogo,
            AuraDev19b6::ShutdownKeyb,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfd);

        let byte1 = [
            AuraDev19b6::BootLogo,
            AuraDev19b6::BootKeyb,
            // AuraControl::SleepLogo,
            AuraDev19b6::SleepKeyb,
            AuraDev19b6::AwakeLogo,
            AuraDev19b6::AwakeKeyb,
            AuraDev19b6::ShutdownLogo,
            AuraDev19b6::ShutdownKeyb,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xef);

        let byte1 = [
            AuraDev19b6::BootLogo,
            AuraDev19b6::BootKeyb,
            AuraDev19b6::SleepLogo,
            // AuraControl::SleepKeyb,
            AuraDev19b6::AwakeLogo,
            AuraDev19b6::AwakeKeyb,
            AuraDev19b6::ShutdownLogo,
            AuraDev19b6::ShutdownKeyb,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xdf);

        let byte2 = [
            AuraDev19b6::AwakeBar,
            AuraDev19b6::BootBar,
            AuraDev19b6::SleepBar,
            AuraDev19b6::ShutdownBar,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte2);
        println!("{:08b}", bytes[1]);
        assert_eq!(bytes[1], 0x1e);

        let byte2 = [
            AuraDev19b6::AwakeBar,
            AuraDev19b6::BootBar,
            // AuraControl::SleepBar,
            AuraDev19b6::ShutdownBar,
        ];
        let bytes = AuraDev19b6::to_bytes(&byte2);
        println!("{:08b}", bytes[1]);
        assert_eq!(bytes[1], 0x16);
    }
}
