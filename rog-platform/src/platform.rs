use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use log::{info, warn};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use zbus::zvariant::Type;

use crate::error::{PlatformError, Result};
use crate::supported::PlatformSupportedFunctions;
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
pub struct AsusPlatform {
    path: PathBuf,
    pp_path: PathBuf,
}

impl AsusPlatform {
    attr_bool!("dgpu_disable", path);

    attr_bool!("egpu_enable", path);

    attr_bool!("panel_od", path);

    attr_bool!("mini_led_mode", path);

    attr_bool!("gpu_mux_mode", path);

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

impl Default for AsusPlatform {
    fn default() -> Self {
        unsafe {
            Self {
                path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked(),
                pp_path: PathBuf::from_str("/this_shouldNeVErr_exisid").unwrap_unchecked(),
            }
        }
    }
}

impl From<AsusPlatform> for PlatformSupportedFunctions {
    fn from(a: AsusPlatform) -> Self {
        PlatformSupportedFunctions {
            post_sound: false,
            gpu_mux: a.has_gpu_mux_mode(),
            panel_overdrive: a.has_panel_od(),
            dgpu_disable: a.has_dgpu_disable(),
            egpu_enable: a.has_egpu_enable(),
            mini_led_mode: a.has_mini_led_mode(),
            ppt_pl1_spl: a.has_ppt_pl1_spl(),
            ppt_pl2_sppt: a.has_ppt_pl2_sppt(),
            ppt_fppt: a.has_ppt_fppt(),
            ppt_apu_sppt: a.has_ppt_apu_sppt(),
            ppt_platform_sppt: a.has_ppt_platform_sppt(),
            nv_dynamic_boost: a.has_nv_dynamic_boost(),
            nv_temp_target: a.has_nv_temp_target(),
        }
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Default, Type, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GpuMode {
    Discrete,
    Optimus,
    Integrated,
    Egpu,
    Vfio,
    Ultimate,
    #[default]
    Error,
    NotSupported,
}

impl GpuMode {
    /// For writing to `gpu_mux_mode` attribute
    pub fn to_mux_attr(&self) -> bool {
        if *self == Self::Discrete {
            return false;
        }
        true
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
