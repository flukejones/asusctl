pub mod error;

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use error::ProfileError;
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "dbus")]
use zvariant_derive::Type;

pub static PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
pub static PLATFORM_PROFILES: &str = "/sys/firmware/acpi/platform_profile_choices";

pub static FAN_CURVE_BASE_PATH: &str = "/sys/devices/platform/asus-nb-wmi/";
pub static FAN_CURVE_ACTIVE_FILE: &str = "enabled_fan_curve_profiles";
pub static FAN_CURVE_FILENAME_PART: &str = "_fan_curve_";

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

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
        file.read_to_string(&mut buf).unwrap();
        Ok(buf.as_str().into())
    }

    pub fn get_profile_names() -> Result<Vec<Profile>, ProfileError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&PLATFORM_PROFILES)
            .unwrap_or_else(|_| panic!("{} not found", &PLATFORM_PROFILES));

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        Ok(buf.rsplit(' ').map(|p| p.into()).collect())
    }

    pub fn set_profile(profile: Profile) {
        let mut file = OpenOptions::new()
            .write(true)
            .open(PLATFORM_PROFILE)
            .unwrap_or_else(|_| panic!("{} not found", PLATFORM_PROFILE));

        file.write_all(<&str>::from(profile).as_bytes()).unwrap();
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

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct FanCurve {
    pub profile: Profile,
    pub cpu: String,
    pub gpu: String,
}

/// Main purpose of `FanCurves` is to enable retoring state on system boot
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Deserialize, Serialize, Debug)]
pub struct FanCurves {
    active_curves: Vec<Profile>,
    balanced: FanCurve,
    performance: FanCurve,
    quiet: FanCurve,
}

impl Default for FanCurves {
    fn default() -> Self {
        let mut curves = Self {
            active_curves: Default::default(),
            balanced: Default::default(),
            performance: Default::default(),
            quiet: Default::default(),
        };
        curves.balanced.profile = Profile::Balanced;
        curves.performance.profile = Profile::Performance;
        curves.quiet.profile = Profile::Quiet;
        curves
    }
}

impl FanCurves {
    pub fn is_fan_curves_supported() -> bool {
        let mut path = PathBuf::new();
        path.push(FAN_CURVE_BASE_PATH);
        path.push(FAN_CURVE_ACTIVE_FILE);
        path.exists()
    }

    pub fn update_from_platform(&mut self) {
        self.balanced.cpu = Self::get_fan_curve_from_file(Profile::Balanced, FanCurvePU::CPU);
        self.balanced.gpu = Self::get_fan_curve_from_file(Profile::Balanced, FanCurvePU::GPU);

        self.performance.cpu = Self::get_fan_curve_from_file(Profile::Performance, FanCurvePU::CPU);
        self.performance.gpu = Self::get_fan_curve_from_file(Profile::Performance, FanCurvePU::GPU);

        self.quiet.cpu = Self::get_fan_curve_from_file(Profile::Quiet, FanCurvePU::CPU);
        self.quiet.gpu = Self::get_fan_curve_from_file(Profile::Quiet, FanCurvePU::GPU);
    }

    pub fn update_platform(&self) {
        Self::set_fan_curve_for_platform(Profile::Balanced, FanCurvePU::CPU, &self.balanced.cpu);
        Self::set_fan_curve_for_platform(Profile::Balanced, FanCurvePU::GPU, &self.balanced.gpu);

        Self::set_fan_curve_for_platform(
            Profile::Performance,
            FanCurvePU::CPU,
            &self.performance.cpu,
        );
        Self::set_fan_curve_for_platform(
            Profile::Performance,
            FanCurvePU::GPU,
            &self.performance.gpu,
        );

        Self::set_fan_curve_for_platform(Profile::Quiet, FanCurvePU::CPU, &self.quiet.cpu);
        Self::set_fan_curve_for_platform(Profile::Quiet, FanCurvePU::GPU, &self.quiet.gpu);
    }

    pub fn get_enabled_curve_names(&self) -> &[Profile] {
        &self.active_curves
    }

    pub fn get_all_fan_curves(&self) -> Vec<FanCurve> {
        vec![
            self.balanced.clone(),
            self.performance.clone(),
            self.quiet.clone(),
        ]
    }

    pub fn get_active_fan_curves(&self) -> &FanCurve {
        match Profile::get_active_profile().unwrap() {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
        }
    }

    pub fn get_fan_curves_for(&self, name: Profile) -> &FanCurve {
        match name {
            Profile::Balanced => &self.balanced,
            Profile::Performance => &self.performance,
            Profile::Quiet => &self.quiet,
        }
    }

    fn get_fan_curve_from_file(name: Profile, pu: FanCurvePU) -> String {
        let mut file: String = FAN_CURVE_BASE_PATH.into();
        file.push_str(pu.into());
        file.push_str(FAN_CURVE_FILENAME_PART);
        file.push_str(name.into());

        let mut file = OpenOptions::new()
            .read(true)
            .open(&file)
            .unwrap_or_else(|_| panic!("{} not found", &file));

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf.trim().to_string()
    }

    pub fn get_fan_curve_for(&self, name: &Profile, pu: &FanCurvePU) -> &str {
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

    fn set_fan_curve_for_platform(name: Profile, pu: FanCurvePU, curve: &str) {
        let mut file: String = FAN_CURVE_BASE_PATH.into();
        file.push_str(pu.into());
        file.push_str(FAN_CURVE_FILENAME_PART);
        file.push_str(name.into());

        let mut file = OpenOptions::new()
            .write(true)
            .open(&file)
            .unwrap_or_else(|_| panic!("{} not found", &file));

        file.write_all(curve.as_bytes()).unwrap();
    }

    pub fn set_fan_curve(&mut self, curve: FanCurve) {
        // First, set the profiles.
        Self::set_fan_curve_for_platform(curve.profile, FanCurvePU::CPU, &curve.cpu);
        match curve.profile {
            Profile::Balanced => self.balanced.cpu = curve.cpu,
            Profile::Performance => self.performance.cpu = curve.cpu,
            Profile::Quiet => self.quiet.cpu = curve.cpu,
        };

        Self::set_fan_curve_for_platform(curve.profile, FanCurvePU::GPU, &curve.gpu);
        match curve.profile {
            Profile::Balanced => self.balanced.gpu = curve.gpu,
            Profile::Performance => self.performance.gpu = curve.gpu,
            Profile::Quiet => self.quiet.cpu = curve.gpu,
        };

        // Any curve that was blank will have been reset, so repopulate the settings
        // Note: successfully set curves will just be re-read in.
        self.update_from_platform();
    }
}
