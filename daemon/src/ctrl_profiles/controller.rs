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
    pub(super) fn do_next_profile(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            config.read();

            let mut i = config
                .toggle_profiles
                .binary_search(&config.active_profile)
                .unwrap_or(0)
                + 1;
            if i >= config.toggle_profiles.len() {
                i = 0;
            }

            let profile = config.toggle_profiles[i].clone();

            if let Some(existing) = config.power_profiles.get(&profile) {
                existing.set_system_all()?;
                config.active_profile = existing.name.clone();
                config.write();
                info!("Profile was changed to: {}", profile);
            }
        }
        Ok(())
    }

    pub(super) fn set_active(&mut self, profile: &str) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            config.read();
            if let Some(existing) = config.power_profiles.get(profile) {
                existing.set_system_all()?;
                config.active_profile = existing.name.clone();
                config.write();
                info!("Profile was changed to: {}", profile);
            }
        }
        Ok(())
    }

    pub(super) fn new_or_modify(&mut self, profile: &Profile) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            config.read();

            if let Some(existing) = config.power_profiles.get_mut(&profile.name) {
                *existing = profile.clone();
                existing.set_system_all()?;
            } else {
                config
                    .power_profiles
                    .insert(profile.name.clone(), profile.clone());
                profile.set_system_all()?;
            }

            config.active_profile = profile.name.clone();
            config.write();
        }
        Ok(())
    }
}
