use config_traits::{StdConfig, StdConfigLoad};
use rog_slash::{DeviceState, SlashMode, SlashType};
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "slash.ron";

/// Config for base system actions for the anime display
#[derive(Deserialize, Serialize, Debug)]
pub struct SlashConfig {
    #[serde(skip)]
    pub slash_type: SlashType,
    pub slash_enabled: bool,
    pub slash_brightness: u8,
    pub slash_interval: u8,
    pub slash_mode: SlashMode,
}

impl Default for SlashConfig {
    fn default() -> Self {
        SlashConfig {
            slash_enabled: true,
            slash_brightness: 255,
            slash_interval: 0,
            slash_mode: SlashMode::Bounce,
            slash_type: SlashType::Unsupported,
        }
    }
}
impl StdConfig for SlashConfig {
    fn new() -> Self {
        Self::default()
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }
}

impl StdConfigLoad for SlashConfig {}

impl From<&SlashConfig> for DeviceState {
    fn from(config: &SlashConfig) -> Self {
        DeviceState {
            slash_enabled: config.slash_enabled,
            slash_brightness: config.slash_brightness,
            slash_interval: config.slash_interval,
            slash_mode: config.slash_mode,
        }
    }
}
