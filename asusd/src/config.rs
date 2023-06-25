use config_traits::{StdConfig, StdConfigLoad2};
use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE: &str = "asusd.ron";

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
}

impl StdConfig for Config {
    fn new() -> Self {
        Config {
            bat_charge_limit: 100,
            panel_od: false,
            mini_led_mode: false,
            disable_nvidia_powerd_on_battery: true,
            ac_command: String::new(),
            bat_command: String::new(),
        }
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }
}

impl StdConfigLoad2<Config458, Config462> for Config {}

#[derive(Deserialize, Serialize)]
pub struct Config462 {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
}

impl From<Config462> for Config {
    fn from(c: Config462) -> Self {
        Self {
            bat_charge_limit: c.bat_charge_limit,
            panel_od: c.panel_od,
            mini_led_mode: false,
            disable_nvidia_powerd_on_battery: true,
            ac_command: String::new(),
            bat_command: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
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
            mini_led_mode: false,
            disable_nvidia_powerd_on_battery: true,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
        }
    }
}
