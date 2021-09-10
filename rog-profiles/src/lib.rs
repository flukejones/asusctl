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
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
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
        let mut file = OpenOptions::new()
            .read(true)
            .open(&PLATFORM_PROFILE)
            .unwrap_or_else(|_| panic!("{} not found", &PLATFORM_PROFILE));

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.as_str().into())
    }

    pub fn get_profile_names() -> Result<Vec<Profile>, ProfileError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&PLATFORM_PROFILES)
            .unwrap_or_else(|_| panic!("{} not found", &PLATFORM_PROFILES));

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.rsplit(' ').map(|p| p.into()).collect())
    }

    pub fn set_profile(profile: Profile) -> Result<(), ProfileError> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(PLATFORM_PROFILE)
            .unwrap_or_else(|_| panic!("{} not found", PLATFORM_PROFILE));

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

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
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

impl Default for FanCurvePU {
    fn default() -> Self {
        Self::CPU
    }
}

/// Main purpose of `FanCurves` is to enable retoring state on system boot
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct FanCurveProfiles {
    enabled: Vec<Profile>,
    balanced: FanCurveSet,
    performance: FanCurveSet,
    quiet: FanCurveSet,
}

impl FanCurveProfiles {
    ///
    pub fn read_from_dev_profile(&mut self, profile: Profile, device: &Device) {
        let mut tmp = FanCurveSet::default();
        tmp.read_from_device(device);
        match profile {
            Profile::Balanced => self.balanced = tmp,
            Profile::Performance => self.performance = tmp,
            Profile::Quiet => self.quiet = tmp,
        }
    }

    pub fn write_to_platform(&self, profile: Profile, device: &mut Device) {
        let fans = match profile {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
        };
        fans.write_cpu_fan(device);
        fans.write_gpu_fan(device);
    }

    pub fn get_enabled_curve_profiles(&self) -> &[Profile] {
        &self.enabled
    }

    pub fn set_enabled_curve_profiles(&mut self, profiles: Vec<Profile>) {
        self.enabled = profiles
    }

    pub fn get_all_fan_curves(&self) -> Vec<FanCurveSet> {
        vec![
            self.balanced.clone(),
            self.performance.clone(),
            self.quiet.clone(),
        ]
    }

    pub fn get_active_fan_curves(&self) -> &FanCurveSet {
        match Profile::get_active_profile().unwrap() {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
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

    pub fn write_and_set_fan_curve(
        &mut self,
        curve: CurveData,
        profile: Profile,
        device: &mut Device,
    ) {
        match curve.fan {
            FanCurvePU::CPU => write_to_fan(&curve, '1', device),
            FanCurvePU::GPU => write_to_fan(&curve, '2', device),
        }
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
    }
}

pub fn write_to_fan(curve: &CurveData, pwm_num: char, device: &mut Device) {
    let mut pwm = "pwmN_auto_pointN_pwm".to_string();

    dbg!(&device);
    for (index, out) in curve.pwm.iter().enumerate() {
        unsafe {
            let buf = pwm.as_bytes_mut();
            buf[3] = pwm_num as u8;
            // Should be quite safe to unwrap as we're not going over 8
            buf[15] = char::from_digit(index as u32 + 1, 10).unwrap() as u8;
        }
        let out = out.to_string();
        dbg!(&pwm);
        dbg!(&out);
        device.set_attribute_value(&pwm, &out).unwrap();
    }

    let mut pwm = "pwmN_auto_pointN_temp".to_string();

    for (index, out) in curve.temp.iter().enumerate() {
        unsafe {
            let buf = pwm.as_bytes_mut();
            buf[3] = pwm_num as u8;
            // Should be quite safe to unwrap as we're not going over 8
            buf[15] = char::from_digit(index as u32 + 1, 10).unwrap() as u8;
        }
        let out = out.to_string();
        device.set_attribute_value(&pwm, &out).unwrap();
    }
}
