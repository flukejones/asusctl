use crate::laptops::LaptopLedData;
use log::{error, info, warn};
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, LedBrightness};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub static AURA_CONFIG_PATH: &str = "/etc/asusd/aura.conf";

#[derive(Deserialize, Serialize)]
pub struct AuraConfigV320 {
    pub brightness: u32,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<AuraMultiZone>,
}

impl AuraConfigV320 {
    pub(crate) fn into_current(self) -> AuraConfig {
        AuraConfig {
            brightness: <LedBrightness>::from(self.brightness),
            current_mode: self.current_mode,
            builtins: self.builtins,
            multizone: self.multizone,
            awake_enabled: true,
            sleep_anim_enabled: true,
            side_leds_enabled:true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AuraConfigV352 {
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<AuraMultiZone>,
}

impl AuraConfigV352 {
    pub(crate) fn into_current(self) -> AuraConfig {
        AuraConfig {
            brightness: self.brightness,
            current_mode: self.current_mode,
            builtins: self.builtins,
            multizone: self.multizone,
            awake_enabled: true,
            sleep_anim_enabled: true,
            side_leds_enabled:true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AuraConfig {
    pub brightness: LedBrightness,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<AuraMultiZone>,
    pub awake_enabled: bool,
    pub sleep_anim_enabled: bool,
    pub side_leds_enabled: bool,
}

impl Default for AuraConfig {
    fn default() -> Self {
        AuraConfig {
            brightness: LedBrightness::Med,
            current_mode: AuraModeNum::Static,
            builtins: BTreeMap::new(),
            multizone: None,
            awake_enabled: true,
            sleep_anim_enabled: true,
            side_leds_enabled:true,
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
                } else if let Ok(data) = serde_json::from_str::<AuraConfigV320>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated AuraConfig version");
                    return config;
                } else if let Ok(data) = serde_json::from_str::<AuraConfigV352>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated AuraConfig version");
                    return config;
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

    /// Multipurpose, will accept AuraEffect with zones and put in the correct store
    pub fn set_builtin(&mut self, effect: AuraEffect) {
        match effect.zone() {
            AuraZone::None => {
                self.builtins.insert(*effect.mode(), effect);
            }
            _ => {
                if let Some(multi) = self.multizone.as_mut() {
                    multi.set(effect)
                }
            }
        }
    }

    pub fn get_multizone(&self, aura_type: AuraModeNum) -> Option<&[AuraEffect; 4]> {
        if let Some(multi) = &self.multizone {
            if aura_type == AuraModeNum::Static {
                return Some(multi.static_());
            } else if aura_type == AuraModeNum::Breathe {
                return Some(multi.breathe());
            }
        }
        None
    }
}

#[derive(Deserialize, Serialize)]
pub struct AuraMultiZone {
    static_: [AuraEffect; 4],
    breathe: [AuraEffect; 4],
}

impl AuraMultiZone {
    pub fn set(&mut self, effect: AuraEffect) {
        if effect.mode == AuraModeNum::Static {
            match effect.zone {
                AuraZone::None => {}
                AuraZone::One => self.static_[0] = effect,
                AuraZone::Two => self.static_[1] = effect,
                AuraZone::Three => self.static_[2] = effect,
                AuraZone::Four => self.static_[3] = effect,
            }
        } else if effect.mode == AuraModeNum::Breathe {
            match effect.zone {
                AuraZone::None => {}
                AuraZone::One => self.breathe[0] = effect,
                AuraZone::Two => self.breathe[1] = effect,
                AuraZone::Three => self.breathe[2] = effect,
                AuraZone::Four => self.breathe[3] = effect,
            }
        }
    }

    pub fn static_(&self) -> &[AuraEffect; 4] {
        &self.static_
    }

    pub fn breathe(&self) -> &[AuraEffect; 4] {
        &self.breathe
    }
}

impl Default for AuraMultiZone {
    fn default() -> Self {
        Self {
            static_: [
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::One,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Two,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Three,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Four,
                    ..Default::default()
                },
            ],
            breathe: [
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::One,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Two,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Three,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Four,
                    ..Default::default()
                },
            ],
        }
    }
}
