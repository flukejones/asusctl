use std::collections::BTreeMap;

use config_traits::{StdConfig, StdConfigLoad};
use log::{debug, info, warn};
use rog_aura::aura_detection::LedSupportData;
use rog_aura::keyboard::LaptopAuraPower;
use rog_aura::{
    AuraDeviceType, AuraEffect, AuraModeNum, AuraZone, Direction, LedBrightness, Speed, GRADIENT,
};
use serde::{Deserialize, Serialize};

use crate::error::RogError;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
// #[serde(default)]
pub struct AuraConfig {
    pub config_name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ally_fix: Option<bool>,
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub multizone: Option<BTreeMap<AuraModeNum, Vec<AuraEffect>>>,
    pub multizone_on: bool,
    pub enabled: LaptopAuraPower,
}

impl StdConfig for AuraConfig {
    /// Detect the keyboard type and load from default DB if data available
    fn new() -> Self {
        panic!("This should not be used");
    }

    fn file_name(&self) -> String {
        if self.config_name.is_empty() {
            panic!("Config file name should not be empty");
        }
        self.config_name.to_owned()
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }
}

impl StdConfigLoad for AuraConfig {}

impl AuraConfig {
    /// Detect the keyboard type and load from default DB if data available
    pub fn new(prod_id: &str) -> Self {
        info!("Setting up AuraConfig for {prod_id:?}");
        // create a default config here
        let device_type = AuraDeviceType::from(prod_id);
        if device_type == AuraDeviceType::Unknown {
            warn!("idProduct:{prod_id:?} is unknown");
        }
        let support_data = LedSupportData::get_data(prod_id);
        let enabled = LaptopAuraPower::new(device_type, &support_data);
        let mut config = AuraConfig {
            config_name: format!("aura_{prod_id}.ron"),
            ally_fix: None,
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

    /// Create a default for the `current_mode` if multizone and no config
    /// exists.
    pub(super) fn create_multizone_default(
        &mut self,
        supported_data: &LedSupportData,
    ) -> Result<(), RogError> {
        let mut default = vec![];
        for (i, tmp) in supported_data.basic_zones.iter().enumerate() {
            default.push(AuraEffect {
                mode: self.current_mode,
                zone: *tmp,
                colour1: *GRADIENT.get(i).unwrap_or(&GRADIENT[0]),
                colour2: *GRADIENT.get(GRADIENT.len() - i).unwrap_or(&GRADIENT[6]),
                speed: Speed::Med,
                direction: Direction::Left,
            });
        }
        if default.is_empty() {
            return Err(RogError::AuraEffectNotSupported);
        }

        if let Some(multizones) = self.multizone.as_mut() {
            multizones.insert(self.current_mode, default);
        } else {
            let mut tmp = BTreeMap::new();
            tmp.insert(self.current_mode, default);
            self.multizone = Some(tmp);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rog_aura::keyboard::AuraPowerState;
    use rog_aura::{
        AuraEffect, AuraModeNum, AuraZone, Colour, Direction, LedBrightness, PowerZones, Speed,
    };

    use super::AuraConfig;

    #[test]
    fn set_multizone_4key_config() {
        std::env::set_var("BOARD_NAME", "");
        let mut config = AuraConfig::new("19b6");

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
        std::env::set_var("BOARD_NAME", "");
        let mut config = AuraConfig::new("19b6");

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

    #[test]
    fn verify_0x1866_g531i() {
        std::env::set_var("BOARD_NAME", "G513I");
        let mut config = AuraConfig::new("1866");

        assert_eq!(config.brightness, LedBrightness::Med);
        assert_eq!(config.builtins.len(), 5);
        assert_eq!(
            config.builtins.first_entry().unwrap().get(),
            &AuraEffect {
                mode: AuraModeNum::Static,
                zone: AuraZone::None,
                colour1: Colour { r: 166, g: 0, b: 0 },
                colour2: Colour { r: 0, g: 0, b: 0 },
                speed: Speed::Med,
                direction: Direction::Right
            }
        );
        assert_eq!(config.enabled.states.len(), 1);
        assert_eq!(
            config.enabled.states[0],
            AuraPowerState {
                zone: PowerZones::KeyboardAndLightbar,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true
            }
        );
    }

    #[test]
    fn verify_0x19b6_g634j() {
        std::env::set_var("BOARD_NAME", "G634J");
        let mut config = AuraConfig::new("19b6");

        assert_eq!(config.brightness, LedBrightness::Med);
        assert_eq!(config.builtins.len(), 12);
        assert_eq!(
            config.builtins.first_entry().unwrap().get(),
            &AuraEffect {
                mode: AuraModeNum::Static,
                zone: AuraZone::None,
                colour1: Colour { r: 166, g: 0, b: 0 },
                colour2: Colour { r: 0, g: 0, b: 0 },
                speed: Speed::Med,
                direction: Direction::Right
            }
        );
        assert_eq!(config.enabled.states.len(), 4);
        assert_eq!(
            config.enabled.states[0],
            AuraPowerState {
                zone: PowerZones::Keyboard,
                boot: true,
                awake: true,
                sleep: true,
                shutdown: true
            }
        );
    }
}
