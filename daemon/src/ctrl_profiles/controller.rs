use crate::error::RogError;
use crate::GetSupported;
use log::{info, warn};
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurves, Profile};
use rog_supported::PlatformProfileFunctions;
use std::sync::Arc;
use std::sync::Mutex;

use super::config::ProfileConfig;

pub struct CtrlPlatformTask {
    config: Arc<Mutex<ProfileConfig>>,
}

impl CtrlPlatformTask {
    pub fn new(config: Arc<Mutex<ProfileConfig>>) -> Self {
        Self { config }
    }
}

impl crate::CtrlTask for CtrlPlatformTask {
    fn do_task(&self) -> Result<(), RogError> {
        if let Ok(mut lock) = self.config.try_lock() {
            // Refresh the config in-case the user has edited it
            if let Some(curves) = &mut lock.fan_curves {
                curves.update_from_platform();
            }
        }
        Ok(())
    }
}

pub struct CtrlPlatformProfile {
    pub config: Arc<Mutex<ProfileConfig>>,
}

impl GetSupported for CtrlPlatformProfile {
    type A = PlatformProfileFunctions;

    fn get_supported() -> Self::A {
        if !Profile::is_platform_profile_supported() {
            warn!(
                r#"
platform_profile kernel interface not found, your laptop does not support this, or the iterface is missing.
To enable profile support you require a kernel with the following patch applied:
https://lkml.org/lkml/2021/8/18/1022
"#
            );
        }
        if !FanCurves::is_fan_curves_supported() {
            info!(
                r#"
fan curves kernel interface not found, your laptop does not support this, or the iterface is missing.
To enable fan-curve support you require a kernel with the following patch applied:
https://lkml.org/lkml/2021/8/20/232
Please note that as of 24/08/2021 this is not final.
"#
            );
        }
        PlatformProfileFunctions {
            platform_profile: Profile::is_platform_profile_supported(),
            fan_curves: FanCurves::is_fan_curves_supported(),
        }
    }
}

impl crate::Reloadable for CtrlPlatformProfile {
    /// Fetch the active profile and use that to set all related components up
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(cfg) = self.config.clone().try_lock() {
            if let Some(curves) = &cfg.fan_curves {
                curves.update_platform();
            }
        }
        Ok(())
    }
}

impl CtrlPlatformProfile {
    pub fn new(config: Arc<Mutex<ProfileConfig>>) -> Result<Self, RogError> {
        if Profile::is_platform_profile_supported() {
            info!("Device has profile control available");
            return Ok(CtrlPlatformProfile { config });
        }
        Err(ProfileError::NotSupported.into())
    }

    pub fn save_config(&self) {
        if let Ok(lock) = self.config.lock() {
            lock.write();
        }
    }

    /// Toggle to next profile in list. This will first read the config, switch, then write out
    pub(super) fn set_next_profile(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            // Read first just incase the user has modified the config before calling this
            config.read();

            match config.active {
                Profile::Balanced => {
                    Profile::set_profile(Profile::Performance);
                    config.active = Profile::Performance;
                }
                Profile::Performance => {
                    Profile::set_profile(Profile::Quiet);
                    config.active = Profile::Quiet;
                }
                Profile::Quiet => {
                    Profile::set_profile(Profile::Balanced);
                    config.active = Profile::Balanced;
                }
            }

            config.write();
        }
        Ok(())
    }
}
