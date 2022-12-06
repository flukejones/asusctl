use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use std::{
    fs::{create_dir, OpenOptions},
    io::{Read, Write},
};
//use log::{error, info, warn};

use crate::{error::Error, update_and_notify::EnabledNotifications};

const CFG_DIR: &str = "rog";
const CFG_FILE_NAME: &str = "rog-control-center.cfg";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub run_in_background: bool,
    pub startup_in_background: bool,
    pub enable_notifications: bool,
    pub enabled_notifications: EnabledNotifications,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            run_in_background: true,
            startup_in_background: false,
            enable_notifications: true,
            enabled_notifications: EnabledNotifications::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            info!("Found XDG config dir {dir:?}");
            dir
        } else {
            error!("Could not get XDG config dir");
            return Err(Error::XdgVars);
        };

        path.push(CFG_DIR);
        if !path.exists() {
            create_dir(path.clone())?;
            info!("Created {path:?}");
        }

        path.push(CFG_FILE_NAME);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let mut buf = String::new();

        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                warn!("Zero len read of Config file");
                let default = Config::default();
                let t = toml::to_string_pretty(&default).unwrap();
                file.write_all(t.as_bytes())?;
                return Ok(default);
            } else if let Ok(data) = toml::from_str::<Config>(&buf) {
                info!("Loaded config file {path:?}");
                return Ok(data);
            }
        }
        Err(Error::ConfigLoadFail)
    }

    pub fn save(&mut self, enabled_notifications: &EnabledNotifications) -> Result<(), Error> {
        let mut path = if let Some(dir) = dirs::config_dir() {
            dir
        } else {
            return Err(Error::XdgVars);
        };

        path.push(CFG_DIR);
        if !path.exists() {
            create_dir(path.clone())?;
            info!("Created {path:?}");
        }

        path.push(CFG_FILE_NAME);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        self.enabled_notifications = enabled_notifications.clone();
        let t = toml::to_string_pretty(&self).unwrap();
        file.write_all(t.as_bytes())?;
        info!("Saved config file {path:?}");
        Ok(())
    }
}
