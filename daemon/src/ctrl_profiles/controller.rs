use crate::error::RogError;
use crate::GetSupported;
use log::{info, warn};
use rog_platform::platform::AsusPlatform;
use rog_platform::supported::PlatformProfileFunctions;
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurveProfiles, Profile};

use super::config::ProfileConfig;

pub struct CtrlPlatformProfile {
    pub config: ProfileConfig,
    pub platform: AsusPlatform,
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

impl CtrlPlatformProfile {
    pub fn new(config: ProfileConfig) -> Result<Self, RogError> {
        let platform = AsusPlatform::new()?;
        if platform.has_platform_profile() || platform.has_throttle_thermal_policy() {
            info!("Device has profile control available");

            let mut controller = CtrlPlatformProfile { config, platform };
            if FanCurveProfiles::get_device().is_ok() {
                info!("Device has fan curves available");
                if controller.config.fan_curves.is_none() {
                    controller.config.fan_curves = Some(Default::default());
                    for _ in [Profile::Balanced, Profile::Performance, Profile::Quiet] {
                        controller.set_next_profile()?;
                        controller.set_active_curve_to_defaults()?;

                        let active = Profile::get_active_profile().unwrap_or(Profile::Balanced);
                        if let Some(curves) = controller.config.fan_curves.as_ref() {
                            info!(
                                "{active:?}: {}",
                                String::from(curves.get_fan_curves_for(active))
                            );
                        }
                    }
                }
            }

            return Ok(controller);
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
