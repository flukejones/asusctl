use log::{error, warn};
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurves, Profile};
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize, Debug)]
pub struct ProfileConfig {
    #[serde(skip)]
    config_path: String,
    /// For restore on boot
    pub active: Profile,
    /// States to restore
    pub fan_curves: Option<FanCurves>,
}

impl ProfileConfig {
    fn new(config_path: String) -> Result<Self, ProfileError> {
        let mut platform = ProfileConfig {
            config_path,
            active: Profile::Balanced,
            fan_curves: None,
        };

        if !Profile::is_platform_profile_supported() {
            return Err(ProfileError::NotSupported);
        }

        if FanCurves::is_fan_curves_supported() {
            let mut curves = FanCurves::default();
            curves.update_from_platform();
            platform.fan_curves = Some(curves);
        }
        Ok(platform)
    }
}

impl ProfileConfig {
    pub fn load(config_path: String) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config_path)
            .unwrap_or_else(|_| panic!("The directory /etc/asusd/ is missing")); // okay to cause panic here
        let mut buf = String::new();
        let mut config;
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                config = Self::new(config_path).unwrap();
            } else if let Ok(data) = serde_json::from_str(&buf) {
                config = data;
                config.config_path = config_path;
            } else {
                warn!("Could not deserialise {}", config_path);
                panic!("Please remove {} then restart service", config_path);
            }
        } else {
            config = Self::new(config_path).unwrap()
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
                let mut data: ProfileConfig = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", self.config_path));
                // copy over serde skipped values
                data.config_path = self.config_path.clone();
                *self = data;
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
