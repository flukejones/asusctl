use log::{error, warn};
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub static CONFIG_PATH: &str = "/etc/asusd/asusd.conf";

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
}

impl Config {
    fn new() -> Self {
        Config {
            bat_charge_limit: 100,
        }
    }

    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load() -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&CONFIG_PATH)
            .unwrap_or_else(|_| panic!("The directory /etc/asusd/ is missing")); // okay to cause panic here
        let mut buf = String::new();
        let config;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = Self::new();
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
            } else {
                warn!(
                    "Could not deserialise {}.\nWill rename to {}-old and recreate config",
                    CONFIG_PATH, CONFIG_PATH
                );
                let cfg_old = CONFIG_PATH.to_string() + "-old";
                std::fs::rename(CONFIG_PATH, cfg_old).unwrap_or_else(|err| {
                    panic!(
                        "Could not rename. Please remove {} then restart service: Error {}",
                        CONFIG_PATH, err
                    )
                });
                config = Self::new();
            }
        } else {
            config = Self::new()
        }
        config.write();
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&CONFIG_PATH)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", CONFIG_PATH, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", CONFIG_PATH);
            } else {
                *self = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_PATH));
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(CONFIG_PATH).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }
}
