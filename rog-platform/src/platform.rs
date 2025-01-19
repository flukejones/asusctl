use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use log::{info, warn};
use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::error::{PlatformError, Result};
use crate::{attr_string, to_device};

/// The "platform" device provides access to things like:
/// - `dgpu_disable`
/// - `egpu_enable`
/// - `panel_od`
/// - `gpu_mux`
/// - various CPU an GPU tunings
/// - `keyboard_mode`, set keyboard RGB mode and speed
/// - `keyboard_state`, set keyboard power states
#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct RogPlatform {
    path: PathBuf,
    pp_path: PathBuf
}

impl RogPlatform {
    attr_string!(
        /// The acpi platform_profile support
        "platform_profile",
        pp_path
    );

    pub fn new() -> Result<Self> {
        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;
        enumerator.match_subsystem("platform").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;
        enumerator.match_sysname("asus-nb-wmi").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        if let Some(device) = (enumerator.scan_devices().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("scan_devices failed".into(), err)
        })?)
        .next()
        {
            info!("Found platform support at {:?}", device.sysname());
            return Ok(Self {
                path: device.syspath().to_owned(),
                pp_path: PathBuf::from_str("/sys/firmware/acpi").unwrap()
            });
        }
        Err(PlatformError::MissingFunction(
            "asus-nb-wmi not found".into()
        ))
    }
}

impl Default for RogPlatform {
    fn default() -> Self {
        unsafe {
            Self {
                path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked(),
                pp_path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked()
            }
        }
    }
}

#[repr(u8)]
#[derive(
    Serialize, Deserialize, Default, Type, Value, OwnedValue, Debug, PartialEq, Eq, Clone, Copy,
)]
pub enum GpuMode {
    Optimus = 0,
    Integrated = 1,
    Egpu = 2,
    Vfio = 3,
    Ultimate = 4,
    #[default]
    Error = 254,
    NotSupported = 255
}

impl From<u8> for GpuMode {
    fn from(v: u8) -> Self {
        match v {
            0 => GpuMode::Optimus,
            1 => GpuMode::Integrated,
            2 => GpuMode::Egpu,
            3 => GpuMode::Vfio,
            4 => GpuMode::Ultimate,
            5 => GpuMode::Error,
            _ => GpuMode::NotSupported
        }
    }
}

impl From<GpuMode> for u8 {
    fn from(v: GpuMode) -> Self {
        v as u8
    }
}

impl GpuMode {
    /// For writing to `gpu_mux_mode` attribute
    pub fn to_mux_attr(&self) -> u8 {
        if *self == Self::Ultimate {
            return 0;
        }
        1
    }

    pub fn to_dgpu_attr(&self) -> u8 {
        if *self == Self::Integrated {
            return 1;
        }
        0
    }

    pub fn to_egpu_attr(&self) -> u8 {
        if *self == Self::Egpu {
            return 1;
        }
        0
    }

    pub fn from_mux(num: u8) -> Self {
        if num == 0 {
            return Self::Ultimate;
        }
        Self::Optimus
    }

    pub fn from_dgpu(num: u8) -> Self {
        if num == 1 {
            return Self::Integrated;
        }
        Self::Optimus
    }

    // `from_dgpu()` should be called also, and should take precedence if result
    // are not equal.
    pub fn from_egpu(num: u8) -> Self {
        if num == 1 {
            return Self::Egpu;
        }
        Self::Optimus
    }
}

impl Display for GpuMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuMode::Optimus => write!(f, "Optimus"),
            GpuMode::Integrated => write!(f, "Integrated"),
            GpuMode::Egpu => write!(f, "eGPU"),
            GpuMode::Vfio => write!(f, "VFIO"),
            GpuMode::Ultimate => write!(f, "Ultimate"),
            GpuMode::Error => write!(f, "Error"),
            GpuMode::NotSupported => write!(f, "Not Supported")
        }
    }
}

#[repr(u32)]
#[derive(
    Deserialize,
    Serialize,
    Default,
    Type,
    Value,
    OwnedValue,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Hash,
    Clone,
    Copy,
)]
#[zvariant(signature = "u")]
/// `platform_profile` in asus_wmi
pub enum PlatformProfile {
    #[default]
    Balanced = 0,
    Performance = 1,
    Quiet = 2
}

impl PlatformProfile {
    pub const fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Performance,
            Self::Performance => Self::Quiet,
            Self::Quiet => Self::Balanced
        }
    }

    pub const fn list() -> [Self; 3] {
        [
            Self::Balanced,
            Self::Performance,
            Self::Quiet
        ]
    }
}

impl From<u8> for PlatformProfile {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::Balanced,
            1 => Self::Performance,
            2 => Self::Quiet,
            _ => {
                warn!("Unknown number for PlatformProfile: {}", num);
                Self::Balanced
            }
        }
    }
}

impl From<i32> for PlatformProfile {
    fn from(num: i32) -> Self {
        (num as u8).into()
    }
}

impl From<PlatformProfile> for u8 {
    fn from(p: PlatformProfile) -> Self {
        match p {
            PlatformProfile::Balanced => 0,
            PlatformProfile::Performance => 1,
            PlatformProfile::Quiet => 2
        }
    }
}

impl From<PlatformProfile> for i32 {
    fn from(p: PlatformProfile) -> Self {
        <u8>::from(p) as i32
    }
}

impl From<PlatformProfile> for &str {
    fn from(profile: PlatformProfile) -> &'static str {
        match profile {
            PlatformProfile::Balanced => "balanced",
            PlatformProfile::Performance => "performance",
            PlatformProfile::Quiet => "quiet"
        }
    }
}

impl From<String> for PlatformProfile {
    fn from(profile: String) -> Self {
        Self::from(&profile)
    }
}

impl From<&String> for PlatformProfile {
    fn from(profile: &String) -> Self {
        match profile.to_ascii_lowercase().trim() {
            "balanced" => PlatformProfile::Balanced,
            "performance" => PlatformProfile::Performance,
            "quiet" => PlatformProfile::Quiet,
            "low-power" => PlatformProfile::Quiet,
            _ => {
                warn!("{profile} is unknown, using ThrottlePolicy::Balanced");
                PlatformProfile::Balanced
            }
        }
    }
}

impl std::str::FromStr for PlatformProfile {
    type Err = PlatformError;

    fn from_str(profile: &str) -> Result<Self> {
        match profile.to_ascii_lowercase().trim() {
            "balanced" => Ok(PlatformProfile::Balanced),
            "performance" => Ok(PlatformProfile::Performance),
            "quiet" => Ok(PlatformProfile::Quiet),
            "low-power" => Ok(PlatformProfile::Quiet),
            _ => Err(PlatformError::NotSupported)
        }
    }
}

impl Display for PlatformProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// CamelCase names of the properties. Intended for use with DBUS
#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, PartialOrd)]
#[zvariant(signature = "s")]
pub enum Properties {
    ChargeControlEndThreshold,
    DgpuDisable,
    GpuMuxMode,
    PostAnimationSound,
    PanelOd,
    MiniLedMode,
    EgpuEnable,
    ThrottlePolicy
}
