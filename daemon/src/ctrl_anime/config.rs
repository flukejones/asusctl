use std::time::Duration;

use config_traits::{StdConfig, StdConfigLoad2};
use rog_anime::error::AnimeError;
use rog_anime::{ActionData, ActionLoader, AnimTime, AnimeType, Fade, Vec2};
use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE: &str = "anime.ron";

#[derive(Deserialize, Serialize)]
pub struct AnimeConfigV341 {
    pub system: Option<ActionLoader>,
    pub boot: Option<ActionLoader>,
    pub suspend: Option<ActionLoader>,
    pub shutdown: Option<ActionLoader>,
}

impl From<AnimeConfigV341> for AnimeConfig {
    fn from(c: AnimeConfigV341) -> AnimeConfig {
        AnimeConfig {
            system: if let Some(ani) = c.system {
                vec![ani]
            } else {
                vec![]
            },
            boot: if let Some(ani) = c.boot {
                vec![ani]
            } else {
                vec![]
            },
            wake: if let Some(ani) = c.suspend {
                vec![ani]
            } else {
                vec![]
            },
            shutdown: if let Some(ani) = c.shutdown {
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

impl From<AnimeConfigV352> for AnimeConfig {
    fn from(c: AnimeConfigV352) -> AnimeConfig {
        AnimeConfig {
            system: c.system,
            boot: c.boot,
            wake: c.wake,
            shutdown: c.shutdown,
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
    pub fn init_from_config(
        &mut self,
        config: &AnimeConfig,
        anime_type: AnimeType,
    ) -> Result<(), AnimeError> {
        let mut sys = Vec::with_capacity(config.system.len());
        for ani in &config.system {
            sys.push(ActionData::from_anime_action(anime_type, ani)?);
        }
        self.system = sys;

        let mut boot = Vec::with_capacity(config.boot.len());
        for ani in &config.boot {
            boot.push(ActionData::from_anime_action(anime_type, ani)?);
        }
        self.boot = boot;

        let mut wake = Vec::with_capacity(config.wake.len());
        for ani in &config.wake {
            wake.push(ActionData::from_anime_action(anime_type, ani)?);
        }
        self.wake = wake;

        let mut shutdown = Vec::with_capacity(config.shutdown.len());
        for ani in &config.shutdown {
            shutdown.push(ActionData::from_anime_action(anime_type, ani)?);
        }
        self.shutdown = shutdown;
        Ok(())
    }
}

/// Config for base system actions for the anime display
#[derive(Deserialize, Serialize, Debug)]
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

impl StdConfig for AnimeConfig {
    fn new() -> Self {
        Self::create_default()
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }
}

impl StdConfigLoad2<AnimeConfigV341, AnimeConfigV352> for AnimeConfig {}

impl AnimeConfig {
    // fn clamp_config_brightness(mut config: &mut AnimeConfig) {
    //     if config.brightness < 0.0 || config.brightness > 1.0 {
    //         warn!(
    //             "Clamped brightness to [0.0 ; 1.0], was {}",
    //             config.brightness
    //         );
    //         config.brightness = f32::max(0.0, f32::min(1.0, config.brightness));
    //     }
    // }

    fn create_default() -> Self {
        // create a default config here
        AnimeConfig {
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
        }
    }
}
