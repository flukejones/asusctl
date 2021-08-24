use crate::VERSION;
use log::{error, info, warn};
use rog_anime::Fade;
use rog_anime::{error::AnimeError, ActionData, ActionLoader, AnimTime, Vec2};
use serde_derive::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::Duration;

pub static ANIME_CONFIG_PATH: &str = "/etc/asusd/anime.conf";
pub static ANIME_CACHE_PATH: &str = "/etc/asusd/anime-cache.conf";

#[derive(Deserialize, Serialize)]
pub struct AnimeConfigV341 {
    pub system: Option<ActionLoader>,
    pub boot: Option<ActionLoader>,
    pub suspend: Option<ActionLoader>,
    pub shutdown: Option<ActionLoader>,
}

impl AnimeConfigV341 {
    pub(crate) fn into_current(self) -> AnimeConfig {
        AnimeConfig {
            system: if let Some(ani) = self.system {
                vec![ani]
            } else {
                vec![]
            },
            boot: if let Some(ani) = self.boot {
                vec![ani]
            } else {
                vec![]
            },
            wake: if let Some(ani) = self.suspend {
                vec![ani]
            } else {
                vec![]
            },
            shutdown: if let Some(ani) = self.shutdown {
                vec![ani]
            } else {
                vec![]
            },
            brightness: 1.0,
            awake_enabled: true,
            boot_anim_enabled: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AnimeConfigV352 {
    pub system: Vec<ActionLoader>,
    pub boot: Vec<ActionLoader>,
    pub wake: Vec<ActionLoader>,
    pub shutdown: Vec<ActionLoader>,
    pub brightness: f32,
}

impl AnimeConfigV352 {
    pub(crate) fn into_current(self) -> AnimeConfig {
        AnimeConfig {
            system: self.system,
            boot: self.boot,
            wake: self.wake,
            shutdown: self.shutdown,
            brightness: 1.0,
            awake_enabled: true,
            boot_anim_enabled: true,
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct AnimeConfigCached {
    pub system: Vec<ActionData>,
    pub boot: Vec<ActionData>,
    pub wake: Vec<ActionData>,
    pub shutdown: Vec<ActionData>,
}

impl AnimeConfigCached {
    pub fn init_from_config(&mut self, config: &AnimeConfig) -> Result<(), AnimeError> {
        let mut sys = Vec::with_capacity(config.system.len());
        for ani in config.system.iter() {
            sys.push(ActionData::from_anime_action(ani)?);
        }
        self.system = sys;

        let mut boot = Vec::with_capacity(config.boot.len());
        for ani in config.boot.iter() {
            boot.push(ActionData::from_anime_action(ani)?);
        }
        self.boot = boot;

        let mut wake = Vec::with_capacity(config.wake.len());
        for ani in config.wake.iter() {
            wake.push(ActionData::from_anime_action(ani)?);
        }
        self.wake = wake;

        let mut shutdown = Vec::with_capacity(config.shutdown.len());
        for ani in config.shutdown.iter() {
            shutdown.push(ActionData::from_anime_action(ani)?);
        }
        self.shutdown = shutdown;
        Ok(())
    }
}

/// Config for base system actions for the anime display
#[derive(Deserialize, Serialize)]
pub struct AnimeConfig {
    pub system: Vec<ActionLoader>,
    pub boot: Vec<ActionLoader>,
    pub wake: Vec<ActionLoader>,
    pub shutdown: Vec<ActionLoader>,
    pub brightness: f32,
    pub awake_enabled: bool,
    pub boot_anim_enabled: bool,
}

impl Default for AnimeConfig {
    fn default() -> Self {
        AnimeConfig {
            system: Vec::new(),
            boot: Vec::new(),
            wake: Vec::new(),
            shutdown: Vec::new(),
            brightness: 1.0,
            awake_enabled: true,
            boot_anim_enabled: true,
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
                } else if let Ok(data) = serde_json::from_str::<AnimeConfigV341>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                } else if let Ok(data) = serde_json::from_str::<AnimeConfigV352>(&buf) {
                    let config = data.into_current();
                    config.write();
                    info!("Updated config version to: {}", VERSION);
                    return config;
                }
                AnimeConfig::write_backup(buf);
                warn!(
                    "Could not deserialise {}. Backed up as *-old",
                    ANIME_CONFIG_PATH
                );
            }
        }
        AnimeConfig::create_default(&mut file)
    }

    fn create_default(file: &mut File) -> Self {
        // create a default config here
        let config = AnimeConfig {
            system: vec![],
            boot: vec![ActionLoader::ImageAnimation {
                file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                scale: 0.9,
                angle: 0.65,
                translation: Vec2::default(),
                brightness: 1.0,
                time: AnimTime::Fade(Fade::new(
                    Duration::from_secs(2),
                    Some(Duration::from_secs(2)),
                    Duration::from_secs(2),
                )),
            }],
            wake: vec![ActionLoader::ImageAnimation {
                file: "/usr/share/asusd/anime/custom/sonic-run.gif".into(),
                scale: 0.9,
                angle: 0.65,
                translation: Vec2::default(),
                brightness: 1.0,
                time: AnimTime::Fade(Fade::new(
                    Duration::from_secs(2),
                    Some(Duration::from_secs(2)),
                    Duration::from_secs(2),
                )),
            }],
            shutdown: vec![ActionLoader::ImageAnimation {
                file: "/usr/share/asusd/anime/custom/sonic-wait.gif".into(),
                scale: 0.9,
                angle: 0.0,
                translation: Vec2::new(3.0, 2.0),
                brightness: 1.0,
                time: AnimTime::Infinite,
            }],
            brightness: 1.0,
            awake_enabled: true,
            boot_anim_enabled: true,
        };
        // Should be okay to unwrap this as is since it is a Default
        let json = serde_json::to_string_pretty(&config).unwrap();
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

    fn write_backup(buf: String) {
        let mut path = ANIME_CONFIG_PATH.to_string();
        path.push_str("-old");
        let mut file = File::create(&path).expect("Couldn't overwrite config");
        file.write_all(buf.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }
}
