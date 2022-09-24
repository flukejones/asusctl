use std::path::PathBuf;

use log::{info, warn};

use crate::{
    attr_u8,
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
pub struct AsusPower {
    mains: PathBuf,
    battery: PathBuf,
    usb: Option<PathBuf>,
}

impl AsusPower {
    /// When checking for battery this will look in order:
    /// - if attr `manufacturer` contains `asus`
    /// - if attr `charge_control_end_threshold` exists and `energy_full_design` >= 50 watt
    /// - if syspath end conatins `BAT`
    /// - if attr `type` is `battery` (last resort)
    pub fn new() -> Result<Self> {
        let mut mains = PathBuf::new();
        let mut battery = None;
        let mut usb = None;

        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;
        enumerator.match_subsystem("power_supply").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        for device in enumerator.scan_devices().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("scan_devices failed".into(), err)
        })? {
            if let Some(attr) = device.attribute_value("type") {
                info!("Power: Checking {:?}", device.syspath());
                match attr.to_string_lossy().to_ascii_lowercase().trim() {
                    "mains" => {
                        info!("Found mains power at {:?}", device.sysname());
                        mains = device.syspath().to_path_buf();
                    }
                    "battery" => {
                        // Priortised list of checks
                        info!("Found a battery");
                        if battery.is_none() {
                            info!("Checking battery attributes");
                            if device
                                .attribute_value("charge_control_end_threshold")
                                .is_some()
                            {
                                info!("Found battery power at {:?}, matched charge_control_end_threshold", device.sysname());
                                battery = Some(device.syspath().to_path_buf());
                            } else if device.sysname().to_string_lossy().starts_with("BAT") {
                                info!(
                                    "Found battery power at {:?}, sysfs path ended with BAT<n>",
                                    device.sysname()
                                );
                                battery = Some(device.syspath().to_path_buf());
                            } else {
                                info!(
                                    "Last resort: Found battery power at {:?} using type = Battery",
                                    device.sysname()
                                );
                                battery = Some(device.syspath().to_path_buf());
                            }
                        }
                    }
                    "usb" => {
                        info!("Found USB-C power at {:?}", device.sysname());
                        usb = Some(device.syspath().to_path_buf());
                    }
                    _ => {}
                };
            }
        }

        if let Some(battery) = battery {
            return Ok(Self {
                mains,
                battery,
                usb,
            });
        }

        Err(PlatformError::MissingFunction(
            "Did not find a battery".to_owned(),
        ))
    }

    attr_u8!("charge_control_end_threshold", battery);
    attr_u8!("online", mains);
}
