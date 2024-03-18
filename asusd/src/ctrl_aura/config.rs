use std::collections::{BTreeMap, HashSet};

use config_traits::{StdConfig, StdConfigLoad};
use log::{debug, info};
use rog_aura::aura_detection::LaptopLedData;
use rog_aura::power::AuraPower;
use rog_aura::usb::{AuraDevRog1, AuraDevTuf, AuraDevice, AuraPowerDev};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Direction, LedBrightness, Speed, GRADIENT};
use serde_derive::{Deserialize, Serialize};

/// Enable/disable LED control in various states such as
/// when the device is awake, suspended, shutting down or
/// booting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuraPowerConfig {
    AuraDevTuf(HashSet<AuraDevTuf>),
    AuraDevRog1(HashSet<AuraDevRog1>),
    AuraDevRog2(AuraPower),
}

impl AuraPowerConfig {
    /// Invalid for TUF laptops
    pub fn to_bytes(control: &Self) -> [u8; 4] {
        match control {
            AuraPowerConfig::AuraDevTuf(_) => [0, 0, 0, 0],
            AuraPowerConfig::AuraDevRog1(c) => {
                let c: Vec<AuraDevRog1> = c.iter().copied().collect();
                AuraDevRog1::to_bytes(&c)
            }
            AuraPowerConfig::AuraDevRog2(c) => c.to_bytes(),
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

        if let Self::AuraDevRog1(c) = control {
            return Some([
                true,
                c.contains(&AuraDevRog1::Boot),
                c.contains(&AuraDevRog1::Awake),
                c.contains(&AuraDevRog1::Sleep),
                c.contains(&AuraDevRog1::Keyboard),
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

    pub fn set_0x1866(&mut self, power: AuraDevRog1, on: bool) {
        if let Self::AuraDevRog1(p) = self {
            if on {
                p.insert(power);
            } else {
                p.remove(&power);
            }
        }
    }

    pub fn set_0x19b6(&mut self, power: AuraPower) {
        if let Self::AuraDevRog2(p) = self {
            *p = power;
        }
    }
}

impl From<&AuraPowerConfig> for AuraPowerDev {
    fn from(config: &AuraPowerConfig) -> Self {
        match config {
            AuraPowerConfig::AuraDevTuf(d) => AuraPowerDev {
                tuf: d.iter().copied().collect(),
                ..Default::default()
            },
            AuraPowerConfig::AuraDevRog1(d) => AuraPowerDev {
                old_rog: d.iter().copied().collect(),
                ..Default::default()
            },
            AuraPowerConfig::AuraDevRog2(d) => AuraPowerDev {
                rog: d.clone(),
                ..Default::default()
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
// #[serde(default)]
pub struct AuraConfig {
    pub config_name: String,
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<BTreeMap<AuraModeNum, Vec<AuraEffect>>>,
    pub multizone_on: bool,
    pub enabled: AuraPowerConfig,
}

impl AuraConfig {
    /// Detect the keyboard type and load from default DB if data available
    pub fn new_with(prod_id: AuraDevice) -> Self {
        info!("creating new AuraConfig");
        Self::from_default_support(prod_id, &LaptopLedData::get_data())
    }
}

impl StdConfig for AuraConfig {
    /// Detect the keyboard type and load from default DB if data available
    fn new() -> Self {
        panic!("This should not be used");
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        if self.config_name.is_empty() {
            panic!("Config file name should not be empty");
        }
        self.config_name.to_owned()
    }
}

impl StdConfigLoad for AuraConfig {}

impl AuraConfig {
    pub fn set_filename(&mut self, prod_id: AuraDevice) {
        self.config_name = format!("aura_{prod_id:?}.ron");
    }

    pub fn from_default_support(prod_id: AuraDevice, support_data: &LaptopLedData) -> Self {
        // create a default config here
        let enabled = if prod_id.is_new_style() {
            AuraPowerConfig::AuraDevRog2(AuraPower::new_all_on())
        } else if prod_id.is_tuf_style() {
            AuraPowerConfig::AuraDevTuf(HashSet::from([
                AuraDevTuf::Awake,
                AuraDevTuf::Boot,
                AuraDevTuf::Sleep,
                AuraDevTuf::Keyboard,
            ]))
        } else {
            AuraPowerConfig::AuraDevRog1(HashSet::from([
                AuraDevRog1::Awake,
                AuraDevRog1::Boot,
                AuraDevRog1::Sleep,
                AuraDevRog1::Keyboard,
                AuraDevRog1::Lightbar,
            ]))
        };
        let mut config = AuraConfig {
            config_name: format!("aura_{prod_id:?}.ron"),
            brightness: LedBrightness::Med,
            current_mode: AuraModeNum::Static,
            builtins: BTreeMap::new(),
            multizone: None,
            multizone_on: false,
            enabled,
        };

        for n in &support_data.basic_modes {
            debug!("creating default for {n}");
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
    use rog_aura::aura_detection::LaptopLedData;
    use rog_aura::usb::AuraDevice;
    use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour};

    use super::AuraConfig;

    #[test]
    fn set_multizone_4key_config() {
        let mut config =
            AuraConfig::from_default_support(AuraDevice::X19b6, &LaptopLedData::default());

        let effect = AuraEffect {
            colour1: Colour {
                r: 0xff,
                g: 0x00,
                b: 0xff,
            },
            zone: AuraZone::Key1,
            ..Default::default()
        };
        config.set_builtin(effect);

        assert!(config.multizone.is_some());

        let effect = AuraEffect {
            colour1: Colour {
                r: 0x00,
                g: 0xff,
                b: 0xff,
            },
            zone: AuraZone::Key2,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            colour1: Colour {
                r: 0xff,
                g: 0xff,
                b: 0x00,
            },
            zone: AuraZone::Key3,
            ..Default::default()
        };
        config.set_builtin(effect);

        let effect = AuraEffect {
            colour1: Colour {
                r: 0x00,
                g: 0xff,
                b: 0x00,
            },
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
        assert_eq!(
            sta[0].colour1,
            Colour {
                r: 0xff,
                g: 0x00,
                b: 0xff
            }
        );
        assert_eq!(
            sta[1].colour1,
            Colour {
                r: 0x00,
                g: 0xff,
                b: 0xff
            }
        );
        assert_eq!(
            sta[2].colour1,
            Colour {
                r: 0xff,
                g: 0xff,
                b: 0x00
            }
        );
        assert_eq!(
            sta[3].colour1,
            Colour {
                r: 0x00,
                g: 0xff,
                b: 0x00
            }
        );
    }

    #[test]
    fn set_multizone_multimode_config() {
        let mut config =
            AuraConfig::from_default_support(AuraDevice::X19b6, &LaptopLedData::default());

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
