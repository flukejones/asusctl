use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::aura_detection::PowerZones;

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KbAuraPowerState {
    pub zone: PowerZones,
    pub boot: bool,
    pub awake: bool,
    pub sleep: bool,
    pub shutdown: bool,
}

impl Default for KbAuraPowerState {
    /// Defaults all to off
    fn default() -> Self {
        Self {
            zone: PowerZones::Keyboard,
            boot: false,
            awake: false,
            sleep: false,
            shutdown: false,
        }
    }
}

impl KbAuraPowerState {
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
pub struct AuraPower {
    pub keyboard: KbAuraPowerState,
    pub logo: KbAuraPowerState,
    pub lightbar: KbAuraPowerState,
    pub lid: KbAuraPowerState,
    pub rear_glow: KbAuraPowerState,
}

impl AuraPower {
    pub fn new_all_on() -> Self {
        Self {
            keyboard: KbAuraPowerState {
                zone: PowerZones::Keyboard,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            logo: KbAuraPowerState {
                zone: PowerZones::Logo,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            lightbar: KbAuraPowerState {
                zone: PowerZones::Lightbar,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            lid: KbAuraPowerState {
                zone: PowerZones::Lid,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true,
            },
            rear_glow: KbAuraPowerState {
                zone: PowerZones::RearGlow,
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

#[test]
fn check_0x19b6_control_bytes_binary_rep() {
    fn to_binary_string(power: &AuraPower) -> String {
        let bytes = power.to_bytes();
        format!(
            "{:08b}, {:08b}, {:08b}, {:08b}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    }

    let boot_logo_ = to_binary_string(&AuraPower {
        logo: KbAuraPowerState {
            boot: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let boot_keyb_ = to_binary_string(&AuraPower {
        keyboard: KbAuraPowerState {
            boot: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let sleep_logo = to_binary_string(&AuraPower {
        logo: KbAuraPowerState {
            sleep: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let sleep_keyb = to_binary_string(&AuraPower {
        keyboard: KbAuraPowerState {
            sleep: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let awake_logo = to_binary_string(&AuraPower {
        logo: KbAuraPowerState {
            awake: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let awake_keyb = to_binary_string(&AuraPower {
        keyboard: KbAuraPowerState {
            awake: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let shut_logo_ = to_binary_string(&AuraPower {
        logo: KbAuraPowerState {
            shutdown: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let shut_keyb_ = to_binary_string(&AuraPower {
        keyboard: KbAuraPowerState {
            shutdown: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let boot_bar__ = to_binary_string(&AuraPower {
        lightbar: KbAuraPowerState {
            boot: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let awake_bar_ = to_binary_string(&AuraPower {
        lightbar: KbAuraPowerState {
            awake: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let sleep_bar_ = to_binary_string(&AuraPower {
        lightbar: KbAuraPowerState {
            sleep: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let shut_bar__ = to_binary_string(&AuraPower {
        lightbar: KbAuraPowerState {
            shutdown: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let boot_lid__ = to_binary_string(&AuraPower {
        lid: KbAuraPowerState {
            boot: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let awkae_lid_ = to_binary_string(&AuraPower {
        lid: KbAuraPowerState {
            awake: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let sleep_lid_ = to_binary_string(&AuraPower {
        lid: KbAuraPowerState {
            sleep: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let shut_lid__ = to_binary_string(&AuraPower {
        lid: KbAuraPowerState {
            shutdown: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let boot_rear_ = to_binary_string(&AuraPower {
        rear_glow: KbAuraPowerState {
            boot: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let awake_rear = to_binary_string(&AuraPower {
        rear_glow: KbAuraPowerState {
            awake: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let sleep_rear = to_binary_string(&AuraPower {
        rear_glow: KbAuraPowerState {
            sleep: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let shut_rear_ = to_binary_string(&AuraPower {
        rear_glow: KbAuraPowerState {
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
    assert_eq!(awkae_lid_, "00000000, 00000000, 00000010, 00000000");
    assert_eq!(sleep_lid_, "00000000, 00000000, 00000100, 00000000");
    assert_eq!(shut_lid__, "00000000, 00000000, 00001000, 00000000");
    //
    assert_eq!(boot_rear_, "00000000, 00000000, 00000000, 00000001");
    assert_eq!(awake_rear, "00000000, 00000000, 00000000, 00000010");
    assert_eq!(sleep_rear, "00000000, 00000000, 00000000, 00000100");
    assert_eq!(shut_rear_, "00000000, 00000000, 00000000, 00001000");

    // All on
    let byte1 = AuraPower::new_all_on();
    let out = to_binary_string(&byte1);
    assert_eq!(out, "11111111, 00011110, 00001111, 00001111");
}
