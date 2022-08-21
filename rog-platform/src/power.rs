use std::path::PathBuf;

use log::warn;

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
    pub fn new() -> Result<Self> {
        let mut mains = PathBuf::new();
        let mut battery = PathBuf::new();
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
                match attr.to_os_string().as_os_str().to_str() {
                    Some("Mains") => mains = device.syspath().to_path_buf(),
                    Some("Battery") => battery = device.syspath().to_path_buf(),
                    Some("USB") => usb = Some(device.syspath().to_path_buf()),
                    _ => {}
                };
            }
        }

        Ok(Self {
            mains,
            battery,
            usb,
        })
    }

    attr_u8!("charge_control_end_threshold", battery);
}
