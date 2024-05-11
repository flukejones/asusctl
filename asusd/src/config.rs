use config_traits::{StdConfig, StdConfigLoad1};
use rog_platform::cpu::CPUEPP;
use rog_platform::platform::ThrottlePolicy;
use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE: &str = "asusd.ron";

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd)]
pub struct Config {
    /// Save charge limit for restoring on boot/resume
    pub charge_control_end_threshold: u8,
    pub panel_od: bool,
    pub boot_sound: bool,
    pub mini_led_mode: bool,
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
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_pl1_spl: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_pl2_sppt: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_fppt: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_apu_sppt: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ppt_platform_sppt: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nv_dynamic_boost: Option<u8>,
    /// Defaults to `None` if not supported
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nv_temp_target: Option<u8>,
    /// Temporary state for AC/Batt
    #[serde(skip)]
    pub last_power_plugged: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            charge_control_end_threshold: 100,
            panel_od: false,
            boot_sound: false,
            mini_led_mode: false,
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
            ppt_pl1_spl: Default::default(),
            ppt_pl2_sppt: Default::default(),
            ppt_fppt: Default::default(),
            ppt_apu_sppt: Default::default(),
            ppt_platform_sppt: Default::default(),
            nv_dynamic_boost: Default::default(),
            nv_temp_target: Default::default(),
            last_power_plugged: Default::default(),
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

impl StdConfigLoad1<Config507> for Config {}

#[derive(Deserialize, Serialize)]
pub struct Config507 {
    /// Save charge limit for restoring on boot
    pub charge_control_end_threshold: u8,
    pub panel_od: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
    pub platform_policy_linked_epp: bool,
    pub platform_policy_on_battery: ThrottlePolicy,
    pub platform_policy_on_ac: ThrottlePolicy,
    //
    pub ppt_pl1_spl: Option<u8>,
    pub ppt_pl2_sppt: Option<u8>,
    pub ppt_fppt: Option<u8>,
    pub ppt_apu_sppt: Option<u8>,
    pub ppt_platform_sppt: Option<u8>,
    pub nv_dynamic_boost: Option<u8>,
    pub nv_temp_target: Option<u8>,
}

impl From<Config507> for Config {
    fn from(c: Config507) -> Self {
        Self {
            charge_control_end_threshold: c.charge_control_end_threshold,
            panel_od: c.panel_od,
            boot_sound: false,
            disable_nvidia_powerd_on_battery: c.disable_nvidia_powerd_on_battery,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
            mini_led_mode: c.mini_led_mode,
            throttle_policy_linked_epp: true,
            throttle_policy_on_battery: c.platform_policy_on_battery,
            change_throttle_policy_on_battery: true,
            throttle_policy_on_ac: c.platform_policy_on_ac,
            change_throttle_policy_on_ac: true,
            throttle_quiet_epp: CPUEPP::Power,
            throttle_balanced_epp: CPUEPP::BalancePower,
            throttle_performance_epp: CPUEPP::Performance,
            ppt_pl1_spl: c.ppt_pl1_spl,
            ppt_pl2_sppt: c.ppt_pl2_sppt,
            ppt_fppt: c.ppt_fppt,
            ppt_apu_sppt: c.ppt_apu_sppt,
            ppt_platform_sppt: c.ppt_platform_sppt,
            nv_dynamic_boost: c.nv_dynamic_boost,
            nv_temp_target: c.nv_temp_target,
            last_power_plugged: 0,
        }
    }
}
