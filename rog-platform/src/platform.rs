use std::{path::PathBuf, str::FromStr};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use zvariant::Type;

use crate::{
    attr_bool, attr_u8,
    error::{PlatformError, Result},
    to_device,
};

/// The "platform" device provides access to things like:
/// - dgpu_disable
/// - egpu_enable
/// - panel_od
/// - gpu_mux
/// - keyboard_mode, set keyboard RGB mode and speed
/// - keyboard_state, set keyboard power states
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct AsusPlatform {
    path: PathBuf,
    pp_path: PathBuf,
}

impl AsusPlatform {
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

        for device in enumerator.scan_devices().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("scan_devices failed".into(), err)
        })? {
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

    attr_bool!("dgpu_disable", path);
    attr_bool!("egpu_enable", path);
    attr_bool!("panel_od", path);
    attr_u8!("gpu_mux_mode", path);
    // This is technically the same as `platform_profile` since both are tied in-kernel
    attr_u8!("throttle_thermal_policy", path);
    // The acpi platform_profile support
    attr_u8!("platform_profile", pp_path);
}

#[derive(Serialize, Deserialize, Type, Debug, PartialEq, Clone, Copy)]
pub enum GpuMode {
    Discrete,
    Optimus,
    Integrated,
    Egpu,
    Error,
    NotSupported,
}

impl GpuMode {
    pub fn to_mux(&self) -> u8 {
        if *self == Self::Discrete {
            return 0;
        }
        1
    }

    pub fn to_dgpu(&self) -> u8 {
        if *self == Self::Integrated {
            return 1;
        }
        0
    }

    pub fn to_egpu(&self) -> u8 {
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
