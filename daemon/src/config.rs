use log::{error, info, warn};
use rog_types::gfx_vendors::GfxVendors;
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::config_old::*;
use crate::VERSION;

pub static CONFIG_PATH: &str = "/etc/asusd/asusd.conf";
pub static AURA_CONFIG_PATH: &str = "/etc/asusd/asusd.conf";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub gfx_mode: GfxVendors,
    /// Only for informational purposes.
    #[serde(skip)]
    pub gfx_tmp_mode: Option<GfxVendors>,
    pub gfx_managed: bool,
    pub gfx_vfio_enable: bool,
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: true,
            gfx_vfio_enable: false,
            bat_charge_limit: 100,
        }
    }
}

impl Config {
    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load() -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&CONFIG_PATH)
            .unwrap_or_else(|_| panic!("The directory /etc/asusd/ is missing")); // okay to cause panic here
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                return Config::create_default(&mut file);
            } else {
                if let Ok(data) = serde_json::from_str(&buf) {
                    return data;
                } else if let Ok(data) = serde_json::from_str::<ConfigV352>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                } else if let Ok(data) = serde_json::from_str::<ConfigV341>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                } else if let Ok(data) = serde_json::from_str::<ConfigV324>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                } else if let Ok(data) = serde_json::from_str::<ConfigV317>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                }
                warn!("Could not deserialise {}", CONFIG_PATH);
                panic!("Please remove {} then restart asusd", CONFIG_PATH);
            }
        }
        Config::create_default(&mut file)
    }

    fn create_default(file: &mut File) -> Self {
        let config = Config::default();

        // Should be okay to unwrap this as is since it is a Default
        let json = serde_json::to_string_pretty(&config).unwrap();
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write {}", CONFIG_PATH));
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
                let mut x: Config = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", CONFIG_PATH));
                // copy over serde skipped values
                x.gfx_tmp_mode = self.gfx_tmp_mode;
                *self = x;
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
