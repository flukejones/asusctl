use log::{error, warn};
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::gfx_vendors::GfxVendors;

pub static CONFIG_PATH: &str = "/etc/asusd/asusd.conf";
pub static AURA_CONFIG_PATH: &str = "/etc/asusd/asusd.conf";

#[derive(Deserialize, Serialize)]
pub struct GfxConfig {
    #[serde(skip)]
    config_path: String,
    /// The current mode set, also applies on boot
    pub gfx_mode: GfxVendors,
    /// Only for informational purposes
    #[serde(skip)]
    pub gfx_tmp_mode: Option<GfxVendors>,
    /// Set if graphics management is enabled
    pub gfx_managed: bool,
    /// Set if vfio option is enabled. This requires the vfio drivers to be built as modules
    pub gfx_vfio_enable: bool,
}

impl GfxConfig {
    fn new(config_path: String) -> Self {
        Self {
            config_path,
            gfx_mode: GfxVendors::Hybrid,
            gfx_tmp_mode: None,
            gfx_managed: true,
            gfx_vfio_enable: false,
        }
    }

    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load(config_path: String) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config_path)
            .unwrap_or_else(|_| panic!("The directory {} is missing", config_path)); // okay to cause panic here
        let mut buf = String::new();
        let mut config;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = Self::new(config_path);
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
                config.config_path = config_path;
            } else {
                warn!("Could not deserialise {}", config_path);
                panic!("Please remove {} then restart service", config_path);
            }
        } else {
            config = Self::new(config_path)
        }
        config.write();
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.config_path)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", self.config_path, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", self.config_path);
            } else {
                let mut x: Self = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", self.config_path));
                // copy over serde skipped values
                x.gfx_tmp_mode = self.gfx_tmp_mode;
                *self = x;
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(&self.config_path).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }
}
