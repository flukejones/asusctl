pub mod error;
pub mod fan_curve_set;

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

use error::ProfileError;
use fan_curve_set::{CurveData, FanCurveSet};
use serde_derive::{Deserialize, Serialize};

use udev::Device;
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

pub static PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
pub static PLATFORM_PROFILES: &str = "/sys/firmware/acpi/platform_profile_choices";

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn find_fan_curve_node() -> Result<Option<Device>, ProfileError> {
    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("hwmon")?;

    for device in enumerator.scan_devices()? {
        if device.parent_with_subsystem("platform")?.is_some() {
            if let Some(name) = device.attribute_value("name") {
                if name == "asus_custom_fan_curve" {
                    return Ok(Some(device));
                }
            }
        }
    }

    Err(ProfileError::NotSupported)
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum Profile {
    Balanced,
    Performance,
    Quiet,
}

impl Profile {
    pub fn is_platform_profile_supported() -> bool {
        Path::new(PLATFORM_PROFILES).exists()
    }

    pub fn get_active_profile() -> Result<Profile, ProfileError> {
        let mut file = OpenOptions::new().read(true).open(&PLATFORM_PROFILE)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.as_str().into())
    }

    pub fn get_profile_names() -> Result<Vec<Profile>, ProfileError> {
        let mut file = OpenOptions::new().read(true).open(&PLATFORM_PROFILES)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.rsplit(' ').map(|p| p.into()).collect())
    }

    pub fn set_profile(profile: Profile) -> Result<(), ProfileError> {
        let mut file = OpenOptions::new().write(true).open(PLATFORM_PROFILE)?;

        file.write_all(<&str>::from(profile).as_bytes())?;
        Ok(())
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl From<Profile> for &str {
    fn from(profile: Profile) -> &'static str {
        match profile {
            Profile::Balanced => "balanced",
            Profile::Performance => "performance",
            Profile::Quiet => "quiet",
        }
    }
}

impl From<&str> for Profile {
    fn from(profile: &str) -> Profile {
        match profile.to_ascii_lowercase().trim() {
            "balanced" => Profile::Balanced,
            "performance" => Profile::Performance,
            "quiet" => Profile::Quiet,
            _ => Profile::Balanced,
        }
    }
}

impl std::str::FromStr for Profile {
    type Err = ProfileError;

    fn from_str(profile: &str) -> Result<Self, Self::Err> {
        match profile.to_ascii_lowercase().trim() {
            "balanced" => Ok(Profile::Balanced),
            "performance" => Ok(Profile::Performance),
            "quiet" => Ok(Profile::Quiet),
            _ => Err(ProfileError::ParseProfileName),
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum FanCurvePU {
    CPU,
    GPU,
}

impl From<FanCurvePU> for &str {
    fn from(pu: FanCurvePU) -> &'static str {
        match pu {
            FanCurvePU::CPU => "cpu",
            FanCurvePU::GPU => "gpu",
        }
    }
}

impl std::str::FromStr for FanCurvePU {
    type Err = ProfileError;

    fn from_str(fan: &str) -> Result<Self, Self::Err> {
        match fan.to_ascii_lowercase().trim() {
            "cpu" => Ok(FanCurvePU::CPU),
            "gpu" => Ok(FanCurvePU::GPU),
            _ => Err(ProfileError::ParseProfileName),
        }
    }
}

impl Default for FanCurvePU {
    fn default() -> Self {
        Self::CPU
    }
}

/// Main purpose of `FanCurves` is to enable restoring state on system boot
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveProfiles {
    balanced: FanCurveSet,
    performance: FanCurveSet,
    quiet: FanCurveSet,
}

impl FanCurveProfiles {
    pub fn get_device() -> Result<Device, ProfileError> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("hwmon")?;

        for device in enumerator.scan_devices()? {
            if device.parent_with_subsystem("platform")?.is_some() {
                if let Some(name) = device.attribute_value("name") {
                    if name == "asus_custom_fan_curve" {
                        return Ok(device);
                    }
                }
            }
        }
        Err(ProfileError::NotSupported)
    }

    pub fn is_supported() -> Result<bool, ProfileError> {
        if Self::get_device().is_ok() {
            return Ok(true);
        }

        Ok(false)
    }

    ///
    pub fn read_from_dev_profile(&mut self, profile: Profile, device: &Device) {
        let mut tmp = FanCurveSet::default();
        tmp.read_cpu_from_device(device);
        tmp.read_gpu_from_device(device);
        match profile {
            Profile::Balanced => self.balanced = tmp,
            Profile::Performance => self.performance = tmp,
            Profile::Quiet => self.quiet = tmp,
        }
    }

    /// Reset the stored (self) and device curve to the defaults of the platform.
    ///
    /// Each platform_profile has a different default and the defualt can be read
    /// only for the currently active profile.
    pub fn set_active_curve_to_defaults(
        &mut self,
        profile: Profile,
        device: &mut Device,
    ) -> std::io::Result<()> {
        // Do reset
        device.set_attribute_value("pwm1_enable", "3")?;
        device.set_attribute_value("pwm2_enable", "3")?;
        // Then read
        let mut tmp = FanCurveSet::default();
        tmp.read_cpu_from_device(device);
        tmp.read_gpu_from_device(device);
        match profile {
            Profile::Balanced => self.balanced = tmp,
            Profile::Performance => self.performance = tmp,
            Profile::Quiet => self.quiet = tmp,
        }
        Ok(())
    }

    /// Write the curves for the selected profile to the device. If the curve is
    /// in the enabled list it will become active. If the curve is zeroed it will be initialised
    /// to a default read from the system.
    // TODO: Make this return an error if curve is zeroed
    pub fn write_profile_curve_to_platform(
        &mut self,
        profile: Profile,
        device: &mut Device,
    ) -> std::io::Result<()> {
        let fans = match profile {
            Profile::Balanced => &mut self.balanced,
            Profile::Performance => &mut self.performance,
            Profile::Quiet => &mut self.quiet,
        };
        fans.write_cpu_fan(device)?;
        fans.write_gpu_fan(device)?;
        Ok(())
    }

    pub fn get_enabled_curve_profiles(&self) -> Vec<Profile> {
        let mut tmp = Vec::new();
        if self.balanced.enabled {
            tmp.push(Profile::Balanced);
        }
        if self.performance.enabled {
            tmp.push(Profile::Performance);
        }
        if self.quiet.enabled {
            tmp.push(Profile::Quiet);
        }
        tmp
    }

    pub fn set_profile_curve_enabled(&mut self, profile: Profile, enabled: bool) {
        match profile {
            Profile::Balanced => self.balanced.enabled = enabled,
            Profile::Performance => self.performance.enabled = enabled,
            Profile::Quiet => self.quiet.enabled = enabled,
        }
    }

    pub fn get_all_fan_curves(&self) -> Vec<FanCurveSet> {
        vec![
            self.balanced.clone(),
            self.performance.clone(),
            self.quiet.clone(),
        ]
    }

    pub fn get_active_fan_curves(&self) -> Result<&FanCurveSet, ProfileError> {
        match Profile::get_active_profile()? {
            Profile::Balanced => Ok(&self.balanced),
            Profile::Performance => Ok(&self.performance),
            Profile::Quiet => Ok(&self.quiet),
        }
    }

    pub fn get_fan_curves_for(&self, name: Profile) -> &FanCurveSet {
        match name {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
        }
    }

    pub fn get_fan_curve_for(&self, name: &Profile, pu: &FanCurvePU) -> &CurveData {
        match name {
            Profile::Balanced => match pu {
                FanCurvePU::CPU => &self.balanced.cpu,
                FanCurvePU::GPU => &self.balanced.gpu,
            },
            Profile::Performance => match pu {
                FanCurvePU::CPU => &self.balanced.cpu,
                FanCurvePU::GPU => &self.balanced.gpu,
            },
            Profile::Quiet => match pu {
                FanCurvePU::CPU => &self.balanced.cpu,
                FanCurvePU::GPU => &self.balanced.gpu,
            },
        }
    }

    pub fn save_fan_curve(&mut self, curve: CurveData, profile: Profile) -> std::io::Result<()> {
        match profile {
            Profile::Balanced => match curve.fan {
                FanCurvePU::CPU => self.balanced.cpu = curve,
                FanCurvePU::GPU => self.balanced.gpu = curve,
            },
            Profile::Performance => match curve.fan {
                FanCurvePU::CPU => self.performance.cpu = curve,
                FanCurvePU::GPU => self.performance.gpu = curve,
            },
            Profile::Quiet => match curve.fan {
                FanCurvePU::CPU => self.quiet.cpu = curve,
                FanCurvePU::GPU => self.quiet.gpu = curve,
            },
        }
        Ok(())
    }
}
