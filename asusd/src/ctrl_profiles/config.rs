use std::path::PathBuf;

use config_traits::{StdConfig, StdConfigLoad};
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::Profile;
use serde_derive::{Deserialize, Serialize};

use crate::CONFIG_PATH_BASE;

const CONFIG_FILE: &str = "profile.ron";
const CONFIG_FAN_FILE: &str = "fan_curves.ron";

#[derive(Deserialize, Serialize, Debug)]
pub struct ProfileConfig {
    /// For restore on boot
    pub active_profile: Profile,
}

impl StdConfig for ProfileConfig {
    fn new() -> Self {
        Self {
            active_profile: Profile::Balanced,
        }
    }

    fn config_dir() -> std::path::PathBuf {
        PathBuf::from(CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }
}

impl StdConfigLoad for ProfileConfig {}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveConfig {
    pub balanced: Vec<CurveData>,
    pub performance: Vec<CurveData>,
    pub quiet: Vec<CurveData>,
}

impl StdConfig for FanCurveConfig {
    /// Create a new config. The defaults are zeroed so the device must be read
    /// to get the actual device defaults.
    fn new() -> Self {
        Self::default()
    }

    fn config_dir() -> std::path::PathBuf {
        PathBuf::from(CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FAN_FILE.to_owned()
    }
}

impl StdConfigLoad for FanCurveConfig {}
