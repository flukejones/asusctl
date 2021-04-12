use log::{error, warn};
use rog_anime::{error::AnimeError, ActionData, AnimTime, AnimeAction, Vec2};
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::Duration;

pub static ANIME_CONFIG_PATH: &str = "/etc/asusd/anime.conf";
pub static ANIME_CACHE_PATH: &str = "/etc/asusd/anime-cache.conf";

#[derive(Deserialize, Serialize, Default)]
pub struct AnimeConfigCached {
    pub system: Option<ActionData>,
    pub boot: Option<ActionData>,
    pub suspend: Option<ActionData>,
    pub shutdown: Option<ActionData>,
}

impl AnimeConfigCached {
    pub fn init_from_config(&mut self, config: &AnimeConfig) -> Result<(), AnimeError> {
        if let Some(ref sys) = config.system {
            self.system = Some(ActionData::from_anime_action(sys)?)
        }
        if let Some(ref boot) = config.boot {
            self.boot = Some(ActionData::from_anime_action(boot)?)
        }
        if let Some(ref suspend) = config.boot {
            self.suspend = Some(ActionData::from_anime_action(suspend)?)
        }
        if let Some(ref shutdown) = config.boot {
            self.shutdown = Some(ActionData::from_anime_action(shutdown)?)
        }
        Ok(())
    }
}

/// Config for base system actions for the anime display
#[derive(Deserialize, Serialize)]
pub struct AnimeConfig {
    pub system: Option<AnimeAction>,
    pub boot: Option<AnimeAction>,
    pub suspend: Option<AnimeAction>,
    pub shutdown: Option<AnimeAction>,
}

impl Default for AnimeConfig {
    fn default() -> Self {
        AnimeConfig {
            system: None,
            boot: None,
            suspend: None,
            shutdown: None,
        }
    }
}

impl AnimeConfig {
    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load() -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&ANIME_CONFIG_PATH)
            .unwrap_or_else(|_| {
                panic!(
                    "The file {} or directory /etc/asusd/ is missing",
                    ANIME_CONFIG_PATH
                )
            }); // okay to cause panic here
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                return AnimeConfig::create_default(&mut file);
            } else {
                if let Ok(data) = serde_json::from_str(&buf) {
                    return data;
                }
                warn!("Could not deserialise {}", ANIME_CONFIG_PATH);
                panic!("Please remove {} then restart asusd", ANIME_CONFIG_PATH);
            }
        }
        AnimeConfig::create_default(&mut file)
    }

    fn create_default(file: &mut File) -> Self {
        // create a default config here
        let config = AnimeConfig {
            system: None,
            boot: Some(AnimeAction::ImageAnimation {
                file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                scale: 0.9,
                angle: 0.65,
                translation: Vec2::default(),
                brightness: 0.5,
                time: AnimTime::Time(Duration::from_secs(5)),
            }),
            suspend: None,
            shutdown: None,
        };
        // Should be okay to unwrap this as is since it is a Default
        let json = serde_json::to_string(&config).unwrap();
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write {}", ANIME_CONFIG_PATH));
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&ANIME_CONFIG_PATH)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", ANIME_CONFIG_PATH, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", ANIME_CONFIG_PATH);
            } else {
                let x: AnimeConfig = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", ANIME_CONFIG_PATH));
                *self = x;
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(ANIME_CONFIG_PATH).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }
}
