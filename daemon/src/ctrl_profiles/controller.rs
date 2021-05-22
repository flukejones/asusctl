use crate::error::RogError;
use crate::{config::Config, GetSupported};
use log::info;
use rog_profiles::profiles::Profile;
use rog_types::supported::FanCpuSupportedFunctions;
use std::sync::Arc;
use std::sync::Mutex;

pub struct CtrlFanAndCpu {
    pub config: Arc<Mutex<Config>>,
}

impl GetSupported for CtrlFanAndCpu {
    type A = FanCpuSupportedFunctions;

    fn get_supported() -> Self::A {
        FanCpuSupportedFunctions {
            stock_fan_modes: Profile::get_fan_path().is_ok(),
            min_max_freq: Profile::get_intel_supported(),
            fan_curve_set: rog_fan_curve::Board::from_board_name().is_some(),
        }
    }
}

impl crate::Reloadable for CtrlFanAndCpu {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut cfg) = self.config.clone().try_lock() {
            let active = cfg.active_profile.clone();
            if let Some(existing) = cfg.power_profiles.get_mut(&active) {
                existing.set_system_all()?;
                cfg.write();
            }
        }
        Ok(())
    }
}

impl CtrlFanAndCpu {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        Profile::get_fan_path()?;
        info!("Device has thermal throttle control");
        Ok(CtrlFanAndCpu { config })
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

        self.set_active(&new_profile)?;

        info!("Profile was changed: {}", &new_profile);
        Ok(())
    }

    pub(super) fn set_active(&mut self, profile: &str) -> Result<(), RogError> {
        if let Ok(mut cfg) = self.config.clone().try_lock() {
            cfg.read();

            if let Some(existing) = cfg.power_profiles.get_mut(profile) {
                existing.set_system_all()?;
                cfg.active_profile = existing.name.clone();
                cfg.write();
            }
        }
        Ok(())
    }

    pub(super) fn new_or_modify(&mut self, profile: &Profile) -> Result<(), RogError> {
        if let Ok(mut cfg) = self.config.clone().try_lock() {
            cfg.read();

            if let Some(existing) = cfg.power_profiles.get_mut(&profile.name) {
                *existing = profile.clone();
                existing.set_system_all()?;
            } else {
                cfg.power_profiles
                    .insert(profile.name.clone(), profile.clone());
            }
            cfg.active_profile = profile.name.clone();
            cfg.write();
        }
        Ok(())
    }
}
