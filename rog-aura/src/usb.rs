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

/// Enable/disable LED control in various states such as
/// when the device is awake, suspended, shutting down or
/// booting.
///
/// # Bits for 0x19b6 keyboard model
///
/// ```text
/// byte 4 in the USB packet is for keyboard + logo power states
/// default is on, `ff`
/// Keyboard and logo use the full range of bits (almost)
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
/// |  1   |  2   | hex | action               | bit   |
/// |------|------|-----|----------------------|-------|
/// | 0000 | 0010 | 02  | lightbar off boot    | bit 2 |
/// | 0000 | 0100 | 04  | lightbar on          | bit 3 |
/// | 0000 | 1000 | 08  | lightbar off sleep   | bit 4 |
/// | 0001 | 0000 | 10  | lightbar shtdn off   | bit 5 |
///
/// # Bits for older 0x1866 keyboard model
///
/// Keybord and Light zone require Awake
/// |    byte 1   |    byte 2   |    byte 3   |          |          |
/// | 1    |  2   | 3    | 4    | 5    | 6    | function | hex      |
/// |------|------|------|------|------|------|----------|----------|
/// | 0000 | 0000 | 0000 | 0000 | 0000 | 0010 | Awake    | 00,00,02 |
/// | 1000 | 0000 | 0000 | 0000 | 0000 | 0010 | Keyboard | 80,00,02 |
/// | 0000 | 0100 | 0000 | 0101 | 0000 | 0010 | Lightbar | 04,05,02 |
/// | 1100 | 0011 | 0001 | 0010 | 0000 | 1001 | Boot/Sht | c3,12,09 |
/// | 0011 | 0000 | 0000 | 1000 | 0000 | 0100 | Sleep    | 30,08,04 |
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum AuraControl {
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

impl From<AuraControl> for u16 {
    fn from(a: AuraControl) -> Self {
        a as u16
    }
}

impl AuraControl {
    pub fn to_bytes(control: &[Self]) -> [u8; 2] {
        let mut a: u16 = 0;
        control.iter().for_each(|n| {
            a |= *n as u16;
        });
        [(a & 0xff) as u8, ((a & 0xff00) >> 8) as u8]
    }
}

impl BitOr<AuraControl> for AuraControl {
    type Output = u16;

    fn bitor(self, rhs: AuraControl) -> Self::Output {
        return self as u16 | rhs as u16;
    }
}

impl BitAnd<AuraControl> for AuraControl {
    type Output = u16;

    fn bitand(self, rhs: AuraControl) -> Self::Output {
        return self as u16 & rhs as u16;
    }
}

#[cfg(test)]
mod tests {
    use crate::usb::AuraControl;

    #[test]
    fn check_led_control_bytes() {
        // All on
        let byte1 = [
            AuraControl::BootLogo,
            AuraControl::BootKeyb,
            AuraControl::SleepLogo,
            AuraControl::SleepKeyb,
            AuraControl::AwakeLogo,
            AuraControl::AwakeKeyb,
            AuraControl::ShutdownLogo,
            AuraControl::ShutdownKeyb,
        ];
        let bytes = AuraControl::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xff);

        //
        let byte1 = [
            // AuraControl::BootLogo,
            AuraControl::BootKeyb,
            AuraControl::SleepLogo,
            AuraControl::SleepKeyb,
            AuraControl::AwakeLogo,
            AuraControl::AwakeKeyb,
            AuraControl::ShutdownLogo,
            AuraControl::ShutdownKeyb,
        ];
        let bytes = AuraControl::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfe);

        let byte1 = [
            AuraControl::BootLogo,
            // AuraControl::BootKeyb,
            AuraControl::SleepLogo,
            AuraControl::SleepKeyb,
            AuraControl::AwakeLogo,
            AuraControl::AwakeKeyb,
            AuraControl::ShutdownLogo,
            AuraControl::ShutdownKeyb,
        ];
        let bytes = AuraControl::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xfd);

        let byte1 = [
            AuraControl::BootLogo,
            AuraControl::BootKeyb,
            // AuraControl::SleepLogo,
            AuraControl::SleepKeyb,
            AuraControl::AwakeLogo,
            AuraControl::AwakeKeyb,
            AuraControl::ShutdownLogo,
            AuraControl::ShutdownKeyb,
        ];
        let bytes = AuraControl::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xef);

        let byte1 = [
            AuraControl::BootLogo,
            AuraControl::BootKeyb,
            AuraControl::SleepLogo,
            // AuraControl::SleepKeyb,
            AuraControl::AwakeLogo,
            AuraControl::AwakeKeyb,
            AuraControl::ShutdownLogo,
            AuraControl::ShutdownKeyb,
        ];
        let bytes = AuraControl::to_bytes(&byte1);
        println!("{:08b}", bytes[0]);
        assert_eq!(bytes[0], 0xdf);

        let byte2 = [
            AuraControl::AwakeBar,
            AuraControl::BootBar,
            AuraControl::SleepBar,
            AuraControl::ShutdownBar,
        ];
        let bytes = AuraControl::to_bytes(&byte2);
        println!("{:08b}", bytes[1]);
        assert_eq!(bytes[1], 0x1e);

        let byte2 = [
            AuraControl::AwakeBar,
            AuraControl::BootBar,
            // AuraControl::SleepBar,
            AuraControl::ShutdownBar,
        ];
        let bytes = AuraControl::to_bytes(&byte2);
        println!("{:08b}", bytes[1]);
        assert_eq!(bytes[1], 0x16);
    }
}
