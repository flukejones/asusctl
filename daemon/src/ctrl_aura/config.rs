use std::collections::{BTreeMap, HashSet};

use config_traits::{StdConfig, StdConfigLoad};
use rog_aura::aura_detection::{LaptopLedData, ASUS_KEYBOARD_DEVICES};
use rog_aura::usb::{AuraDev1866, AuraDev19b6, AuraDevTuf, AuraDevice, AuraPowerDev};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Direction, LedBrightness, Speed, GRADIENT};
use rog_platform::hid_raw::HidRaw;
use rog_platform::keyboard_led::KeyboardLed;
use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE: &str = "aura.ron";

/// Enable/disable LED control in various states such as
/// when the device is awake, suspended, shutting down or
/// booting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuraPowerConfig {
    AuraDevTuf(HashSet<AuraDevTuf>),
    AuraDev1866(HashSet<AuraDev1866>),
    AuraDev19b6(HashSet<AuraDev19b6>),
}

impl AuraPowerConfig {
    /// Invalid for TUF laptops
    pub fn to_bytes(control: &Self) -> [u8; 3] {
        match control {
            AuraPowerConfig::AuraDevTuf(_) => [0, 0, 0],
            AuraPowerConfig::AuraDev1866(c) => {
                let c: Vec<AuraDev1866> = c.iter().copied().collect();
                AuraDev1866::to_bytes(&c)
            }
            AuraPowerConfig::AuraDev19b6(c) => {
                let c: Vec<AuraDev19b6> = c.iter().copied().collect();
                AuraDev19b6::to_bytes(&c)
            }
        }
    }

    pub fn to_tuf_bool_array(control: &Self) -> Option<[bool; 5]> {
        if let Self::AuraDevTuf(c) = control {
            return Some([
                true,
                c.contains(&AuraDevTuf::Boot),
                c.contains(&AuraDevTuf::Awake),
                c.contains(&AuraDevTuf::Sleep),
                c.contains(&AuraDevTuf::Keyboard),
            ]);
        }

        if let Self::AuraDev1866(c) = control {
            return Some([
                true,
                c.contains(&AuraDev1866::Boot),
                c.contains(&AuraDev1866::Awake),
                c.contains(&AuraDev1866::Sleep),
                c.contains(&AuraDev1866::Keyboard),
            ]);
        }

        None
    }

    pub fn set_tuf(&mut self, power: AuraDevTuf, on: bool) {
        if let Self::AuraDevTuf(p) = self {
            if on {
                p.insert(power);
            } else {
                p.remove(&power);
            }
        }
    }

    pub fn set_0x1866(&mut self, power: AuraDev1866, on: bool) {
        if let Self::AuraDev1866(p) = self {
            if on {
                p.insert(power);
            } else {
                p.remove(&power);
            }
        }
    }

    pub fn set_0x19b6(&mut self, power: AuraDev19b6, on: bool) {
        if let Self::AuraDev19b6(p) = self {
            if on {
                p.insert(power);
            } else {
                p.remove(&power);
            }
        }
    }
}

impl From<&AuraPowerConfig> for AuraPowerDev {
    fn from(config: &AuraPowerConfig) -> Self {
        match config {
            AuraPowerConfig::AuraDevTuf(d) => AuraPowerDev {
                tuf: d.iter().copied().collect(),
                x1866: vec![],
                x19b6: vec![],
            },
            AuraPowerConfig::AuraDev1866(d) => AuraPowerDev {
                tuf: vec![],
                x1866: d.iter().copied().collect(),
                x19b6: vec![],
            },
            AuraPowerConfig::AuraDev19b6(d) => AuraPowerDev {
                tuf: vec![],
                x1866: vec![],
                x19b6: d.iter().copied().collect(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(default)]
pub struct AuraConfig {
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<BTreeMap<AuraModeNum, Vec<AuraEffect>>>,
    pub multizone_on: bool,
    pub enabled: AuraPowerConfig,
}

impl Default for AuraConfig {
    fn default() -> Self {
        let mut prod_id = AuraDevice::Unknown;
        for prod in &ASUS_KEYBOARD_DEVICES {
            if HidRaw::new(prod).is_ok() {
                prod_id = AuraDevice::from(*prod);
                break;
            }
        }

        if prod_id == AuraDevice::Unknown {
            if let Ok(p) = KeyboardLed::new() {
                if p.has_kbd_rgb_mode() {
                    prod_id = AuraDevice::Tuf;
                }
            }
        }

        let enabled = if prod_id == AuraDevice::X19B6 {
            AuraPowerConfig::AuraDev19b6(HashSet::from([
                AuraDev19b6::BootLogo,
                AuraDev19b6::BootKeyb,
                AuraDev19b6::SleepLogo,
                AuraDev19b6::SleepKeyb,
                AuraDev19b6::AwakeLogo,
                AuraDev19b6::AwakeKeyb,
                AuraDev19b6::ShutdownLogo,
                AuraDev19b6::ShutdownKeyb,
                AuraDev19b6::BootBar,
                AuraDev19b6::AwakeBar,
                AuraDev19b6::SleepBar,
                AuraDev19b6::ShutdownBar,
            ]))
        } else if prod_id == AuraDevice::Tuf {
            AuraPowerConfig::AuraDevTuf(HashSet::from([
                AuraDevTuf::Awake,
                AuraDevTuf::Boot,
                AuraDevTuf::Sleep,
                AuraDevTuf::Keyboard,
            ]))
        } else {
            AuraPowerConfig::AuraDev1866(HashSet::from([
                AuraDev1866::Awake,
                AuraDev1866::Boot,
                AuraDev1866::Sleep,
                AuraDev1866::Keyboard,
                AuraDev1866::Lightbar,
            ]))
        };

        AuraConfig {
            brightness: LedBrightness::Med,
            current_mode: AuraModeNum::Static,
            builtins: BTreeMap::new(),
            multizone: None,
            multizone_on: false,
            enabled,
        }
    }
}

impl StdConfig for AuraConfig {
    fn new() -> Self {
        Self::create_default(&LaptopLedData::get_data())
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }
}

impl StdConfigLoad for AuraConfig {}

impl AuraConfig {
    fn create_default(support_data: &LaptopLedData) -> Self {
        // create a default config here
        let mut config = AuraConfig::default();

        for n in &support_data.basic_modes {
            config
                .builtins
                .insert(*n, AuraEffect::default_with_mode(*n));

            if !support_data.basic_zones.is_empty() {
                let mut default = vec![];
                for (i, tmp) in support_data.basic_zones.iter().enumerate() {
                    default.push(AuraEffect {
                        mode: *n,
                        zone: *tmp,
                        colour1: *GRADIENT.get(i).unwrap_or(&GRADIENT[0]),
                        colour2: *GRADIENT.get(GRADIENT.len() - i).unwrap_or(&GRADIENT[6]),
                        speed: Speed::Med,
                        direction: Direction::Left,
                    });
                }
                if let Some(m) = config.multizone.as_mut() {
                    m.insert(*n, default);
                } else {
                    let mut tmp = BTreeMap::new();
                    tmp.insert(*n, default);
                    config.multizone = Some(tmp);
                }
            }
        }
        config
    }

    /// Set the mode data, current mode, and if multizone enabled.
    ///
    /// Multipurpose, will accept `AuraEffect` with zones and put in the correct
    /// store.
    pub fn set_builtin(&mut self, effect: AuraEffect) {
        self.current_mode = effect.mode;
        if effect.zone() == AuraZone::None {
            self.builtins.insert(*effect.mode(), effect);
            self.multizone_on = false;
        } else {
            if let Some(multi) = self.multizone.as_mut() {
                if let Some(fx) = multi.get_mut(effect.mode()) {
                    for fx in fx.iter_mut() {
                        if fx.zone == effect.zone {
                            *fx = effect;
                            return;
                        }
                    }
                    fx.push(effect);
                } else {
                    multi.insert(*effect.mode(), vec![effect]);
                }
            } else {
                let mut tmp = BTreeMap::new();
                tmp.insert(*effect.mode(), vec![effect]);
                self.multizone = Some(tmp);
            }
            self.multizone_on = true;
        }
    }

    pub fn get_multizone(&self, aura_type: AuraModeNum) -> Option<&[AuraEffect]> {
        if let Some(multi) = &self.multizone {
            return multi.get(&aura_type).map(|v| v.as_slice());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour};

    use super::AuraConfig;

    #[test]
    fn set_multizone_4key_config() {
        let mut config = AuraConfig::default();

        let effect = AuraEffect {
            colour1: Colour(0xff, 0x00, 0xff),
            zone: AuraZone::Key1,
            ..Default::default()
        };
        config.set_builtin(effect);

        assert!(config.multizone.is_some());

        let effect = AuraEffect {
            colour1: Colour(0x00, 0xff, 0xff),
            zone: AuraZone::Key2,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            colour1: Colour(0xff, 0xff, 0x00),
            zone: AuraZone::Key3,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            colour1: Colour(0x00, 0xff, 0x00),
            zone: AuraZone::Key4,
            ..Default::default()
        };
        let effect_clone = effect.clone();
        config.set_builtin(effect);
        // This should replace existing
        config.set_builtin(effect_clone);

        let res = config.multizone.unwrap();
        let sta = res.get(&AuraModeNum::Static).unwrap();
        assert_eq!(sta.len(), 4);
        assert_eq!(sta[0].colour1, Colour(0xff, 0x00, 0xff));
        assert_eq!(sta[1].colour1, Colour(0x00, 0xff, 0xff));
        assert_eq!(sta[2].colour1, Colour(0xff, 0xff, 0x00));
        assert_eq!(sta[3].colour1, Colour(0x00, 0xff, 0x00));
    }

    #[test]
    fn set_multizone_multimode_config() {
        let mut config = AuraConfig::default();

        let effect = AuraEffect {
            zone: AuraZone::Key1,
            ..Default::default()
        };
        config.set_builtin(effect);

        assert!(config.multizone.is_some());

        let effect = AuraEffect {
            zone: AuraZone::Key2,
            mode: AuraModeNum::Breathe,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            zone: AuraZone::Key3,
            mode: AuraModeNum::Comet,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            zone: AuraZone::Key4,
            mode: AuraModeNum::Pulse,
            ..Default::default()
        };
        config.set_builtin(effect);

        let res = config.multizone.unwrap();
        let sta = res.get(&AuraModeNum::Static).unwrap();
        assert_eq!(sta.len(), 1);

        let sta = res.get(&AuraModeNum::Breathe).unwrap();
        assert_eq!(sta.len(), 1);

        let sta = res.get(&AuraModeNum::Comet).unwrap();
        assert_eq!(sta.len(), 1);

        let sta = res.get(&AuraModeNum::Pulse).unwrap();
        assert_eq!(sta.len(), 1);
    }
}
