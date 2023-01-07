use config_traits::StdConfig;
use log::{info, warn};
use rog_platform::platform::AsusPlatform;
use rog_platform::supported::PlatformProfileFunctions;
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurveProfiles, Profile};

use super::config::{FanCurveConfig, ProfileConfig};
use crate::error::RogError;
use crate::GetSupported;

pub struct CtrlPlatformProfile {
    pub profile_config: ProfileConfig,
    pub fan_config: Option<FanCurveConfig>,
    pub platform: AsusPlatform,
}

impl GetSupported for CtrlPlatformProfile {
    type A = PlatformProfileFunctions;

    fn get_supported() -> Self::A {
        if !Profile::is_platform_profile_supported() {
            warn!(
                "platform_profile kernel interface not found, your laptop does not support this, \
                 or the interface is missing."
            );
        }

        let res = FanCurveProfiles::is_supported();
        let mut fan_curve_supported = res.is_err();
        if let Ok(r) = res {
            fan_curve_supported = r;
        };

        if !fan_curve_supported {
            info!(
                "fan curves kernel interface not found, your laptop does not support this, or the \
                 interface is missing."
            );
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

            let mut controller = CtrlPlatformProfile {
                profile_config: config,
                fan_config: None,
                platform,
            };
            if FanCurveProfiles::get_device().is_ok() {
                info!("Device has fan curves available");
                if controller.fan_config.is_none() {
                    controller.fan_config = Some(Default::default());
                    for _ in [Profile::Balanced, Profile::Performance, Profile::Quiet] {
                        controller.set_next_profile()?;
                        controller.set_active_curve_to_defaults()?;

                        let active = Profile::get_active_profile().unwrap_or(Profile::Balanced);
                        if let Some(curves) = controller.fan_config.as_ref() {
                            info!(
                                "{active:?}: {}",
                                String::from(curves.device().get_fan_curves_for(active))
                            );
                            curves.write();
                        }
                    }
                }
            }

            return Ok(controller);
        }

        Err(ProfileError::NotSupported.into())
    }

    pub fn save_config(&mut self) {
        self.profile_config.write();
        if let Some(fans) = self.fan_config.as_mut() {
            fans.update_config();
            fans.write();
        }
    }

    /// Toggle to next profile in list. This will first read the config, switch,
    /// then write out
    pub(super) fn set_next_profile(&mut self) -> Result<(), RogError> {
        // Read first just incase the user has modified the config before calling this
        match self.profile_config.active_profile {
            Profile::Balanced => {
                Profile::set_profile(Profile::Performance)?;
                self.profile_config.active_profile = Profile::Performance;
            }
            Profile::Performance => {
                Profile::set_profile(Profile::Quiet)?;
                self.profile_config.active_profile = Profile::Quiet;
            }
            Profile::Quiet => {
                Profile::set_profile(Profile::Balanced)?;
                self.profile_config.active_profile = Profile::Balanced;
            }
        }
        self.write_profile_curve_to_platform()?;
        Ok(())
    }

    /// Set the curve for the active profile active
    pub(super) fn write_profile_curve_to_platform(&mut self) -> Result<(), RogError> {
        if let Some(curves) = &mut self.fan_config {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.device_mut().write_profile_curve_to_platform(
                    self.profile_config.active_profile,
                    &mut device,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn set_active_curve_to_defaults(&mut self) -> Result<(), RogError> {
        if let Some(curves) = self.fan_config.as_mut() {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.device_mut().set_active_curve_to_defaults(
                    self.profile_config.active_profile,
                    &mut device,
                )?;
                curves.update_config();
            }
        }
        Ok(())
    }
}
