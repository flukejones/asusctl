use std::path::PathBuf;

use config_traits::{StdConfig, StdConfigLoad1};
use rog_profiles::fan_curve_set::FanCurveSet;
use rog_profiles::{FanCurveProfiles, Profile};
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
        CONFIG_FILE.to_string()
    }
}

impl StdConfigLoad1 for ProfileConfig {}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveConfig {
    balanced: FanCurveSet,
    performance: FanCurveSet,
    quiet: FanCurveSet,
    #[serde(skip)]
    device: FanCurveProfiles,
}

impl FanCurveConfig {
    pub fn update_device_config(&mut self) {
        self.balanced = self.device.balanced.clone();
        self.performance = self.device.performance.clone();
        self.quiet = self.device.quiet.clone();
    }

    pub fn update_config(&mut self) {
        self.balanced = self.device.balanced.clone();
        self.performance = self.device.performance.clone();
        self.quiet = self.device.quiet.clone();
    }

    pub fn device(&self) -> &FanCurveProfiles {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut FanCurveProfiles {
        &mut self.device
    }
}

impl StdConfig for FanCurveConfig {
    fn new() -> Self {
        let mut tmp = Self::default();
        tmp.update_device_config();
        tmp
    }

    fn config_dir() -> std::path::PathBuf {
        PathBuf::from(CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FAN_FILE.to_string()
    }
}

impl StdConfigLoad1 for FanCurveConfig {}
