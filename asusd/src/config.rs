use std::collections::HashMap;

use config_traits::{StdConfig, StdConfigLoad1};
use rog_platform::asus_armoury::FirmwareAttribute;
use rog_platform::cpu::CPUEPP;
use rog_platform::platform::ThrottlePolicy;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "asusd.ron";

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Config {
    // The current charge limit applied
    pub charge_control_end_threshold: u8,
    /// Save charge limit for restoring
    #[serde(skip)]
    pub base_charge_control_end_threshold: u8,
    pub disable_nvidia_powerd_on_battery: bool,
    /// An optional command/script to run when power is changed to AC
    pub ac_command: String,
    /// An optional command/script to run when power is changed to battery
    pub bat_command: String,
    /// Set true if energy_performance_preference should be set if the
    /// throttle/platform profile is changed
    pub throttle_policy_linked_epp: bool,
    /// Which throttle/profile to use on battery power
    pub throttle_policy_on_battery: ThrottlePolicy,
    /// Should the throttle policy be set on bat/ac change?
    pub change_throttle_policy_on_battery: bool,
    /// Which throttle/profile to use on AC power
    pub throttle_policy_on_ac: ThrottlePolicy,
    /// Should the throttle policy be set on bat/ac change?
    pub change_throttle_policy_on_ac: bool,
    /// The energy_performance_preference for this throttle/platform profile
    pub throttle_quiet_epp: CPUEPP,
    /// The energy_performance_preference for this throttle/platform profile
    pub throttle_balanced_epp: CPUEPP,
    /// The energy_performance_preference for this throttle/platform profile
    pub throttle_performance_epp: CPUEPP,
    pub profile_tunings: HashMap<ThrottlePolicy, HashMap<FirmwareAttribute, i32>>,
    pub armoury_settings: HashMap<FirmwareAttribute, i32>,
    /// Temporary state for AC/Batt
    #[serde(skip)]
    pub last_power_plugged: u8
}

impl Default for Config {
    fn default() -> Self {
        Self {
            charge_control_end_threshold: 100,
            base_charge_control_end_threshold: 100,
            disable_nvidia_powerd_on_battery: true,
            ac_command: Default::default(),
            bat_command: Default::default(),
            throttle_policy_linked_epp: true,
            throttle_policy_on_battery: ThrottlePolicy::Quiet,
            change_throttle_policy_on_battery: true,
            throttle_policy_on_ac: ThrottlePolicy::Performance,
            change_throttle_policy_on_ac: true,
            throttle_quiet_epp: CPUEPP::Power,
            throttle_balanced_epp: CPUEPP::BalancePower,
            throttle_performance_epp: CPUEPP::Performance,
            profile_tunings: HashMap::default(),
            armoury_settings: HashMap::default(),
            last_power_plugged: Default::default()
        }
    }
}

impl StdConfig for Config {
    fn new() -> Self {
        Config {
            charge_control_end_threshold: 100,
            disable_nvidia_powerd_on_battery: true,
            throttle_policy_on_battery: ThrottlePolicy::Quiet,
            throttle_policy_on_ac: ThrottlePolicy::Performance,
            ac_command: String::new(),
            bat_command: String::new(),
            ..Default::default()
        }
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }
}

impl StdConfigLoad1<Config601> for Config {}

#[derive(Deserialize, Serialize)]
pub struct Config601 {
    pub charge_control_end_threshold: u8,
    #[serde(skip)]
    pub base_charge_control_end_threshold: u8,
    pub panel_od: bool,
    pub boot_sound: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
    pub throttle_policy_linked_epp: bool,
    pub throttle_policy_on_battery: ThrottlePolicy,
    pub change_throttle_policy_on_battery: bool,
    pub throttle_policy_on_ac: ThrottlePolicy,
    pub change_throttle_policy_on_ac: bool,
    pub throttle_quiet_epp: CPUEPP,
    pub throttle_balanced_epp: CPUEPP,
    pub throttle_performance_epp: CPUEPP,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_pl1_spl: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_pl2_sppt: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_pl3_fppt: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_fppt: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_apu_sppt: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_platform_sppt: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nv_dynamic_boost: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nv_temp_target: Option<u8>,
    #[serde(skip)]
    pub last_power_plugged: u8
}

impl From<Config601> for Config {
    fn from(c: Config601) -> Self {
        Self {
            // Restore the base charge limit
            charge_control_end_threshold: c.charge_control_end_threshold,
            base_charge_control_end_threshold: c.charge_control_end_threshold,
            disable_nvidia_powerd_on_battery: c.disable_nvidia_powerd_on_battery,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
            throttle_policy_linked_epp: c.throttle_policy_linked_epp,
            throttle_policy_on_battery: c.throttle_policy_on_battery,
            change_throttle_policy_on_battery: c.change_throttle_policy_on_battery,
            throttle_policy_on_ac: c.throttle_policy_on_ac,
            change_throttle_policy_on_ac: c.change_throttle_policy_on_ac,
            throttle_quiet_epp: c.throttle_quiet_epp,
            throttle_balanced_epp: c.throttle_balanced_epp,
            throttle_performance_epp: c.throttle_performance_epp,
            last_power_plugged: c.last_power_plugged,
            profile_tunings: HashMap::default(),
            armoury_settings: HashMap::default()
        }
    }
}
