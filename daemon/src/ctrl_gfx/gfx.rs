use ctrl_gfx::error::GfxError;
use ctrl_gfx::*;
use log::{error, info, warn};
use rog_types::gfx_vendors::GfxVendors;
use std::io::Write;
use std::iter::FromIterator;
use std::process::Command;
use std::str::FromStr;
use std::{sync::Arc, sync::Mutex};
use sysfs_class::{PciDevice, SysClass};
use system::{GraphicsDevice, PciBus};
use zbus::dbus_interface;

use crate::*;

pub struct CtrlGraphics {
    bus: PciBus,
    _amd: Vec<GraphicsDevice>,
    _intel: Vec<GraphicsDevice>,
    nvidia: Vec<GraphicsDevice>,
    #[allow(dead_code)]
    other: Vec<GraphicsDevice>,
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
        self.get_gfx_mode()
            .map(|gfx| gfx.into())
            .unwrap_or_else(|err| format!("Get vendor failed: {}", err))
    }

    fn power(&self) -> String {
        Self::get_runtime_status().unwrap_or_else(|err| format!("Get power status failed: {}", err))
    }

    fn set_vendor(&mut self, vendor: String) {
        if let Ok(tmp) = GfxVendors::from_str(&vendor) {
            let msg = self.set_gfx_config(tmp).unwrap_or_else(|err| {
                warn!("{}", err);
                format!("Failed: {}", err.to_string())
            });
            self.notify_gfx(&vendor)
                .unwrap_or_else(|err| warn!("{}", err));
            self.notify_action(&msg)
                .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    #[dbus_interface(signal)]
    fn notify_gfx(&self, vendor: &str) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    fn notify_action(&self, action: &str) -> zbus::Result<()> {}
}

impl ZbusAdd for CtrlGraphics {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(
                &"/org/asuslinux/Gfx"
                    .try_into()
                    .expect("Couldn't add to zbus"),
                self,
            )
            .map_err(|err| {
                warn!("CtrlGraphics: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl Reloadable for CtrlGraphics {
    fn reload(&mut self) -> Result<(), RogError> {
        self.auto_power()?;
        info!("Reloaded gfx mode: {:?}", self.get_gfx_mode()?);
        Ok(())
    }
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

        Ok(CtrlGraphics {
            bus,
            _amd: amd,
            _intel: intel,
            nvidia,
            other,
            config,
        })
    }

    fn save_gfx_mode(&self, vendor: GfxVendors) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.lock() {
            config.gfx_mode = vendor.clone();
            config.write();
            return Ok(());
        }
        // TODO: Error here
        Ok(())
    }

    /// Associated method to get which vendor mode is set
    pub fn get_gfx_mode(&self) -> Result<GfxVendors, RogError> {
        if let Ok(config) = self.config.lock() {
            return Ok(config.gfx_mode.clone());
        }
        // TODO: Error here
        Ok(GfxVendors::Hybrid)
    }

    fn get_runtime_status() -> Result<String, RogError> {
        const PATH: &str = "/sys/bus/pci/devices/0000:01:00.0/power/runtime_status";
        let buf = std::fs::read_to_string(PATH).map_err(|err| GfxError::Read(PATH.into(), err))?;
        Ok(buf)
    }

    fn toggle_fallback_service(vendor: GfxVendors) -> Result<(), RogError> {
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

    fn write_xorg_conf(vendor: GfxVendors) -> Result<(), RogError> {
        let text = if vendor == GfxVendors::Nvidia {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_NVIDIA, PRIMARY_GPU_END].concat()
        } else {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_END].concat()
        };

        info!("Writing {}", PRIMARY_GPU_XORG_PATH);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(PRIMARY_GPU_XORG_PATH)
            .map_err(|err| GfxError::Write(PRIMARY_GPU_XORG_PATH.into(), err))?;

        file.write_all(&text)
            .and_then(|_| file.sync_all())
            .map_err(|err| GfxError::Write(MODPROBE_PATH.into(), err))?;
        Ok(())
    }

    fn write_modprobe_conf() -> Result<(), RogError> {
        info!("Writing {}", MODPROBE_PATH);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(MODPROBE_PATH)
            .map_err(|err| GfxError::Path(MODPROBE_PATH.into(), err))?;

        file.write_all(MODPROBE_BASE)
            .and_then(|_| file.sync_all())
            .map_err(|err| GfxError::Write(MODPROBE_PATH.into(), err))?;

        Ok(())
    }

    fn unbind_remove_nvidia(&self) -> Result<(), RogError> {
        // Unbind NVIDIA graphics devices and their functions
        let unbinds = self.nvidia.iter().map(|dev| dev.unbind());

        // Remove NVIDIA graphics devices and their functions
        let removes = self.nvidia.iter().map(|dev| dev.remove());

        Result::from_iter(unbinds.chain(removes))
            .map_err(|err| GfxError::Command("device unbind error".into(), err))?;

        Ok(())
    }

    fn do_driver_action(driver: &str, action: &str) -> Result<(), RogError> {
        let mut cmd = Command::new(action);
        cmd.arg(driver);

        let status = cmd
            .status()
            .map_err(|err| GfxError::Command(format!("{:?}", cmd), err))?;
        if !status.success() {
            let msg = format!("{} {} failed: {:?}", action, driver, status);
            error!("{}", msg);
            return Err(GfxError::Modprobe(msg).into());
        }
        Ok(())
    }

    fn do_display_manager_action(action: &str) -> Result<(), RogError> {
        let service = "display-manager.service";
        let mut cmd = Command::new("systemctl");
        cmd.arg(action);
        cmd.arg(service);

        let status = cmd
            .status()
            .map_err(|err| GfxError::Command(format!("{:?}", cmd), err))?;
        if !status.success() {
            let msg = format!("systemctl {} {} failed: {:?}", action, service, status);
            error!("{}", msg);
            return Err(GfxError::DisplayManager(msg).into());
        }
        Ok(())
    }

    fn wait_display_manager_inactive() -> Result<(), RogError> {
        let service = "display-manager.service";
        let mut cmd = Command::new("systemctl");
        cmd.arg("is-active");
        cmd.arg(service);

        let mut count = 0;

        while count <= 4 {
            let output = cmd
                .output()
                .map_err(|err| GfxError::Command(format!("{:?}", cmd), err))?;
            if output.stdout.starts_with("inactive".as_bytes()) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            count += 1;
        }
        return Err(
            GfxError::DisplayManager("display-manager did not completely stop".into()).into(),
        );
    }

    pub fn do_vendor_tasks(&mut self, vendor: GfxVendors) -> Result<(), RogError> {
        Self::write_xorg_conf(vendor)?;
        Self::write_modprobe_conf()?; // TODO: Not required here, should put in startup?

        // Rescan before doing remove or add drivers
        self.bus
            .rescan()
            .map_err(|err| GfxError::Bus("bus rescan error".into(), err))?;

        let drivers = vec!["nvidia_drm", "nvidia_modeset", "nvidia"]; // i2c_nvidia_gpu?

        match vendor {
            GfxVendors::Nvidia | GfxVendors::Hybrid | GfxVendors::Compute => {
                for driver in drivers {
                    Self::do_driver_action(driver, "modprobe")?;
                }
            }
            // TODO: compute mode, needs different setup
            // GfxVendors::Compute => {}
            GfxVendors::Integrated => {
                for driver in drivers {
                    Self::do_driver_action(driver, "rmmod")?;
                }
                self.unbind_remove_nvidia()?;
            }
        }

        self.save_gfx_mode(vendor)?;
        Ok(())
    }

    /// For manually calling (not on boot/startup)
    ///
    /// Will stop and start display manager without warning
    pub fn set_gfx_config(&mut self, vendor: GfxVendors) -> Result<String, RogError> {
        Self::do_display_manager_action("stop")?;
        Self::wait_display_manager_inactive()?;
        self.do_vendor_tasks(vendor)?;
        Self::do_display_manager_action("start")?;
        // TODO: undo if failed? Save last mode, catch errors...
        let v: &str = vendor.into();
        Ok(format!("Graphics mode changed to {} successfully", v))
    }

    // if CtrlRogBios::has_dedicated_gfx_toggle() {
    //     if let Ok(config) = self.config.clone().try_lock() {
    //         // Switch to dedicated if config says to do so
    //         if config.gfx_nv_mode_is_dedicated && vendor == GfxVendors::Nvidia {
    //             CtrlRogBios::set_gfx_mode(true)
    //                 .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
    //         } else if let Ok(ded) = CtrlRogBios::get_gfx_mode() {
    //             // otherwise if switching to non-Nvidia mode turn off dedicated mode
    //             if ded == 1 && vendor != GfxVendors::Nvidia {
    //                 CtrlRogBios::set_gfx_mode(false)
    //                     .unwrap_or_else(|err| warn!("Gfx controller: {}", err));
    //             }
    //         }
    //     }
    // }

    fn auto_power(&mut self) -> Result<(), RogError> {
        let vendor = self.get_gfx_mode()?;
        self.do_vendor_tasks(vendor)?;
        Self::toggle_fallback_service(vendor)?;
        Ok(())
    }
}
