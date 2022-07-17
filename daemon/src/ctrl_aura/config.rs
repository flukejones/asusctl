use crate::laptops::LaptopLedData;
use log::{error, warn};
use rog_aura::usb::AuraControl;
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Direction, LedBrightness, Speed, GRADIENT};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub static AURA_CONFIG_PATH: &str = "/etc/asusd/aura.conf";

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AuraConfig {
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<BTreeMap<AuraModeNum, Vec<AuraEffect>>>,
    pub multizone_on: bool,
    pub enabled: HashSet<AuraControl>,
}

impl Default for AuraConfig {
    fn default() -> Self {
        AuraConfig {
            brightness: LedBrightness::Med,
            current_mode: AuraModeNum::Static,
            builtins: BTreeMap::new(),
            multizone: None,
            multizone_on: false,
            enabled: HashSet::from([
                AuraControl::BootLogo,
                AuraControl::BootKeyb,
                AuraControl::SleepLogo,
                AuraControl::SleepKeyb,
                AuraControl::AwakeLogo,
                AuraControl::AwakeKeyb,
                AuraControl::ShutdownLogo,
                AuraControl::ShutdownKeyb,
                AuraControl::AwakeBar,
                AuraControl::BootBar,
                AuraControl::SleepBar,
                AuraControl::ShutdownBar,
            ]),
        }
    }
}

impl AuraConfig {
    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load(supported_led_modes: &LaptopLedData) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&AURA_CONFIG_PATH)
            .unwrap_or_else(|_| {
                panic!(
                    "The file {} or directory /etc/asusd/ is missing",
                    AURA_CONFIG_PATH
                )
            }); // okay to cause panic here
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                return AuraConfig::create_default(&mut file, supported_led_modes);
            } else {
                if let Ok(data) = serde_json::from_str(&buf) {
                    return data;
                }
                warn!(
                    "Could not deserialise {}.\nWill rename to {}-old and recreate config",
                    AURA_CONFIG_PATH, AURA_CONFIG_PATH
                );
                let cfg_old = AURA_CONFIG_PATH.to_string() + "-old";
                std::fs::rename(AURA_CONFIG_PATH, cfg_old).unwrap_or_else(|err| {
                    panic!(
                        "Could not rename. Please remove {} then restart service: Error {}",
                        AURA_CONFIG_PATH, err
                    )
                });
            }
        }
        AuraConfig::create_default(&mut file, supported_led_modes)
    }

    fn create_default(file: &mut File, support_data: &LaptopLedData) -> Self {
        // create a default config here
        let mut config = AuraConfig::default();

        for n in &support_data.standard {
            config
                .builtins
                .insert(*n, AuraEffect::default_with_mode(*n));

            if !support_data.multizone.is_empty() {
                let mut default = vec![];
                for (i, tmp) in support_data.multizone.iter().enumerate() {
                    default.push(AuraEffect {
                        mode: *n,
                        zone: *tmp,
                        colour1: *GRADIENT.get(i).unwrap_or(&GRADIENT[0]),
                        colour2: *GRADIENT.get(GRADIENT.len() - i).unwrap_or(&GRADIENT[6]),
                        speed: Speed::Med,
                        direction: Direction::Left,
                    })
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

        // Should be okay to unwrap this as is since it is a Default
        let json = serde_json::to_string(&config).unwrap();
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write {}", AURA_CONFIG_PATH));
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&AURA_CONFIG_PATH)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", AURA_CONFIG_PATH, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", AURA_CONFIG_PATH);
            } else {
                let x: AuraConfig = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", AURA_CONFIG_PATH));
                *self = x;
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(AURA_CONFIG_PATH).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }

    /// Set the mode data, current mode, and if multizone enabled.
    ///
    /// Multipurpose, will accept AuraEffect with zones and put in the correct store.
    pub fn set_builtin(&mut self, effect: AuraEffect) {
        self.current_mode = effect.mode;
        match effect.zone() {
            AuraZone::None => {
                self.builtins.insert(*effect.mode(), effect);
                self.multizone_on = false;
            }
            _ => {
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
    use super::AuraConfig;
    use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour};

    #[test]
    fn set_multizone_4key_config() {
        let mut config = AuraConfig::default();

        let mut effect = AuraEffect::default();
        effect.colour1 = Colour(0xff, 0x00, 0xff);
        effect.zone = AuraZone::Key1;
        config.set_builtin(effect);

        assert!(config.multizone.is_some());

        let mut effect = AuraEffect::default();
        effect.colour1 = Colour(0x00, 0xff, 0xff);
        effect.zone = AuraZone::Key2;
        config.set_builtin(effect);

        let mut effect = AuraEffect::default();
        effect.colour1 = Colour(0xff, 0xff, 0x00);
        effect.zone = AuraZone::Key3;
        config.set_builtin(effect);

        let mut effect = AuraEffect::default();
        effect.colour1 = Colour(0x00, 0xff, 0x00);
        effect.zone = AuraZone::Key4;
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

        let mut effect = AuraEffect::default();
        effect.zone = AuraZone::Key1;
        config.set_builtin(effect);

        assert!(config.multizone.is_some());

        let mut effect = AuraEffect::default();
        effect.zone = AuraZone::Key2;
        effect.mode = AuraModeNum::Breathe;
        config.set_builtin(effect);

        let mut effect = AuraEffect::default();
        effect.zone = AuraZone::Key3;
        effect.mode = AuraModeNum::Comet;
        config.set_builtin(effect);

        let mut effect = AuraEffect::default();
        effect.zone = AuraZone::Key4;
        effect.mode = AuraModeNum::Pulse;
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
