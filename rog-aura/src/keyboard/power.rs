//! Power state for Laptop MCU RGB/LED. This is generally for newer
//! 0x18c6, 0x19B6, 0x1a30, keyboard models (2021+)
use std::fmt::Debug;
use std::ops::{BitAnd, BitOr};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::aura_detection::PowerZones;

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuraPowerState {
    pub boot: bool,
    pub awake: bool,
    pub sleep: bool,
    pub shutdown: bool,
}

impl Default for AuraPowerState {
    /// Defaults all to off
    fn default() -> Self {
        Self {
            boot: false,
            awake: false,
            sleep: false,
            shutdown: false,
        }
    }
}

impl AuraPowerState {
    pub fn to_byte(&self, zone: PowerZones) -> u32 {
        match zone {
            PowerZones::Logo => {
                self.boot as u32
                    | (self.awake as u32) << 2
                    | (self.sleep as u32) << 4
                    | (self.shutdown as u32) << 6
            }
            PowerZones::Keyboard => {
                (self.boot as u32) << 1
                    | (self.awake as u32) << 3
                    | (self.sleep as u32) << 5
                    | (self.shutdown as u32) << 7
            }
            PowerZones::Lightbar => {
                (self.boot as u32) << (7 + 2)
                    | (self.awake as u32) << (7 + 3)
                    | (self.sleep as u32) << (7 + 4)
                    | (self.shutdown as u32) << (7 + 5)
            }
            PowerZones::Lid => {
                (self.boot as u32) << (15 + 1)
                    | (self.awake as u32) << (15 + 2)
                    | (self.sleep as u32) << (15 + 3)
                    | (self.shutdown as u32) << (15 + 4)
            }
            PowerZones::RearGlow => {
                (self.boot as u32) << (23 + 1)
                    | (self.awake as u32) << (23 + 2)
                    | (self.sleep as u32) << (23 + 3)
                    | (self.shutdown as u32) << (23 + 4)
            }
        }
    }
}

/// Track and control the Aura keyboard power state
///
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
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaptopAuraPower {
    pub keyboard: AuraPowerState,
    pub logo: AuraPowerState,
    pub lightbar: AuraPowerState,
    pub lid: AuraPowerState,
    pub rear_glow: AuraPowerState,
}

impl LaptopAuraPower {
    pub fn new_all_on() -> Self {
        Self {
            keyboard: AuraPowerState {
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            logo: AuraPowerState {
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            lightbar: AuraPowerState {
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            lid: AuraPowerState {
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            rear_glow: AuraPowerState {
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let mut a: u32 = 0;
        a |= self.keyboard.to_byte(PowerZones::Keyboard);
        a |= self.logo.to_byte(PowerZones::Logo);
        a |= self.lid.to_byte(PowerZones::Lid);
        a |= self.lightbar.to_byte(PowerZones::Lightbar);
        a |= self.rear_glow.to_byte(PowerZones::RearGlow);
        [
            (a & 0xff) as u8,
            ((a & 0xff00) >> 8) as u8,
            ((a & 0xff0000) >> 16) as u8,
            ((a & 0xff000000) >> 24) as u8,
        ]
    }
}

#[typeshare]
#[cfg_attr(
    feature = "dbus",
    derive(Type, Value, OwnedValue),
    zvariant(signature = "u")
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LaptopTufAuraPower {
    Boot = 0,
    Awake = 1,
    Sleep = 2,
    Keyboard = 3,
}

impl LaptopTufAuraPower {
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
#[cfg_attr(
    feature = "dbus",
    derive(Type, Value, OwnedValue),
    zvariant(signature = "u")
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LaptopOldAuraPower {
    Awake = 0x000002,
    Keyboard = 0x080000,
    Lightbar = 0x040500,
    Boot = 0xc31209,
    Sleep = 0x300804,
}

impl From<LaptopOldAuraPower> for u32 {
    fn from(a: LaptopOldAuraPower) -> Self {
        a as u32
    }
}

impl LaptopOldAuraPower {
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

impl BitOr<LaptopOldAuraPower> for LaptopOldAuraPower {
    type Output = u32;

    fn bitor(self, rhs: LaptopOldAuraPower) -> Self::Output {
        self as u32 | rhs as u32
    }
}

impl BitAnd<LaptopOldAuraPower> for LaptopOldAuraPower {
    type Output = u32;

    fn bitand(self, rhs: LaptopOldAuraPower) -> Self::Output {
        self as u32 & rhs as u32
    }
}

#[cfg(test)]
mod test {
    use crate::keyboard::{AuraPowerState, LaptopAuraPower, LaptopOldAuraPower};

    #[test]
    fn check_0x1866_control_bytes() {
        let bytes = [LaptopOldAuraPower::Keyboard, LaptopOldAuraPower::Awake];
        let bytes = LaptopOldAuraPower::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x08, 0x00, 0x02, 0x00]);

        let bytes = [LaptopOldAuraPower::Lightbar, LaptopOldAuraPower::Awake];
        let bytes = LaptopOldAuraPower::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x04, 0x05, 0x02, 0x00]);

        let bytes = [LaptopOldAuraPower::Sleep];
        let bytes = LaptopOldAuraPower::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0x30, 0x08, 0x04, 0x00]);

        let bytes = [LaptopOldAuraPower::Boot];
        let bytes = LaptopOldAuraPower::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xc3, 0x12, 0x09, 0x00]);

        let bytes = [
            LaptopOldAuraPower::Keyboard,
            LaptopOldAuraPower::Lightbar,
            LaptopOldAuraPower::Awake,
            LaptopOldAuraPower::Sleep,
            LaptopOldAuraPower::Boot,
        ];

        let bytes = LaptopOldAuraPower::to_bytes(&bytes);
        println!("{:08b}, {:08b}, {:08b}", bytes[0], bytes[1], bytes[2]);
        assert_eq!(bytes, [0xff, 0x1f, 0x000f, 0x00]);
    }

    #[test]
    fn check_0x19b6_control_bytes_binary_rep() {
        fn to_binary_string(power: &LaptopAuraPower) -> String {
            let bytes = power.to_bytes();
            format!(
                "{:08b}, {:08b}, {:08b}, {:08b}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            )
        }

        let boot_logo_ = to_binary_string(&LaptopAuraPower {
            logo: AuraPowerState {
                boot: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let boot_keyb_ = to_binary_string(&LaptopAuraPower {
            keyboard: AuraPowerState {
                boot: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let sleep_logo = to_binary_string(&LaptopAuraPower {
            logo: AuraPowerState {
                sleep: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let sleep_keyb = to_binary_string(&LaptopAuraPower {
            keyboard: AuraPowerState {
                sleep: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let awake_logo = to_binary_string(&LaptopAuraPower {
            logo: AuraPowerState {
                awake: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let awake_keyb = to_binary_string(&LaptopAuraPower {
            keyboard: AuraPowerState {
                awake: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let shut_logo_ = to_binary_string(&LaptopAuraPower {
            logo: AuraPowerState {
                shutdown: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let shut_keyb_ = to_binary_string(&LaptopAuraPower {
            keyboard: AuraPowerState {
                shutdown: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let boot_bar__ = to_binary_string(&LaptopAuraPower {
            lightbar: AuraPowerState {
                boot: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let awake_bar_ = to_binary_string(&LaptopAuraPower {
            lightbar: AuraPowerState {
                awake: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let sleep_bar_ = to_binary_string(&LaptopAuraPower {
            lightbar: AuraPowerState {
                sleep: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let shut_bar__ = to_binary_string(&LaptopAuraPower {
            lightbar: AuraPowerState {
                shutdown: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let boot_lid__ = to_binary_string(&LaptopAuraPower {
            lid: AuraPowerState {
                boot: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let awake_lid_ = to_binary_string(&LaptopAuraPower {
            lid: AuraPowerState {
                awake: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let sleep_lid_ = to_binary_string(&LaptopAuraPower {
            lid: AuraPowerState {
                sleep: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let shut_lid__ = to_binary_string(&LaptopAuraPower {
            lid: AuraPowerState {
                shutdown: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let boot_rear_ = to_binary_string(&LaptopAuraPower {
            rear_glow: AuraPowerState {
                boot: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let awake_rear = to_binary_string(&LaptopAuraPower {
            rear_glow: AuraPowerState {
                awake: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let sleep_rear = to_binary_string(&LaptopAuraPower {
            rear_glow: AuraPowerState {
                sleep: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let shut_rear_ = to_binary_string(&LaptopAuraPower {
            rear_glow: AuraPowerState {
                shutdown: true,
                ..Default::default()
            },
            ..Default::default()
        });

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
        assert_eq!(awake_lid_, "00000000, 00000000, 00000010, 00000000");
        assert_eq!(sleep_lid_, "00000000, 00000000, 00000100, 00000000");
        assert_eq!(shut_lid__, "00000000, 00000000, 00001000, 00000000");
        //
        assert_eq!(boot_rear_, "00000000, 00000000, 00000000, 00000001");
        assert_eq!(awake_rear, "00000000, 00000000, 00000000, 00000010");
        assert_eq!(sleep_rear, "00000000, 00000000, 00000000, 00000100");
        assert_eq!(shut_rear_, "00000000, 00000000, 00000000, 00001000");

        // All on
        let byte1 = LaptopAuraPower::new_all_on();
        let out = to_binary_string(&byte1);
        assert_eq!(out, "11111111, 00011110, 00001111, 00001111");
    }
}
