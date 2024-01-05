use config_traits::{StdConfig, StdConfigLoad3};
use rog_platform::platform::PlatformPolicy;
use serde_derive::{Deserialize, Serialize};

const CONFIG_FILE: &str = "asusd.ron";

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    /// Save charge limit for restoring on boot
    pub charge_control_end_threshold: u8,
    pub panel_od: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
    pub platform_policy_linked_epp: bool,
    pub platform_policy_on_battery: PlatformPolicy,
    pub platform_policy_on_ac: PlatformPolicy,
    //
    pub ppt_pl1_spl: Option<u8>,
    pub ppt_pl2_sppt: Option<u8>,
    pub ppt_fppt: Option<u8>,
    pub ppt_apu_sppt: Option<u8>,
    pub ppt_platform_sppt: Option<u8>,
    pub nv_dynamic_boost: Option<u8>,
    pub nv_temp_target: Option<u8>,
}

impl StdConfig for Config {
    fn new() -> Self {
        Config {
            charge_control_end_threshold: 100,
            disable_nvidia_powerd_on_battery: true,
            platform_policy_on_battery: PlatformPolicy::Quiet,
            platform_policy_on_ac: PlatformPolicy::Performance,
            ac_command: String::new(),
            bat_command: String::new(),
            ..Default::default()
        }
    }

    fn config_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(crate::CONFIG_PATH_BASE)
    }

    fn file_name(&self) -> String {
        CONFIG_FILE.to_owned()
    }
}

impl StdConfigLoad3<Config462, Config472, Config506> for Config {}

#[derive(Deserialize, Serialize)]
pub struct Config506 {
    /// Save charge limit for restoring on boot
    pub charge_control_end_threshold: u8,
    pub panel_od: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
    /// Restored on boot as well as when power is plugged
    #[serde(skip)]
    pub platform_policy_to_restore: PlatformPolicy,
    pub platform_policy_on_battery: PlatformPolicy,
    pub platform_policy_on_ac: PlatformPolicy,
    //
    pub ppt_pl1_spl: Option<u8>,
    pub ppt_pl2_sppt: Option<u8>,
    pub ppt_fppt: Option<u8>,
    pub ppt_apu_sppt: Option<u8>,
    pub ppt_platform_sppt: Option<u8>,
    pub nv_dynamic_boost: Option<u8>,
    pub nv_temp_target: Option<u8>,
}

impl From<Config506> for Config {
    fn from(c: Config506) -> Self {
        Self {
            charge_control_end_threshold: c.charge_control_end_threshold,
            panel_od: c.panel_od,
            disable_nvidia_powerd_on_battery: c.disable_nvidia_powerd_on_battery,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
            mini_led_mode: c.mini_led_mode,
            platform_policy_linked_epp: true,
            platform_policy_on_battery: c.platform_policy_on_battery,
            platform_policy_on_ac: c.platform_policy_on_ac,
            ppt_pl1_spl: c.ppt_pl1_spl,
            ppt_pl2_sppt: c.ppt_pl2_sppt,
            ppt_fppt: c.ppt_fppt,
            ppt_apu_sppt: c.ppt_apu_sppt,
            ppt_platform_sppt: c.ppt_platform_sppt,
            nv_dynamic_boost: c.nv_dynamic_boost,
            nv_temp_target: c.nv_temp_target,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config472 {
    /// Save charge limit for restoring on boot
    pub bat_charge_limit: u8,
    pub panel_od: bool,
    pub mini_led_mode: bool,
    pub disable_nvidia_powerd_on_battery: bool,
    pub ac_command: String,
    pub bat_command: String,
}

impl From<Config472> for Config {
    fn from(c: Config472) -> Self {
        Self {
            charge_control_end_threshold: c.bat_charge_limit,
            panel_od: c.panel_od,
            disable_nvidia_powerd_on_battery: true,
            ac_command: c.ac_command,
            bat_command: c.bat_command,
            ..Default::default()
        }
    }
}

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
            charge_control_end_threshold: c.bat_charge_limit,
            panel_od: c.panel_od,
            disable_nvidia_powerd_on_battery: true,
            ac_command: String::new(),
            bat_command: String::new(),
            ..Default::default()
        }
    }
}
