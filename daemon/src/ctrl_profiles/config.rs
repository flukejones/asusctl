use rog_profiles::{FanCurveProfiles, Profile};
use serde_derive::{Deserialize, Serialize};

use crate::config_traits::{StdConfig, StdConfigLoad1};

const CONFIG_FILE: &str = "profile.conf";
const CONFIG_FAN_FILE: &str = "fan_curves.conf";

#[derive(Deserialize, Serialize, Debug)]
pub struct ProfileConfig {
    /// For restore on boot
    pub active_profile: Profile,
    /// States to restore
    pub fan_curves: Option<FanCurveProfiles>,
}

impl StdConfig for ProfileConfig {
    fn new() -> Self {
        Self {
            active_profile: Profile::Balanced,
            fan_curves: None,
        }
    }

    fn file_name() -> &'static str {
        CONFIG_FILE
    }
}

impl StdConfigLoad1<ProfileConfig> for ProfileConfig {}

impl StdConfig for FanCurveProfiles {
    fn new() -> Self {
        Self::default()
    }

    fn file_name() -> &'static str {
        CONFIG_FAN_FILE
    }
}

impl StdConfigLoad1<ProfileConfig> for FanCurveProfiles {}