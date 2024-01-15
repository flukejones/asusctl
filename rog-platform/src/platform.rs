use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use log::{info, warn};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use zbus::zvariant::{OwnedValue, Type, Value};

use crate::error::{PlatformError, Result};
use crate::{attr_bool, attr_string, attr_u8, to_device};

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
    pp_path: PathBuf,
}

impl RogPlatform {
    attr_bool!("dgpu_disable", path);

    attr_bool!("egpu_enable", path);

    attr_bool!("panel_od", path);

    attr_bool!("mini_led_mode", path);

    attr_u8!("gpu_mux_mode", path);

    attr_u8!(
        /// This is technically the same as `platform_profile` since both are
        /// tied in-kernel
        "throttle_thermal_policy",
        path
    );

    attr_string!(
        /// The acpi platform_profile support
        "platform_profile",
        pp_path
    );

    attr_u8!(
        /// Package Power Target total of CPU: PL1 on Intel, SPL on AMD.
        /// Shown on Intel+Nvidia or AMD+Nvidia based systems:
        /// * min=5, max=250
        "ppt_pl1_spl",
        path
    );

    attr_u8!(
        /// Slow Package Power Tracking Limit of CPU: PL2 on Intel, SPPT,
        /// on AMD. Shown on Intel+Nvidia or AMD+Nvidia based systems:
        /// * min=5, max=250
        "ppt_pl2_sppt",
        path
    );

    attr_u8!(
        /// Fast Package Power Tracking Limit of CPU. AMD+Nvidia only:
        /// * min=5, max=250
        "ppt_fppt",
        path
    );

    attr_u8!(
        /// APU SPPT limit. Shown on full AMD systems only:
        /// * min=5, max=130
        "ppt_apu_sppt",
        path
    );

    attr_u8!(
        /// Platform SPPT limit. Shown on full AMD systems only:
        /// * min=5, max=130
        "ppt_platform_sppt",
        path
    );

    attr_u8!(
        /// Dynamic boost limit of the Nvidia dGPU:
        /// * min=5, max=25
        "nv_dynamic_boost",
        path
    );

    attr_u8!(
        /// Target temperature limit of the Nvidia dGPU:
        /// * min=75, max=87
        "nv_temp_target",
        path
    );

    attr_bool!(
        /// Control the POST animation "FWOOoosh" sound
        "post_animation_sound",
        path
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
                pp_path: PathBuf::from_str("/sys/firmware/acpi").unwrap(),
            });
        }
        Err(PlatformError::MissingFunction(
            "asus-nb-wmi not found".into(),
        ))
    }
}

impl Default for RogPlatform {
    fn default() -> Self {
        unsafe {
            Self {
                path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked(),
                pp_path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked(),
            }
        }
    }
}

#[typeshare]
#[repr(u8)]
#[derive(
    Serialize, Deserialize, Default, Type, Value, OwnedValue, Debug, PartialEq, Eq, Clone, Copy,
)]
pub enum GpuMode {
    Discrete = 0,
    Optimus = 1,
    Integrated = 2,
    Egpu = 3,
    Vfio = 4,
    Ultimate = 5,
    #[default]
    Error = 6,
    NotSupported = 7,
}

impl From<u8> for GpuMode {
    fn from(v: u8) -> Self {
        match v {
            0 => GpuMode::Discrete,
            1 => GpuMode::Optimus,
            2 => GpuMode::Integrated,
            3 => GpuMode::Egpu,
            4 => GpuMode::Vfio,
            5 => GpuMode::Ultimate,
            6 => GpuMode::Error,
            _ => GpuMode::NotSupported,
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
        if *self == Self::Discrete {
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
            return Self::Discrete;
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
            GpuMode::Discrete => write!(f, "Discrete"),
            GpuMode::Optimus => write!(f, "Optimus"),
            GpuMode::Integrated => write!(f, "Integrated"),
            GpuMode::Egpu => write!(f, "eGPU"),
            GpuMode::Vfio => write!(f, "VFIO"),
            GpuMode::Ultimate => write!(f, "Ultimate"),
            GpuMode::Error => write!(f, "Error"),
            GpuMode::NotSupported => write!(f, "Not Supported"),
        }
    }
}

#[typeshare]
#[repr(u8)]
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
    Ord,
    Hash,
    Clone,
    Copy,
)]
#[zvariant(signature = "s")]
/// `throttle_thermal_policy` in asus_wmi
pub enum ThrottlePolicy {
    #[default]
    Balanced = 0,
    Performance = 1,
    Quiet = 2,
}

impl ThrottlePolicy {
    pub const fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Performance,
            Self::Performance => Self::Quiet,
            Self::Quiet => Self::Balanced,
        }
    }

    pub const fn list() -> [Self; 3] {
        [Self::Balanced, Self::Performance, Self::Quiet]
    }
}

impl From<u8> for ThrottlePolicy {
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

impl From<ThrottlePolicy> for u8 {
    fn from(p: ThrottlePolicy) -> Self {
        match p {
            ThrottlePolicy::Balanced => 0,
            ThrottlePolicy::Performance => 1,
            ThrottlePolicy::Quiet => 2,
        }
    }
}

impl From<ThrottlePolicy> for &str {
    fn from(profile: ThrottlePolicy) -> &'static str {
        match profile {
            ThrottlePolicy::Balanced => "balanced",
            ThrottlePolicy::Performance => "performance",
            ThrottlePolicy::Quiet => "quiet",
        }
    }
}

impl std::str::FromStr for ThrottlePolicy {
    type Err = PlatformError;

    fn from_str(profile: &str) -> Result<Self> {
        match profile.to_ascii_lowercase().trim() {
            "balanced" => Ok(ThrottlePolicy::Balanced),
            "performance" => Ok(ThrottlePolicy::Performance),
            "quiet" => Ok(ThrottlePolicy::Quiet),
            _ => Err(PlatformError::NotSupported),
        }
    }
}

impl Display for ThrottlePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// CamelCase names of the properties. Intended for use with DBUS
#[typeshare]
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
    ThrottlePolicy,
    PptPl1Spl,
    PptPl2Sppt,
    PptFppt,
    PptApuSppt,
    PptPlatformSppt,
    NvDynamicBoost,
    NvTempTarget,
}
