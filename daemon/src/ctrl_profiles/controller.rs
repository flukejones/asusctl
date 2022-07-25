use std::sync::{Arc, Mutex};

use crate::error::RogError;
use crate::{CtrlTask, GetSupported};
use async_trait::async_trait;
use log::{info, warn};
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurveProfiles, Profile};
use rog_supported::PlatformProfileFunctions;
use smol::Executor;

use super::config::ProfileConfig;

pub struct CtrlPlatformProfile {
    pub config: ProfileConfig,
}

impl GetSupported for CtrlPlatformProfile {
    type A = PlatformProfileFunctions;

    fn get_supported() -> Self::A {
        if !Profile::is_platform_profile_supported() {
            warn!("platform_profile kernel interface not found, your laptop does not support this, or the interface is missing.");
        }

        let res = FanCurveProfiles::is_supported();
        let mut fan_curve_supported = res.is_err();
        if let Ok(r) = res {
            fan_curve_supported = r;
        };

        if !fan_curve_supported {
            info!("fan curves kernel interface not found, your laptop does not support this, or the interface is missing.");
        }

        PlatformProfileFunctions {
            platform_profile: Profile::is_platform_profile_supported(),
            fan_curves: fan_curve_supported,
        }
    }
}

impl crate::Reloadable for CtrlPlatformProfile {
    /// Fetch the active profile and use that to set all related components up
    fn reload(&mut self) -> Result<(), RogError> {
        if let Some(curves) = &mut self.config.fan_curves {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                // There is a possibility that the curve was default zeroed, so this call initialises
                // the data from system read and we need to save it after
                curves.write_profile_curve_to_platform(self.config.active_profile, &mut device)?;
                self.config.write();
            }
        }
        Ok(())
    }
}

impl CtrlPlatformProfile {
    pub fn new(mut config: ProfileConfig) -> Result<Self, RogError> {
        if Profile::is_platform_profile_supported() {
            info!("Device has profile control available");

            if FanCurveProfiles::get_device().is_ok() {
                info!("Device has fan curves available");
                if config.fan_curves.is_none() {
                    let active = Profile::get_active_profile().unwrap_or(Profile::Balanced);
                    let dev = FanCurveProfiles::get_device()?;
                    let mut curves = FanCurveProfiles::default();

                    warn!("No default fan-curves: cycling profiles to set defaults");
                    Profile::set_profile(Profile::Balanced)?;
                    curves.read_from_dev_profile(Profile::Balanced, &dev);
                    info!(
                        "{:?}: {}",
                        config.active_profile,
                        String::from(curves.get_fan_curves_for(Profile::Balanced))
                    );
                    Profile::set_profile(Profile::Performance)?;
                    curves.read_from_dev_profile(Profile::Performance, &dev);
                    info!(
                        "{:?}: {}",
                        config.active_profile,
                        String::from(curves.get_fan_curves_for(Profile::Performance))
                    );
                    Profile::set_profile(Profile::Quiet)?;
                    curves.read_from_dev_profile(Profile::Quiet, &dev);
                    info!(
                        "{:?}: {}",
                        config.active_profile,
                        String::from(curves.get_fan_curves_for(Profile::Quiet))
                    );

                    Profile::set_profile(active)?;
                    config.fan_curves = Some(curves);
                    config.write();
                    info!("Set fan curve defaults");
                }
            }

            return Ok(CtrlPlatformProfile { config });
        }

        Err(ProfileError::NotSupported.into())
    }

    pub fn save_config(&self) {
        self.config.write();
    }

    /// Toggle to next profile in list. This will first read the config, switch, then write out
    pub(super) fn set_next_profile(&mut self) -> Result<(), RogError> {
        // Read first just incase the user has modified the config before calling this
        match self.config.active_profile {
            Profile::Balanced => {
                Profile::set_profile(Profile::Performance)?;
                self.config.active_profile = Profile::Performance;
            }
            Profile::Performance => {
                Profile::set_profile(Profile::Quiet)?;
                self.config.active_profile = Profile::Quiet;
            }
            Profile::Quiet => {
                Profile::set_profile(Profile::Balanced)?;
                self.config.active_profile = Profile::Balanced;
            }
        }
        self.write_profile_curve_to_platform()?;
        Ok(())
    }

    /// Set the curve for the active profile active
    pub(super) fn write_profile_curve_to_platform(&mut self) -> Result<(), RogError> {
        if let Some(curves) = &mut self.config.fan_curves {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.write_profile_curve_to_platform(self.config.active_profile, &mut device)?;
            }
        }
        Ok(())
    }

    pub(super) fn set_active_curve_to_defaults(&mut self) -> Result<(), RogError> {
        if let Some(curves) = self.config.fan_curves.as_mut() {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.set_active_curve_to_defaults(self.config.active_profile, &mut device)?;
            }
        }
        Ok(())
    }
}

pub struct CtrlProfileTask {
    ctrl: Arc<Mutex<CtrlPlatformProfile>>,
}

impl CtrlProfileTask {
    pub fn new(ctrl: Arc<Mutex<CtrlPlatformProfile>>) -> Self {
        Self { ctrl }
    }
}

#[async_trait]
impl CtrlTask for CtrlProfileTask {
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let ctrl = self.ctrl.clone();
        self.repeating_task(666, executor, move || {
            if let Ok(ref mut lock) = ctrl.try_lock() {
                let new_profile = Profile::get_active_profile().unwrap();
                if new_profile != lock.config.active_profile {
                    lock.config.active_profile = new_profile;
                    lock.write_profile_curve_to_platform().unwrap();
                    lock.save_config();
                }
            }
        })
        .await;
        Ok(())
    }
}
