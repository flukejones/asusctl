use ctrl_gfx::error::GfxError;
use ctrl_gfx::*;
use ctrl_rog_bios::CtrlRogBios;
use log::{error, info, warn};
use std::io::Write;
use std::iter::FromIterator;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::{error::Error, sync::Arc, sync::Mutex};
use sysfs_class::{PciDevice, SysClass};
use system::{GraphicsDevice, Module, PciBus};
use vendors::{GfxCtrlAction, GfxVendors};
use zbus::dbus_interface;

use crate::*;

pub struct CtrlGraphics {
    bus: PciBus,
    _amd: Vec<GraphicsDevice>,
    _intel: Vec<GraphicsDevice>,
    nvidia: Vec<GraphicsDevice>,
    #[allow(dead_code)]
    other: Vec<GraphicsDevice>,
    initfs_cmd: Option<Command>,
    config: Arc<Mutex<Config>>,
}

trait Dbus {
    fn vendor(&self) -> String;
    fn power(&self) -> String;
    fn set_vendor(&mut self, vendor: String);
    fn notify_gfx(&self, vendor: &str) -> zbus::Result<()>;
    fn notify_action(&self, action: &str) -> zbus::Result<()>;
}

use std::convert::TryInto;

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl Dbus for CtrlGraphics {
    fn vendor(&self) -> String {
        Self::get_vendor().unwrap_or_else(|err| format!("Get vendor failed: {}", err))
    }

    fn power(&self) -> String {
        Self::get_runtime_status().unwrap_or_else(|err| format!("Get power status failed: {}", err))
    }

    fn set_vendor(&mut self, vendor: String) {
        if let Ok(tmp) = GfxVendors::from_str(&vendor) {
            let action = self.set(tmp).unwrap_or_else(|err| {
                warn!("{}", err);
                format!("Failed: {}", err.to_string())
            });
            self.notify_gfx(&vendor)
                .unwrap_or_else(|err| warn!("{}", err));
            self.notify_action(&action)
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    #[dbus_interface(signal)]
    fn notify_gfx(&self, vendor: &str) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    fn notify_action(&self, action: &str) -> zbus::Result<()> {}
}

impl CtrlGraphics {
    pub fn new(config: Arc<Mutex<Config>>) -> std::io::Result<CtrlGraphics> {
        let bus = PciBus::new()?;

        info!("Rescanning PCI bus");
        bus.rescan()?;

        let devs = PciDevice::all()?;

        let functions = |parent: &PciDevice| -> Vec<PciDevice> {
            let mut functions = Vec::new();
            if let Some(parent_slot) = parent.id().split('.').next() {
                for func in devs.iter() {
                    if let Some(func_slot) = func.id().split('.').next() {
                        if func_slot == parent_slot {
                            info!("{}: Function for {}", func.id(), parent.id());
                            functions.push(func.clone());
                        }
                    }
                }
            }
            functions
        };

        let mut amd = Vec::new();
        let mut intel = Vec::new();
        let mut nvidia = Vec::new();
        let mut other = Vec::new();
        for dev in devs.iter() {
            let c = dev.class()?;
            if 0x03 == (c >> 16) & 0xFF {
                match dev.vendor()? {
                    0x1002 => {
                        info!("{}: AMD graphics", dev.id());
                        amd.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    0x10DE => {
                        info!("{}: NVIDIA graphics", dev.id());
                        nvidia.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    0x8086 => {
                        info!("{}: Intel graphics", dev.id());
                        intel.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    vendor => {
                        info!("{}: Other({:X}) graphics", dev.id(), vendor);
                        other.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                }
            }
        }

        let mut initfs_cmd = None;

        if Path::new(INITRAMFS_PATH).exists() {
            let mut cmd = Command::new("update-initramfs");
            cmd.arg("-u");
            initfs_cmd = Some(cmd);
            info!("Using initramfs update command 'update-initramfs'");
        }
        // } else if Path::new(DRACUT_PATH).exists() {
        //     let mut cmd = Command::new("dracut");
        //     cmd.arg("-f");
        //     initfs_cmd = Some(cmd);
        //     info!("Using initramfs update command 'dracut'");
        // }

        Ok(CtrlGraphics {
            bus,
            _amd: amd,
            _intel: intel,
            nvidia,
            other,
            initfs_cmd,
            config,
        })
    }

    pub fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &"/org/asuslinux/Gfx".try_into().expect("Some fail here"),
                self,
            )
            .map_err(|err| {
                warn!("CtrlGraphics: add_to_server {}", err);
                err
            })
            .ok();
    }

    pub fn reload(&mut self) -> Result<(), Box<dyn Error>> {
        self.auto_power()?;
        info!("Reloaded gfx mode: {:?}", CtrlGraphics::get_vendor()?);
        Ok(())
    }

    fn get_prime_discrete() -> Result<String, GfxError> {
        let s = std::fs::read_to_string(PRIME_DISCRETE_PATH)
            .map_err(|err| GfxError::Read(PRIME_DISCRETE_PATH.into(), err))?
            .trim()
            .to_owned();
        Ok(s)
    }

    fn set_prime_discrete(mode: &str) -> Result<(), GfxError> {
        std::fs::write(PRIME_DISCRETE_PATH, mode)
            .map_err(|err| GfxError::Read(PRIME_DISCRETE_PATH.into(), err))?;
        Ok(())
    }

    /// Associated method to get which vendor mode is set
    pub fn get_vendor() -> Result<String, GfxError> {
        let mode = match Self::get_prime_discrete() {
            Ok(m) => m,
            Err(_) => "nvidia".to_string(),
        };
        let modules = Module::all().map_err(|err| GfxError::Read("get_vendor".into(), err))?;

        let driver_loaded = if modules
            .iter()
            .any(|module| module.name == "nouveau" || module.name == "nvidia")
        {
            true
        } else {
            false
        };

        let vendor = if mode == "off" {
            if driver_loaded {
                info!("dGPU driver loaded for compute mode");
                "compute".to_string()
            } else {
                info!("No dGPU driver loaded");
                "integrated".to_string()
            }
        } else {
            info!("Assuming dGPU driver loaded");
            if mode == "on-demand" {
                "hybrid".to_string()
            } else {
                "nvidia".to_string()
            }
        };

        Ok(vendor)
    }

    pub fn is_switching_prime_modes(vendor: &GfxVendors) -> Result<bool, GfxError> {
        let prev_mode = GfxVendors::from_str(&Self::get_vendor()?)?;
        let x = (prev_mode == GfxVendors::Hybrid || prev_mode == GfxVendors::Nvidia)
            && (*vendor == GfxVendors::Hybrid || *vendor == GfxVendors::Nvidia);
        Ok(x)
    }

    pub fn set_gfx_config(vendor: GfxVendors) -> Result<(), GfxError> {
        let mode = if vendor == GfxVendors::Hybrid {
            "on-demand\n"
        } else if vendor == GfxVendors::Nvidia {
            "on\n"
        } else {
            // Integrated or Compute
            "off\n"
        };

        info!("Setting {} to {}", PRIME_DISCRETE_PATH, mode);
        Self::set_prime_discrete(mode)?;

        {
            info!("Writing {}", MODPROBE_PATH);

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(MODPROBE_PATH)
                .map_err(|err| GfxError::Path(MODPROBE_PATH.into(), err))?;

            let text = if vendor == GfxVendors::Hybrid {
                MODPROBE_HYBRID
            } else if vendor == GfxVendors::Compute {
                MODPROBE_COMPUTE
            } else if vendor == GfxVendors::Nvidia {
                MODPROBE_NVIDIA
            } else {
                MODPROBE_INTEGRATED
            };

            file.write_all(text)
                .and_then(|_| file.sync_all())
                .map_err(|err| GfxError::Write(MODPROBE_PATH.into(), err))?;
        }

        info!("Writing {}", PRIMARY_GPU_XORG_PATH);

        // begin section for non-separated Nvidia xorg modules
        // eg, not put in their own directory
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(PRIMARY_GPU_XORG_PATH)
            .map_err(|err| GfxError::Write(PRIMARY_GPU_XORG_PATH.into(), err))?;

        let text = if vendor == GfxVendors::Nvidia {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_NVIDIA, PRIMARY_GPU_END].concat()
        } else {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_END].concat()
        };

        file.write_all(&text)
            .and_then(|_| file.sync_all())
            .map_err(|err| GfxError::Write(MODPROBE_PATH.into(), err))?;

        let action = if vendor == GfxVendors::Nvidia {
            info!("Enabling nvidia-fallback.service");
            "enable"
        } else {
            info!("Disabling nvidia-fallback.service");
            "disable"
        };

        let status = Command::new("systemctl")
            .arg(action)
            .arg("nvidia-fallback.service")
            .status()
            .map_err(|err| GfxError::Command("systemctl".into(), err))?;

        if !status.success() {
            // Error is ignored in case this service is removed
            warn!(
                "systemctl: {} (ignore warning if service does not exist!)",
                status
            );
        }
        Ok(())
    }

    /// Write out config files if required, enable/disable relevant services, and update the ramdisk
    pub fn set(&mut self, vendor: GfxVendors) -> Result<String, GfxError> {
        if CtrlRogBios::has_dedicated_gfx_toggle() {
            if let Ok(config) = self.config.clone().try_lock() {
                // Switch to dedicated if config says to do so
                if config.gfx_nv_mode_is_dedicated && vendor == GfxVendors::Nvidia {
                    CtrlRogBios::set_gfx_mode(true)
                        .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
                } else if let Ok(ded) = CtrlRogBios::get_gfx_mode() {
                    // otherwise if switching to non-Nvidia mode turn off dedicated mode
                    if ded == 1 && vendor != GfxVendors::Nvidia {
                        CtrlRogBios::set_gfx_mode(false)
                            .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
                    }
                }
            }
        }

        // Switching from hybrid to/from nvidia shouldn't require a ramdisk update
        // or a reboot.
        let no_reboot = Self::is_switching_prime_modes(&vendor)?;

        Self::set_gfx_config(vendor)?;

        let mut required_action = GfxCtrlAction::None;
        if !no_reboot {
            info!("Updating initramfs");
            if let Some(cmd) = self.initfs_cmd.as_mut() {
                let status = cmd
                    .status()
                    .map_err(|err| GfxError::Write(format!("{:?}", cmd), err))?;
                if !status.success() {
                    error!("Ram disk update failed");
                } else {
                    info!("Successfully updated iniramfs");
                }
            }
            required_action = GfxCtrlAction::Reboot;
        } else if no_reboot {
            required_action = GfxCtrlAction::RestartX;
        }

        Ok(required_action.into())
    }

    pub fn get_runtime_status() -> Result<String, GfxError> {
        const PATH: &str = "/sys/bus/pci/devices/0000:01:00.0/power/runtime_status";
        let buf = std::fs::read_to_string(PATH).map_err(|err| GfxError::Read(PATH.into(), err))?;
        Ok(buf)
    }

    fn set_power(&self, power: bool) -> Result<(), GfxError> {
        if power {
            info!("Enabling graphics power");
            self.bus
                .rescan()
                .map_err(|err| GfxError::Bus("bus rescan error".into(), err))?;
        } else {
            info!("Disabling graphics power");

            // Unbind NVIDIA graphics devices and their functions
            let unbinds = self.nvidia.iter().map(|dev| dev.unbind());

            // Remove NVIDIA graphics devices and their functions
            let removes = self.nvidia.iter().map(|dev| dev.remove());

            Result::from_iter(unbinds.chain(removes))
                .map_err(|err| GfxError::Command("device unbind error".into(), err))?;
        }

        Ok(())
    }

    fn auto_power(&self) -> Result<(), GfxError> {
        let vendor = CtrlGraphics::get_vendor()?;
        self.set_power(vendor != "integrated")
    }
}
