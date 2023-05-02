use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use log::{info, warn};

use crate::error::{PlatformError, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct HidRaw(PathBuf);

impl HidRaw {
    pub fn new(id_product: &str) -> Result<Self> {
        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;

        enumerator.match_subsystem("hidraw").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        for device in enumerator
            .scan_devices()
            .map_err(|e| PlatformError::IoPath("enumerator".to_owned(), e))?
        {
            if let Some(parent) = device
                .parent_with_subsystem_devtype("usb", "usb_device")
                .map_err(|e| {
                    PlatformError::IoPath(device.devpath().to_string_lossy().to_string(), e)
                })?
            {
                if let Some(parent) = parent.attribute_value("idProduct") {
                    if parent == id_product {
                        if let Some(dev_node) = device.devnode() {
                            info!("Using device at: {:?} for hidraw control", dev_node);
                            return Ok(Self(dev_node.to_owned()));
                        }
                    }
                }
            } else {
                // Try to see if there is a virtual device created with uhid for testing
                let dev_path = device.devpath().to_string_lossy();
                dbg!(&dev_path);
                if dev_path.contains("virtual") && dev_path.contains(&id_product.to_uppercase()) {
                    if let Some(dev_node) = device.devnode() {
                        info!("Using device at: {:?} for asdfgsadfgh control", dev_node);
                        return Ok(Self(dev_node.to_owned()));
                    }
                }
            }
        }
        Err(PlatformError::MissingFunction(format!(
            "hidraw dev {} not found",
            id_product
        )))
    }

    pub fn write_bytes(&self, message: &[u8]) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.0)
            .map_err(|e| PlatformError::IoPath(self.0.to_string_lossy().to_string(), e))?;
        // println!("write: {:02x?}", &message);
        file.write_all(message)
            .map_err(|e| PlatformError::IoPath(self.0.to_string_lossy().to_string(), e))
    }
}
