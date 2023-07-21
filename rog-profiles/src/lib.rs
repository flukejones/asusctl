pub mod error;
pub mod fan_curve_set;

use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use error::ProfileError;
use fan_curve_set::CurveData;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use typeshare::typeshare;
use udev::Device;
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;

pub const PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
pub const PLATFORM_PROFILES: &str = "/sys/firmware/acpi/platform_profile_choices";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
        let buf = fs::read_to_string(PLATFORM_PROFILE)?;
        Ok(buf.as_str().into())
    }

    pub fn get_profile_names() -> Result<Vec<Profile>, ProfileError> {
        let buf = fs::read_to_string(PLATFORM_PROFILES)?;
        Ok(buf.rsplit(' ').map(|p| p.into()).collect())
    }

    pub fn set_profile(profile: Profile) -> Result<(), ProfileError> {
        let mut file = OpenOptions::new().write(true).open(PLATFORM_PROFILE)?;
        file.write_all(<&str>::from(profile).as_bytes())?;
        Ok(())
    }

    pub fn from_throttle_thermal_policy(num: u8) -> Self {
        match num {
            1 => Self::Performance,
            2 => Self::Quiet,
            _ => Self::Balanced,
        }
    }

    pub fn get_next_profile(current: Profile) -> Profile {
        // Read first just incase the user has modified the config before calling this
        match current {
            Profile::Balanced => Profile::Performance,
            Profile::Performance => Profile::Quiet,
            Profile::Quiet => Profile::Balanced,
        }
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

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Deserialize, Serialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FanCurvePU {
    CPU,
    GPU,
    MID,
}

impl FanCurvePU {
    fn which_fans(device: &Device) -> Vec<Self> {
        let mut fans = Vec::with_capacity(3);
        for fan in [Self::CPU, Self::GPU, Self::MID] {
            let pwm_num: char = fan.into();
            let pwm_enable = format!("pwm{pwm_num}_enable");
            debug!("Looking for {pwm_enable}");
            for attr in device.attributes() {
                let tmp = attr.name().to_string_lossy();
                if tmp.contains(&pwm_enable) {
                    debug!("Found {pwm_enable}");
                    fans.push(fan);
                }
            }
        }
        fans
    }
}

impl From<FanCurvePU> for &str {
    fn from(pu: FanCurvePU) -> &'static str {
        match pu {
            FanCurvePU::CPU => "cpu",
            FanCurvePU::GPU => "gpu",
            FanCurvePU::MID => "mid",
        }
    }
}

impl From<FanCurvePU> for char {
    fn from(pu: FanCurvePU) -> char {
        match pu {
            FanCurvePU::CPU => '1',
            FanCurvePU::GPU => '2',
            FanCurvePU::MID => '3',
        }
    }
}

impl std::str::FromStr for FanCurvePU {
    type Err = ProfileError;

    fn from_str(fan: &str) -> Result<Self, Self::Err> {
        match fan.to_ascii_lowercase().trim() {
            "cpu" => Ok(FanCurvePU::CPU),
            "gpu" => Ok(FanCurvePU::GPU),
            "mid" => Ok(FanCurvePU::MID),
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
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveProfiles {
    pub balanced: Vec<CurveData>,
    pub performance: Vec<CurveData>,
    pub quiet: Vec<CurveData>,
}

impl FanCurveProfiles {
    pub fn get_device() -> Result<Device, ProfileError> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("hwmon")?;

        for device in enumerator.scan_devices()? {
            if let Some(name) = device.attribute_value("name") {
                if name == "asus_custom_fan_curve" {
                    debug!("asus_custom_fan_curve found");
                    return Ok(device);
                }
            }
        }
        Err(ProfileError::NotSupported)
    }

    /// Return an array of `FanCurvePU`. An empty array indicates no support for
    /// Curves.
    pub fn supported_fans() -> Result<Vec<FanCurvePU>, ProfileError> {
        let device = Self::get_device()?;
        Ok(FanCurvePU::which_fans(&device))
    }

    ///
    pub fn read_from_dev_profile(
        &mut self,
        profile: Profile,
        device: &Device,
    ) -> Result<(), ProfileError> {
        let fans = Self::supported_fans()?;
        let mut curves = Vec::with_capacity(3);

        for fan in fans {
            let mut curve = CurveData {
                fan,
                ..Default::default()
            };
            debug!("Reading curve for {fan:?}");
            curve.read_from_device(device);
            debug!("Curve: {curve:?}");
            curves.push(curve);
        }

        match profile {
            Profile::Balanced => self.balanced = curves,
            Profile::Performance => self.performance = curves,
            Profile::Quiet => self.quiet = curves,
        }
        Ok(())
    }

    /// Reset the stored (self) and device curve to the defaults of the
    /// platform.
    ///
    /// Each `platform_profile` has a different default and the defualt can be
    /// read only for the currently active profile.
    pub fn set_active_curve_to_defaults(
        &mut self,
        profile: Profile,
        device: &mut Device,
    ) -> Result<(), ProfileError> {
        let fans = Self::supported_fans()?;
        // Do reset for all
        for fan in fans {
            let pwm_num: char = fan.into();
            let pwm = format!("pwm{pwm_num}_enable");
            device.set_attribute_value(&pwm, "3")?;
        }
        self.read_from_dev_profile(profile, device)?;
        Ok(())
    }

    /// Write the curves for the selected profile to the device. If the curve is
    /// in the enabled list it will become active. If the curve is zeroed it
    /// will be initialised to a default read from the system.
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
        for fan in fans {
            debug!("write_profile_curve_to_platform: writing profile:{profile}, {fan:?}");
            fan.write_to_device(device)?;
        }
        Ok(())
    }

    pub fn set_profile_curve_enabled(&mut self, profile: Profile, enabled: bool) {
        match profile {
            Profile::Balanced => {
                for curve in self.balanced.iter_mut() {
                    curve.enabled = enabled;
                }
            }
            Profile::Performance => {
                for curve in self.performance.iter_mut() {
                    curve.enabled = enabled;
                }
            }
            Profile::Quiet => {
                for curve in self.quiet.iter_mut() {
                    curve.enabled = enabled;
                }
            }
        }
    }

    pub fn get_fan_curves_for(&self, name: Profile) -> &[CurveData] {
        match name {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
        }
    }

    pub fn get_fan_curve_for(&self, name: &Profile, pu: FanCurvePU) -> Option<&CurveData> {
        match name {
            Profile::Balanced => {
                for this_curve in self.balanced.iter() {
                    if this_curve.fan == pu {
                        return Some(this_curve);
                    }
                }
            }
            Profile::Performance => {
                for this_curve in self.performance.iter() {
                    if this_curve.fan == pu {
                        return Some(this_curve);
                    }
                }
            }
            Profile::Quiet => {
                for this_curve in self.quiet.iter() {
                    if this_curve.fan == pu {
                        return Some(this_curve);
                    }
                }
            }
        }
        None
    }

    pub fn save_fan_curve(&mut self, curve: CurveData, profile: Profile) -> std::io::Result<()> {
        match profile {
            Profile::Balanced => {
                for this_curve in self.balanced.iter_mut() {
                    if this_curve.fan == curve.fan {
                        *this_curve = curve;
                        break;
                    }
                }
            }
            Profile::Performance => {
                for this_curve in self.performance.iter_mut() {
                    if this_curve.fan == curve.fan {
                        *this_curve = curve;
                        break;
                    }
                }
            }
            Profile::Quiet => {
                for this_curve in self.quiet.iter_mut() {
                    if this_curve.fan == curve.fan {
                        *this_curve = curve;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
