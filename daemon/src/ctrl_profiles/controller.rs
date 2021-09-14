use std::sync::{Arc, Mutex};

use crate::error::RogError;
use crate::{CtrlTask, GetSupported};
use log::{info, warn};
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurveProfiles, Profile};
use rog_supported::PlatformProfileFunctions;

use super::config::ProfileConfig;

pub struct CtrlPlatformProfile {
    pub config: ProfileConfig,
}

impl GetSupported for CtrlPlatformProfile {
    type A = PlatformProfileFunctions;

    fn get_supported() -> Self::A {
        if !Profile::is_platform_profile_supported() {
            warn!(
                r#"
platform_profile kernel interface not found, your laptop does not support this, or the interface is missing.
To enable profile support you require a kernel with the following patch applied:
https://lkml.org/lkml/2021/8/18/1022
"#
            );
        }

        let res = FanCurveProfiles::is_supported();
        let mut fan_curve_supported = res.is_err();
        if let Ok(r) = res {
            fan_curve_supported = r;
        };

        if fan_curve_supported {
            info!(
                r#"
fan curves kernel interface not found, your laptop does not support this, or the interface is missing.
To enable fan-curve support you require a kernel with the following patch applied:
https://lkml.org/lkml/2021/8/20/232
Please note that as of 24/08/2021 this is not final.
"#
            );
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
                curves.write_profile_curve_to_platform(self.config.active_profile, &mut device)?;
            }
        }
        Ok(())
    }
}

impl CtrlPlatformProfile {
    pub fn new(mut config: ProfileConfig) -> Result<Self, RogError> {
        if Profile::is_platform_profile_supported() {
            info!("Device has profile control available");

            if let Ok(ref device) = FanCurveProfiles::get_device() {
                let profile = config.active_profile;
                if let Some(curve) = config.fan_curves.as_mut() {
                    curve.read_from_dev_profile(profile, device);
                }
            }
            config.write();

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

impl CtrlTask for CtrlProfileTask {
    fn do_task(&self) -> Result<(), RogError> {
        if let Ok(ref mut lock) = self.ctrl.try_lock() {
            let new_profile = Profile::get_active_profile().unwrap();
            if new_profile != lock.config.active_profile {
                lock.config.active_profile = new_profile;
                lock.save_config();
            }
        }
        Ok(())
    }
}
