use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use log::{error, warn};
use serde_derive::{Deserialize, Serialize};

use crate::config_file_open;

pub static CONFIG_FILE: &str = "/etc/asusd/asusd.conf";

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
}

impl Config {
    fn new() -> Self {
        Config {
            bat_charge_limit: 100,
            panel_od: false,
            disable_nvidia_powerd_on_battery: true,
            ac_command: String::new(),
            bat_command: String::new(),
        }
    }

    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load() -> Self {
        let mut file = config_file_open(CONFIG_FILE);
        let mut buf = String::new();
        let config;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = Self::new();
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
            } else if let Ok(data) = serde_json::from_str::<Config455>(&buf) {
                config = data.into();
            } else if let Ok(data) = serde_json::from_str::<Config458>(&buf) {
                config = data.into();
            } else {
                warn!(
                    "Could not deserialise {}.\nWill rename to {}-old and recreate config",
                    CONFIG_FILE, CONFIG_FILE
                );
                let cfg_old = CONFIG_FILE.to_owned() + "-old";
                std::fs::rename(CONFIG_FILE, cfg_old).unwrap_or_else(|err| {
                    panic!(
                        "Could not rename. Please remove {} then restart service: Error {}",
                        CONFIG_FILE, err
                    )
                });
                config = Self::new();
            }
        } else {
            config = Self::new();
        }
        config.write();
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(CONFIG_FILE)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", CONFIG_FILE, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", CONFIG_FILE);
            } else {
                *self = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_FILE));
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(CONFIG_FILE).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config455 {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
}

impl From<Config455> for Config {
    fn from(c: Config455) -> Self {
        Self {
            bat_charge_limit: c.bat_charge_limit,
            panel_od: c.panel_od,
            disable_nvidia_powerd_on_battery: true,
            ac_command: String::new(),
            bat_command: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Config458 {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
    pub ac_command: String,
    pub bat_command: String,
}

impl From<Config458> for Config {
    fn from(c: Config458) -> Self {
        Self {
            bat_charge_limit: c.bat_charge_limit,
            panel_od: c.panel_od,
            disable_nvidia_powerd_on_battery: true,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
        }
    }
}
