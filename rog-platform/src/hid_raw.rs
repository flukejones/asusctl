use std::cell::UnsafeCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::{info, warn};
use udev::Device;

use crate::error::{PlatformError, Result};

#[derive(Debug)]
pub struct HidRaw {
    devfs_path: UnsafeCell<PathBuf>,
    syspath: PathBuf,
    prod_id: String,
}

impl HidRaw {
    pub fn new(id_product: &str) -> Result<(Self, Device)> {
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
            if let Some(parent_device) = device
                .parent_with_subsystem_devtype("usb", "usb_device")
                .map_err(|e| {
                PlatformError::IoPath(device.devpath().to_string_lossy().to_string(), e)
            })? {
                if let Some(parent) = parent_device.attribute_value("idProduct") {
                    if parent == id_product {
                        if let Some(dev_node) = device.devnode() {
                            info!("Using device at: {:?} for hidraw control", dev_node);
                            return Ok((
                                Self {
                                    devfs_path: UnsafeCell::new(dev_node.to_owned()),
                                    prod_id: id_product.to_string(),
                                    syspath: device.syspath().into(),
                                },
                                parent_device,
                            ));
                        }
                    }
                }
            } else {
                // Try to see if there is a virtual device created with uhid for testing
                let dev_path = device.devpath().to_string_lossy();
                if dev_path.contains("virtual") && dev_path.contains(&id_product.to_uppercase()) {
                    if let Some(dev_node) = device.devnode() {
                        info!(
                            "Using device at: {:?} for <TODO: label control> control",
                            dev_node
                        );
                        return Ok((
                            Self {
                                devfs_path: UnsafeCell::new(dev_node.to_owned()),
                                prod_id: id_product.to_string(),
                                syspath: device.syspath().into(),
                            },
                            device,
                        ));
                    }
                }
            }
        }
        Err(PlatformError::MissingFunction(format!(
            "hidraw dev {} not found",
            id_product
        )))
    }

    pub fn from_device(device: Device) -> Result<Self> {
        if let Some(parent) = device
            .parent_with_subsystem_devtype("usb", "usb_device")
            .map_err(|e| PlatformError::IoPath(device.devpath().to_string_lossy().to_string(), e))?
        {
            if let Some(dev_node) = device.devnode() {
                if let Some(id_product) = parent.attribute_value("idProduct") {
                    return Ok(Self {
                        devfs_path: UnsafeCell::new(dev_node.to_owned()),
                        prod_id: id_product.to_string_lossy().into(),
                        syspath: device.syspath().into(),
                    });
                }
            }
        }
        Err(PlatformError::MissingFunction(
            "hidraw dev no dev path".to_string(),
        ))
    }

    pub fn prod_id(&self) -> &str {
        &self.prod_id
    }

    pub fn devfs_path(&self) -> PathBuf {
        unsafe { &*(self.devfs_path.get()) }.clone()
    }

    pub fn syspath(&self) -> &Path {
        &self.syspath
    }

    pub fn write_bytes(&self, message: &[u8]) -> Result<()> {
        let mut path = unsafe { &*(self.devfs_path.get()) };
        let mut file = match OpenOptions::new().write(true).open(path) {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    "write_bytes failed for {:?}, trying again: {e}",
                    self.devfs_path
                );
                unsafe {
                    *(self.devfs_path.get()) =
                        (*(Self::new(&self.prod_id)?.0.devfs_path.get())).clone();
                    path = &mut *(self.devfs_path.get());
                }
                OpenOptions::new()
                    .write(true)
                    .open(path)
                    .map_err(|e| PlatformError::IoPath(path.to_string_lossy().to_string(), e))?
            }
        };
        file.write_all(message)
            .map_err(|e| PlatformError::IoPath(path.to_string_lossy().to_string(), e))
    }

    pub fn set_wakeup_disabled(&self) -> Result<()> {
        let path = unsafe { &*(self.devfs_path.get()) };
        let mut dev = Device::from_syspath(path)?;
        Ok(dev.set_attribute_value("power/wakeup", "disabled")?)
    }
}
