use config_traits::{StdConfig, StdConfigLoad};
use log::{info, warn};
use rog_platform::platform::AsusPlatform;
use rog_platform::supported::PlatformProfileFunctions;
use rog_profiles::error::ProfileError;
use rog_profiles::{FanCurveProfiles, Profile};

use super::config::{FanCurveConfig, ProfileConfig};
use crate::error::RogError;
use crate::GetSupported;

// TODO: macro wrapper for warn/info/error log macros to add module name
const MOD_NAME: &str = "CtrlPlatformProfile";

pub struct FanCurves {
    config_file: FanCurveConfig,
    profiles: FanCurveProfiles,
}

impl FanCurves {
    pub fn update_profiles_from_config(&mut self) {
        self.profiles.balanced = self.config_file.balanced.clone();
        self.profiles.performance = self.config_file.performance.clone();
        self.profiles.quiet = self.config_file.quiet.clone();
    }

    pub fn update_config_from_profiles(&mut self) {
        self.config_file.balanced = self.profiles.balanced.clone();
        self.config_file.performance = self.profiles.performance.clone();
        self.config_file.quiet = self.profiles.quiet.clone();
    }

    pub fn profiles(&self) -> &FanCurveProfiles {
        &self.profiles
    }

    pub fn profiles_mut(&mut self) -> &mut FanCurveProfiles {
        &mut self.profiles
    }
}

pub struct CtrlPlatformProfile {
    pub profile_config: ProfileConfig,
    pub fan_curves: Option<FanCurves>,
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

        let res = FanCurveProfiles::supported_fans();

        if res.is_err() {
            info!(
                "fan curves kernel interface not found, your laptop does not support this, or the \
                 interface is missing."
            );
        }

        PlatformProfileFunctions {
            platform_profile: Profile::is_platform_profile_supported(),
            fans: res.unwrap_or_default(),
        }
    }
}

impl CtrlPlatformProfile {
    pub fn new(config: ProfileConfig) -> Result<Self, RogError> {
        let platform = AsusPlatform::new()?;
        if platform.has_platform_profile() || platform.has_throttle_thermal_policy() {
            info!("{MOD_NAME}: Device has profile control available");

            let mut controller = CtrlPlatformProfile {
                profile_config: config,
                fan_curves: None,
                platform,
            };
            if FanCurveProfiles::get_device().is_ok() {
                info!("{MOD_NAME}: Device has fan curves available");
                let fan_config = FanCurveConfig::new();
                // Only do defaults if the config doesn't already exist
                if !fan_config.file_path().exists() {
                    info!("{MOD_NAME}: Fetching default fan curves");
                    controller.fan_curves = Some(FanCurves {
                        config_file: fan_config,
                        profiles: FanCurveProfiles::default(),
                    });
                    for _ in [Profile::Balanced, Profile::Performance, Profile::Quiet] {
                        // For each profile we need to switch to it before we
                        // can read the existing values from hardware. The ACPI method used
                        // for this is what limits us.
                        let next =
                            Profile::get_next_profile(controller.profile_config.active_profile);
                        Profile::set_profile(next)
                            .map_err(|e| warn!("{MOD_NAME}: set_profile, {}", e))
                            .ok();
                        controller.profile_config.active_profile = next;

                        // Make sure to set the baseline to default
                        controller.set_active_curve_to_defaults()?;
                        let active = Profile::get_active_profile().unwrap_or(Profile::Balanced);

                        if let Some(curves) = controller.fan_curves.as_ref() {
                            info!("{MOD_NAME}: {active:?}:");
                            for curve in curves.profiles().get_fan_curves_for(active) {
                                info!("{}", String::from(curve));
                            }
                        }
                    }
                    if let Some(curves) = controller.fan_curves.as_ref() {
                        curves.config_file.write();
                    }
                } else {
                    info!("{MOD_NAME}: Fan curves previously stored, loading...");
                    let mut fan_curves = FanCurves {
                        config_file: fan_config.load(),
                        profiles: FanCurveProfiles::default(),
                    };
                    fan_curves.update_profiles_from_config();
                    controller.fan_curves = Some(fan_curves);
                }
            }

            return Ok(controller);
        }

        Err(ProfileError::NotSupported.into())
    }

    pub fn save_config(&mut self) {
        self.profile_config.write();
        if let Some(fans) = self.fan_curves.as_mut() {
            fans.update_config_from_profiles();
            fans.config_file.write(); // config write
        }
    }

    /// Set the curve for the active profile active
    pub(super) fn write_profile_curve_to_platform(&mut self) -> Result<(), RogError> {
        if let Some(curves) = &mut self.fan_curves {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.profiles_mut().write_profile_curve_to_platform(
                    self.profile_config.active_profile,
                    &mut device,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn set_active_curve_to_defaults(&mut self) -> Result<(), RogError> {
        if let Some(curves) = self.fan_curves.as_mut() {
            if let Ok(mut device) = FanCurveProfiles::get_device() {
                curves.profiles_mut().set_active_curve_to_defaults(
                    self.profile_config.active_profile,
                    &mut device,
                )?;
                curves.update_config_from_profiles();
            }
        }
        Ok(())
    }
}
