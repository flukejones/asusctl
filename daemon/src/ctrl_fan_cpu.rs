use crate::error::RogError;
use crate::{config::Config, GetSupported};
use log::{info, warn};
use rog_types::{
    profile::{FanLevel, Profile, ProfileEvent},
    supported::FanCpuSupportedFunctions,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::{dbus_interface, fdo::Error};
use zvariant::ObjectPath;

static FAN_TYPE_1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy";
static FAN_TYPE_2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/fan_boost_mode";
static AMD_BOOST_PATH: &str = "/sys/devices/system/cpu/cpufreq/boost";

pub struct CtrlFanAndCpu {
    pub path: &'static str,
    config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlFanAndCpu {
    type A = FanCpuSupportedFunctions;

    fn get_supported() -> Self::A {
        FanCpuSupportedFunctions {
            stock_fan_modes: CtrlFanAndCpu::get_fan_path().is_ok(),
            min_max_freq: intel_pstate::PState::new().is_ok(),
            fan_curve_set: rog_fan_curve::Board::from_board_name().is_some(),
        }
    }
}

pub struct FanAndCpuZbus {
    inner: Arc<Mutex<CtrlFanAndCpu>>,
}

impl FanAndCpuZbus {
    pub fn new(inner: Arc<Mutex<CtrlFanAndCpu>>) -> Self {
        Self { inner }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl FanAndCpuZbus {
    /// Set profile details
    fn set_profile(&self, profile: String) {
        if let Ok(event) = serde_json::from_str(&profile) {
            if let Ok(mut ctrl) = self.inner.try_lock() {
                if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                    cfg.read();
                    ctrl.handle_profile_event(&event, &mut cfg)
                        .unwrap_or_else(|err| warn!("{}", err));
                    if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                        if let Ok(json) = serde_json::to_string(profile) {
                            self.notify_profile(&json)
                                .unwrap_or_else(|err| warn!("{}", err));
                        }
                    }
                }
            }
        }
    }

    /// Fetch the active profile name
    fn next_profile(&mut self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                cfg.read();
                ctrl.do_next_profile(&mut cfg)
                    .unwrap_or_else(|err| warn!("{}", err));
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string(profile) {
                        self.notify_profile(&json)
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
            }
        }
    }

    /// Fetch the active profile name
    fn active_profile_name(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                return Ok(cfg.active_profile.clone());
            }
        }
        Err(Error::Failed(
            "Failed to get active profile name".to_string(),
        ))
    }

    // TODO: Profile can't implement Type because of Curve
    /// Fetch the active profile details
    fn profile(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string_pretty(profile) {
                        return Ok(json);
                    }
                }
            }
        }
        Err(Error::Failed(
            "Failed to get active profile details".to_string(),
        ))
    }

    /// Fetch all profile data
    fn profiles(&mut self) -> zbus::fdo::Result<String> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Ok(json) = serde_json::to_string_pretty(&cfg.power_profiles) {
                    return Ok(json);
                }
            }
        }
        Err(Error::Failed(
            "Failed to get all profile details".to_string(),
        ))
    }

    fn profile_names(&self) -> zbus::fdo::Result<Vec<String>> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                let profile_names = cfg.power_profiles.keys().cloned().collect::<Vec<String>>();
                return Ok(profile_names);
            }
        }

        Err(Error::Failed("Failed to get all profile names".to_string()))
    }

    fn remove(&self, profile: &str) -> zbus::fdo::Result<()> {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();

                if !cfg.power_profiles.contains_key(profile) {
                    return Err(Error::Failed("Invalid profile specified".to_string()));
                }

                if cfg.power_profiles.keys().len() == 1 {
                    return Err(Error::Failed("Cannot delete the last profile".to_string()));
                }

                if cfg.active_profile == *profile {
                    return Err(Error::Failed(
                        "Cannot delete the active profile".to_string(),
                    ));
                }

                cfg.power_profiles.remove(profile);
                cfg.write();

                return Ok(());
            }
        }

        Err(Error::Failed("Failed to lock configuration".to_string()))
    }

    #[dbus_interface(signal)]
    fn notify_profile(&self, profile: &str) -> zbus::Result<()> {}
}

impl crate::ZbusAdd for FanAndCpuZbus {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &ObjectPath::from_str_unchecked("/org/asuslinux/Profile"),
                self,
            )
            .map_err(|err| {
                warn!("DbusFanAndCpu: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl crate::Reloadable for CtrlFanAndCpu {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            let profile = config.active_profile.clone();
            self.set(&profile, &mut config)?;
            // info!(
            //     "Reloaded fan mode: {:?}",
            //     FanLevel::from(config.power_profile)
            // );
        }
        Ok(())
    }
}

impl CtrlFanAndCpu {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let path = CtrlFanAndCpu::get_fan_path()?;
        info!("Device has thermal throttle control");
        Ok(CtrlFanAndCpu { path, config })
    }

    fn get_fan_path() -> Result<&'static str, RogError> {
        if Path::new(FAN_TYPE_1_PATH).exists() {
            Ok(FAN_TYPE_1_PATH)
        } else if Path::new(FAN_TYPE_2_PATH).exists() {
            Ok(FAN_TYPE_2_PATH)
        } else {
            Err(RogError::MissingFunction(
                "Fan mode not available, you may require a v5.8.10 series kernel or newer".into(),
            ))
        }
    }

    /// Toggle to next profile in list
    pub(super) fn do_next_profile(&mut self, config: &mut Config) -> Result<(), RogError> {
        config.read();

        let mut i = config
            .toggle_profiles
            .iter()
            .position(|x| x == &config.active_profile)
            .map(|i| i + 1)
            .unwrap_or(0);
        if i >= config.toggle_profiles.len() {
            i = 0;
        }

        let new_profile = config
            .toggle_profiles
            .get(i)
            .unwrap_or(&config.active_profile)
            .clone();

        self.set(&new_profile, config)?;

        info!("Profile was changed: {}", &new_profile);
        Ok(())
    }

    fn set_fan_mode(&mut self, preset: u8, config: &mut Config) -> Result<(), RogError> {
        let mode = config.active_profile.clone();
        let mut fan_ctrl = OpenOptions::new()
            .write(true)
            .open(self.path)
            .map_err(|err| RogError::Path(self.path.into(), err))?;
        config.read();
        let mut mode_config = config
            .power_profiles
            .get_mut(&mode)
            .ok_or_else(|| RogError::MissingProfile(mode.clone()))?;
        config.curr_fan_mode = preset;
        mode_config.fan_preset = preset;
        config.write();
        fan_ctrl
            .write_all(format!("{}\n", preset).as_bytes())
            .map_err(|err| RogError::Write(self.path.into(), err))?;
        info!("Fan mode set to: {:?}", FanLevel::from(preset));
        Ok(())
    }

    fn handle_profile_event(
        &mut self,
        event: &ProfileEvent,
        config: &mut Config,
    ) -> Result<(), RogError> {
        match event {
            ProfileEvent::Toggle => self.do_next_profile(config)?,
            ProfileEvent::ChangeMode(mode) => {
                self.set_fan_mode(*mode, config)?;
                let mode = config.active_profile.clone();
                self.set_pstate_for_fan_mode(&mode, config)?;
                self.set_fan_curve_for_fan_mode(&mode, config)?;
            }
            ProfileEvent::Cli(command) => {
                let profile_key = match command.profile.as_ref() {
                    Some(k) => k.clone(),
                    None => config.active_profile.clone(),
                };

                let mut profile = if command.create {
                    config
                        .power_profiles
                        .entry(profile_key.clone())
                        .or_insert_with(Profile::default)
                } else {
                    config
                        .power_profiles
                        .get_mut(&profile_key)
                        .ok_or_else(|| RogError::MissingProfile(profile_key.clone()))?
                };

                if command.turbo.is_some() {
                    profile.turbo = command.turbo.unwrap();
                }
                if let Some(min_perc) = command.min_percentage {
                    profile.min_percentage = min_perc;
                }
                if let Some(max_perc) = command.max_percentage {
                    profile.max_percentage = max_perc;
                }
                if let Some(ref preset) = command.fan_preset {
                    profile.fan_preset = preset.into();
                }
                if let Some(ref curve) = command.curve {
                    profile.fan_curve = Some(curve.clone());
                }

                self.set(&profile_key, config)?;
            }
        }
        Ok(())
    }

    fn set(&mut self, profile: &str, config: &mut Config) -> Result<(), RogError> {
        let mode_config = config
            .power_profiles
            .get(profile)
            .ok_or_else(|| RogError::MissingProfile(profile.into()))?;
        let mut fan_ctrl = OpenOptions::new()
            .write(true)
            .open(self.path)
            .map_err(|err| RogError::Path(self.path.into(), err))?;
        config.curr_fan_mode = mode_config.fan_preset;
        fan_ctrl
            .write_all(format!("{}\n", mode_config.fan_preset).as_bytes())
            .map_err(|err| RogError::Write(self.path.into(), err))?;

        self.set_pstate_for_fan_mode(profile, config)?;
        self.set_fan_curve_for_fan_mode(profile, config)?;

        config.active_profile = profile.into();

        config.write();
        Ok(())
    }

    fn set_pstate_for_fan_mode(&self, mode: &str, config: &mut Config) -> Result<(), RogError> {
        info!("Setting pstate");
        let mode_config = config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        // Set CPU pstate
        if let Ok(pstate) = intel_pstate::PState::new() {
            pstate.set_min_perf_pct(mode_config.min_percentage)?;
            pstate.set_max_perf_pct(mode_config.max_percentage)?;
            pstate.set_no_turbo(!mode_config.turbo)?;
            info!(
                "Intel CPU Power: min: {}%, max: {}%, turbo: {}",
                mode_config.min_percentage, mode_config.max_percentage, mode_config.turbo
            );
        } else {
            info!("Setting pstate for AMD CPU");
            // must be AMD CPU
            let mut file = OpenOptions::new()
                .write(true)
                .open(AMD_BOOST_PATH)
                .map_err(|err| RogError::Path(self.path.into(), err))?;

            let boost = if mode_config.turbo { "1" } else { "0" }; // opposite of Intel
            file.write_all(boost.as_bytes())
                .map_err(|err| RogError::Write(AMD_BOOST_PATH.into(), err))?;
            info!("AMD CPU Turbo: {}", boost);
        }
        Ok(())
    }

    fn set_fan_curve_for_fan_mode(&self, mode: &str, config: &Config) -> Result<(), RogError> {
        let mode_config = &config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        if let Some(ref curve) = mode_config.fan_curve {
            use rog_fan_curve::{Board, Fan};
            if let Some(board) = Board::from_board_name() {
                curve.apply(board, Fan::Cpu)?;
                curve.apply(board, Fan::Gpu)?;
            } else {
                warn!("Fan curve unsupported on this board.")
            }
        }

        Ok(())
    }
}
