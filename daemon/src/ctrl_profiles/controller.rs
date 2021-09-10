use crate::error::RogError;
use crate::GetSupported;
use log::{info, warn};
use rog_profiles::error::ProfileError;
use rog_profiles::fan_curve_set::FanCurveSet;
use rog_profiles::Profile;
use rog_supported::PlatformProfileFunctions;
use udev::Device;

use super::config::ProfileConfig;

pub struct CtrlPlatformProfile {
    pub config: ProfileConfig,
    pub fan_device: Option<Device>,
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

        let res = FanCurveSet::is_supported();
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
        if let Some(curves) = &self.config.fan_curves {
            if let Ok(mut device) = FanCurveSet::get_device() {
                curves.write_to_platform(self.config.active_profile, &mut device);
            }
        }
        Ok(())
    }
}

impl CtrlPlatformProfile {
    pub fn new(mut config: ProfileConfig, fan_device: Option<Device>) -> Result<Self, RogError> {
        if Profile::is_platform_profile_supported() {
            info!("Device has profile control available");

            if let Some(ref device) = fan_device {
                let profile = config.active_profile;
                config
                    .fan_curves
                    .as_mut()
                    .unwrap()
                    .read_from_dev_profile(profile, device);
            }
            config.write();

            return Ok(CtrlPlatformProfile { config, fan_device });
        }

        Err(ProfileError::NotSupported.into())
    }

    pub fn get_device(&self) -> Option<Device> {
        self.fan_device.clone()
    }

    pub fn save_config(&self) {
        self.config.write();
    }

    /// Toggle to next profile in list. This will first read the config, switch, then write out
    pub(super) fn set_next_profile(&mut self) -> Result<(), RogError> {
        // Read first just incase the user has modified the config before calling this
        self.config.read();

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

        self.config.write();

        Ok(())
    }
}
