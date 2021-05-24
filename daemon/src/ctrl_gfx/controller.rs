use ::zbus::Connection;
use ctrl_gfx::error::GfxError;
use ctrl_gfx::*;
use ctrl_rog_bios::CtrlRogBios;
use log::{error, info, warn};
use logind_zbus::{
    types::{SessionClass, SessionInfo, SessionState, SessionType},
    ManagerProxy, SessionProxy,
};
use rog_types::gfx_vendors::{GfxPower, GfxRequiredUserAction, GfxVendors};
use std::{io::Write, ops::Add, path::Path, time::Instant};
use std::{iter::FromIterator, thread::JoinHandle};
use std::{process::Command, thread::sleep, time::Duration};
use std::{str::FromStr, sync::mpsc};
use std::{sync::Arc, sync::Mutex};
use sysfs_class::{PciDevice, SysClass};
use system::{GraphicsDevice, PciBus};

use crate::*;

const THREAD_TIMEOUT_MSG: &str = "GFX: thread time exceeded 3 minutes, exiting";

pub struct CtrlGraphics {
    bus: PciBus,
    _amd: Vec<GraphicsDevice>,
    _intel: Vec<GraphicsDevice>,
    nvidia: Vec<GraphicsDevice>,
    #[allow(dead_code)]
    other: Vec<GraphicsDevice>,
    config: Arc<Mutex<Config>>,
    thread_kill: Arc<Mutex<Option<mpsc::Sender<bool>>>>,
}

impl Reloadable for CtrlGraphics {
    fn reload(&mut self) -> Result<(), RogError> {
        self.auto_power()?;
        info!("GFX: Reloaded gfx mode: {:?}", self.get_gfx_mode()?);
        Ok(())
    }
}

impl CtrlGraphics {
    pub fn new(config: Arc<Mutex<Config>>) -> std::io::Result<CtrlGraphics> {
        let bus = PciBus::new()?;
        info!("GFX: Rescanning PCI bus");
        bus.rescan()?;
        let devs = PciDevice::all()?;

        let functions = |parent: &PciDevice| -> Vec<PciDevice> {
            let mut functions = Vec::new();
            if let Some(parent_slot) = parent.id().split('.').next() {
                for func in devs.iter() {
                    if let Some(func_slot) = func.id().split('.').next() {
                        if func_slot == parent_slot {
                            info!("GFX: {}: Function for {}", func.id(), parent.id());
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
                        info!("GFX: {}: AMD graphics", dev.id());
                        amd.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    0x10DE => {
                        info!("GFX: {}: NVIDIA graphics", dev.id());
                        nvidia.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    0x8086 => {
                        info!("GFX: {}: Intel graphics", dev.id());
                        intel.push(GraphicsDevice::new(dev.id().to_owned(), functions(&dev)));
                    }
                    vendor => {
                        info!("GFX: {}: Other({:X}) graphics", dev.id(), vendor);
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
            thread_kill: Arc::new(Mutex::new(None)),
        })
    }

    pub fn bus(&self) -> PciBus {
        self.bus.clone()
    }

    pub fn devices(&self) -> Vec<GraphicsDevice> {
        self.nvidia.clone()
    }

    /// Save the selected `Vendor` mode to config
    fn save_gfx_mode(vendor: GfxVendors, config: Arc<Mutex<Config>>) {
        if let Ok(mut config) = config.lock() {
            config.gfx_mode = vendor;
            config.write();
        }
    }

    /// Associated method to get which vendor mode is set
    pub fn get_gfx_mode(&self) -> Result<GfxVendors, RogError> {
        if let Ok(config) = self.config.lock() {
            if let Some(mode) = config.gfx_tmp_mode {
                return Ok(mode);
            }
            return Ok(config.gfx_mode);
        }
        // TODO: Error here
        Ok(GfxVendors::Hybrid)
    }

    pub(super) fn get_runtime_status() -> Result<GfxPower, RogError> {
        let path = Path::new("/sys/bus/pci/devices/0000:01:00.0/power/runtime_status");
        if path.exists() {
            let buf = std::fs::read_to_string(path).map_err(|err| {
                RogError::Read(
                    "/sys/bus/pci/devices/0000:01:00.0/power/runtime_status".to_string(),
                    err,
                )
            })?;
            Ok(GfxPower::from_str(&buf)?)
        } else {
            Ok(GfxPower::Off)
        }
    }

    /// Some systems have a fallback service to load nouveau if nvidia fails
    fn toggle_fallback_service(vendor: GfxVendors) -> Result<(), RogError> {
        let action = if vendor == GfxVendors::Nvidia {
            info!("GFX: Enabling nvidia-fallback.service");
            "enable"
        } else {
            info!("GFX: Disabling nvidia-fallback.service");
            "disable"
        };

        let status = Command::new("systemctl")
            .arg(action)
            .arg("nvidia-fallback.service")
            .status()
            .map_err(|err| RogError::Command("systemctl".into(), err))?;

        if !status.success() {
            // Error is ignored in case this service is removed
            warn!(
                "systemctl: {} (ignore warning if service does not exist!)",
                status
            );
        }

        Ok(())
    }

    /// Write the appropriate xorg config for the chosen mode
    fn write_xorg_conf(vendor: GfxVendors) -> Result<(), RogError> {
        let text = if vendor == GfxVendors::Nvidia {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_NVIDIA, PRIMARY_GPU_END].concat()
        } else {
            [PRIMARY_GPU_BEGIN, PRIMARY_GPU_END].concat()
        };

        if !Path::new(XORG_PATH).exists() {
            std::fs::create_dir(XORG_PATH).map_err(|err| RogError::Write(XORG_PATH.into(), err))?;
        }

        let file = XORG_PATH.to_string().add(XORG_FILE);
        info!("GFX: Writing {}", file);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&file)
            .map_err(|err| RogError::Write(file, err))?;

        file.write_all(&text)
            .and_then(|_| file.sync_all())
            .map_err(|err| RogError::Write(MODPROBE_PATH.into(), err))?;
        Ok(())
    }

    /// Creates the full modprobe.conf required for vfio pass-through
    fn get_vfio_conf(devices: &[GraphicsDevice]) -> Vec<u8> {
        let mut vifo = MODPROBE_VFIO.to_vec();
        for (d_count, dev) in devices.iter().enumerate() {
            for (f_count, func) in dev.functions().iter().enumerate() {
                let vendor = func.vendor().unwrap();
                let device = func.device().unwrap();
                unsafe {
                    vifo.append(format!("{:x}", vendor).as_mut_vec());
                }
                vifo.append(&mut vec![b':']);
                unsafe {
                    vifo.append(format!("{:x}", device).as_mut_vec());
                }
                if f_count < dev.functions().len() - 1 {
                    vifo.append(&mut vec![b',']);
                }
            }
            if d_count < dev.functions().len() - 1 {
                vifo.append(&mut vec![b',']);
            }
        }
        let mut conf = MODPROBE_INTEGRATED.to_vec();
        conf.append(&mut vifo);
        conf
    }

    fn write_modprobe_conf(vendor: GfxVendors, devices: &[GraphicsDevice]) -> Result<(), RogError> {
        info!("GFX: Writing {}", MODPROBE_PATH);
        let content = match vendor {
            GfxVendors::Nvidia | GfxVendors::Hybrid => {
                let mut base = MODPROBE_BASE.to_vec();
                base.append(&mut MODPROBE_DRM_MODESET.to_vec());
                base
            }
            GfxVendors::Vfio => Self::get_vfio_conf(devices),
            GfxVendors::Integrated => MODPROBE_INTEGRATED.to_vec(),
            GfxVendors::Compute => MODPROBE_BASE.to_vec(),
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(MODPROBE_PATH)
            .map_err(|err| RogError::Path(MODPROBE_PATH.into(), err))?;

        file.write_all(&content)
            .and_then(|_| file.sync_all())
            .map_err(|err| RogError::Write(MODPROBE_PATH.into(), err))?;

        Ok(())
    }

    fn unbind_remove_nvidia(devices: &[GraphicsDevice]) -> Result<(), RogError> {
        // Unbind NVIDIA graphics devices and their functions
        let unbinds = devices.iter().map(|dev| dev.unbind());
        // Remove NVIDIA graphics devices and their functions
        let removes = devices.iter().map(|dev| dev.remove());
        Result::from_iter(unbinds.chain(removes))
            .map_err(|err| RogError::Command("device unbind error".into(), err))
    }

    fn unbind_only(devices: &[GraphicsDevice]) -> Result<(), RogError> {
        let unbinds = devices.iter().map(|dev| dev.unbind());
        Result::from_iter(unbinds)
            .map_err(|err| RogError::Command("device unbind error".into(), err))
    }

    fn do_driver_action(driver: &str, action: &str) -> Result<(), GfxError> {
        let mut cmd = Command::new(action);
        cmd.arg(driver);

        let mut count = 0;
        const MAX_TRIES: i32 = 6;
        loop {
            if count > MAX_TRIES {
                let msg = format!("{} {} failed for unknown reason", action, driver);
                error!("GFX: {}", msg);
                return Ok(()); //Err(RogError::Modprobe(msg));
            }

            let output = cmd
                .output()
                .map_err(|err| GfxError::Command(format!("{:?}", cmd), err))?;
            if !output.status.success() {
                if output
                    .stderr
                    .ends_with("is not currently loaded\n".as_bytes())
                {
                    return Ok(());
                }
                if output.stderr.ends_with("is builtin.\n".as_bytes()) {
                    return Err(GfxError::VfioBuiltin);
                }
                if output.stderr.ends_with("Permission denied\n".as_bytes()) {
                    warn!(
                        "{} {} failed: {:?}",
                        action,
                        driver,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    warn!("GFX: It may be safe to ignore the above error, run `lsmod |grep {}` to confirm modules loaded", driver);
                    return Ok(());
                }
                if String::from_utf8_lossy(&output.stderr)
                    .contains(&format!("Module {} not found", driver))
                {
                    return Err(GfxError::MissingModule(driver.into()));
                }
                if count >= MAX_TRIES {
                    let msg = format!(
                        "{} {} failed: {:?}",
                        action,
                        driver,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Err(GfxError::Modprobe(msg));
                }
            } else if output.status.success() {
                return Ok(());
            }

            count += 1;
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    fn do_display_manager_action(action: &str) -> Result<(), RogError> {
        let mut cmd = Command::new("systemctl");
        cmd.arg(action);
        cmd.arg(DISPLAY_MANAGER);

        let status = cmd
            .status()
            .map_err(|err| RogError::Command(format!("{:?}", cmd), err))?;
        if !status.success() {
            let msg = format!(
                "systemctl {} {} failed: {:?}",
                action, DISPLAY_MANAGER, status
            );
            return Err(GfxError::DisplayManagerAction(msg, status).into());
        }
        Ok(())
    }

    fn wait_display_manager_state(state: &str) -> Result<(), RogError> {
        let mut cmd = Command::new("systemctl");
        cmd.arg("is-active");
        cmd.arg(DISPLAY_MANAGER);

        let mut count = 0;

        while count <= 5 {
            let output = cmd
                .output()
                .map_err(|err| RogError::Command(format!("{:?}", cmd), err))?;
            if output.stdout.starts_with(state.as_bytes()) {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            count += 1;
        }
        Err(GfxError::DisplayManagerTimeout(state.into()).into())
    }

    /// Determine if we need to logout/thread. Integrated<->Vfio mode does not
    /// require logout.
    fn logout_required(&self, vendor: GfxVendors) -> GfxRequiredUserAction {
        if let Ok(config) = self.config.lock() {
            let current = config.gfx_mode;
            if matches!(
                current,
                GfxVendors::Integrated | GfxVendors::Vfio | GfxVendors::Compute
            ) && matches!(
                vendor,
                GfxVendors::Integrated | GfxVendors::Vfio | GfxVendors::Compute
            ) {
                return GfxRequiredUserAction::None;
            }
        }
        GfxRequiredUserAction::Logout
    }

    /// Write the config changes and add/remove drivers and devices depending
    /// on selected mode:
    ///
    /// Tasks:
    /// - write xorg config
    /// - write modprobe config
    /// - rescan for devices
    ///   + add drivers
    ///   + or remove drivers and devices
    ///
    /// The daemon needs direct access to this function when it detects that the
    pub fn do_vendor_tasks(
        vendor: GfxVendors,
        vfio_enable: bool,
        devices: &[GraphicsDevice],
        bus: &PciBus,
    ) -> Result<(), RogError> {
        // Rescan before doing remove or add drivers
        bus.rescan()?;
        //
        Self::write_xorg_conf(vendor)?;
        // Write different modprobe to enable boot control to work
        Self::write_modprobe_conf(vendor, devices)?;

        match vendor {
            GfxVendors::Nvidia | GfxVendors::Hybrid | GfxVendors::Compute => {
                if vfio_enable {
                    for driver in VFIO_DRIVERS.iter() {
                        Self::do_driver_action(driver, "rmmod")?;
                    }
                }
                for driver in NVIDIA_DRIVERS.iter() {
                    Self::do_driver_action(driver, "modprobe")?;
                }
            }
            GfxVendors::Vfio => {
                if vfio_enable {
                    Self::do_driver_action("nouveau", "rmmod")?;
                    for driver in NVIDIA_DRIVERS.iter() {
                        Self::do_driver_action(driver, "rmmod")?;
                    }
                    Self::unbind_only(&devices)?;
                    Self::do_driver_action("vfio-pci", "modprobe")?;
                } else {
                    return Err(GfxError::VfioDisabled.into());
                }
            }
            GfxVendors::Integrated => {
                Self::do_driver_action("nouveau", "rmmod")?;
                if vfio_enable {
                    for driver in VFIO_DRIVERS.iter() {
                        Self::do_driver_action(driver, "rmmod")?;
                    }
                }
                for driver in NVIDIA_DRIVERS.iter() {
                    Self::do_driver_action(driver, "rmmod")?;
                }
                Self::unbind_remove_nvidia(&devices)?;
            }
        }
        Ok(())
    }

    /// Check if the user has any graphical uiser sessions that are active or online
    fn graphical_user_sessions_exist(
        connection: &Connection,
        sessions: &[SessionInfo],
    ) -> Result<bool, RogError> {
        for session in sessions {
            let session_proxy = SessionProxy::new(connection, session)?;
            if session_proxy.get_class()? == SessionClass::User {
                match session_proxy.get_type()? {
                    SessionType::X11 | SessionType::Wayland | SessionType::MIR => {
                        match session_proxy.get_state()? {
                            SessionState::Online | SessionState::Active => return Ok(true),
                            SessionState::Closing | SessionState::Invalid => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }

    /// Spools until all user sessions are ended then switches to requested mode
    fn fire_starter(
        vendor: GfxVendors,
        devices: Vec<GraphicsDevice>,
        bus: PciBus,
        thread_stop: mpsc::Receiver<bool>,
        config: Arc<Mutex<Config>>,
    ) -> Result<String, RogError> {
        info!("GFX: display-manager thread started");

        const SLEEP_PERIOD: Duration = Duration::from_millis(100);
        let start_time = Instant::now();

        let connection = Connection::new_system()?;
        let manager = ManagerProxy::new(&connection)?;
        let mut sessions = manager.list_sessions()?;

        loop {
            let tmp = manager.list_sessions()?;
            if !tmp.iter().eq(&sessions) {
                info!("GFX thread: Sessions list changed");
                sessions = tmp;
            }

            if !Self::graphical_user_sessions_exist(&connection, &sessions)? {
                break;
            }

            if let Ok(stop) = thread_stop.try_recv() {
                if stop {
                    return Ok("Graphics mode change was cancelled".into());
                }
            }
            // exit if 3 minutes pass
            if Instant::now().duration_since(start_time).as_secs() > 180 {
                warn!("{}", THREAD_TIMEOUT_MSG);
                return Ok(THREAD_TIMEOUT_MSG.into());
            }

            // Don't spin at max speed
            sleep(SLEEP_PERIOD);
        }

        info!("GFX thread: all graphical user sessions ended, continuing");
        Self::do_display_manager_action("stop")?;
        Self::wait_display_manager_state("inactive")?;

        let vfio_enable = if let Ok(mut config) = config.try_lock() {
            // Since we have a lock, reset tmp to none. This thread should only ever run
            // for Integrated, Hybrid, or Nvidia. Tmp is also only for informational
            config.gfx_tmp_mode = None;
            //
            config.gfx_vfio_enable
        } else {
            false
        };

        Self::do_vendor_tasks(vendor, vfio_enable, &devices, &bus)?;
        Self::do_display_manager_action("restart")?;
        // Save selected mode in case of reboot
        Self::save_gfx_mode(vendor, config);
        info!("GFX thread: display-manager started");

        let v: &str = vendor.into();
        info!("GFX thread: Graphics mode changed to {} successfully", v);
        Ok(format!("Graphics mode changed to {} successfully", v))
    }

    /// Before starting a new thread the old one *must* be cancelled
    fn cancel_thread(&self) {
        if let Ok(lock) = self.thread_kill.lock() {
            if let Some(tx) = lock.as_ref() {
                // Cancel the running thread
                info!("GFX: Cancelling previous thread");
                tx.send(true)
                    .map_err(|err| {
                        warn!("GFX thread: {}", err);
                    })
                    .ok();
            }
        }
    }

    /// The thread is used only in cases where a logout is required
    fn setup_thread(&mut self, vendor: GfxVendors) {
        let config = self.config.clone();
        let devices = self.nvidia.clone();
        let bus = self.bus.clone();
        let (tx, rx) = mpsc::channel();
        if let Ok(mut lock) = self.thread_kill.lock() {
            *lock = Some(tx);
        }
        let killer = self.thread_kill.clone();

        let _join: JoinHandle<()> = std::thread::spawn(move || {
            Self::fire_starter(vendor, devices, bus, rx, config)
                .map_err(|err| {
                    error!("GFX: {}", err);
                })
                .ok();
            // clear the tx/rx when done
            if let Ok(mut lock) = killer.try_lock() {
                *lock = None;
            }
        });
    }

    /// Initiates a mode change by starting a thread that will wait until all
    /// graphical sessions are exited before performing the tasks required
    /// to switch modes.
    ///
    /// For manually calling (not on boot/startup) via dbus
    pub fn set_gfx_config(
        &mut self,
        vendor: GfxVendors,
    ) -> Result<GfxRequiredUserAction, RogError> {
        if let Ok(gsync) = CtrlRogBios::get_gfx_mode() {
            if gsync == 1 {
                return Err(GfxError::GsyncModeActive.into());
            }
        }

        let vfio_enable = if let Ok(config) = self.config.try_lock() {
            config.gfx_vfio_enable
        } else {
            false
        };

        if !vfio_enable && matches!(vendor, GfxVendors::Vfio) {
            return Err(GfxError::VfioDisabled.into());
        }

        // Must always cancel any thread running
        self.cancel_thread();
        // determine which method we need here
        let action_required = self.logout_required(vendor);
        if matches!(action_required, GfxRequiredUserAction::Logout) {
            // Yeah need the thread to check if all users are logged out
            info!("GFX: mode change requires a logout to complete");
            self.setup_thread(vendor);
        } else {
            // Okay cool, we can switch on/off vfio
            info!("GFX: mode change does not require logout");
            let devices = self.nvidia.clone();
            let bus = self.bus.clone();
            Self::do_vendor_tasks(vendor, vfio_enable, &devices, &bus)?;
            info!("GFX: Graphics mode changed to {}", <&str>::from(vendor));
            if matches!(vendor, GfxVendors::Vfio | GfxVendors::Compute) {
                if let Ok(mut config) = self.config.try_lock() {
                    config.gfx_tmp_mode = Some(vendor);
                };
            }
        }
        // TODO: undo if failed? Save last mode, catch errors...
        Ok(action_required)
    }

    /// Used only on boot to set correct mode
    fn auto_power(&mut self) -> Result<(), RogError> {
        let vendor = self.get_gfx_mode()?;
        let devices = self.nvidia.clone();
        let bus = self.bus.clone();

        let vfio_enable = if let Ok(config) = self.config.try_lock() {
            config.gfx_vfio_enable
        } else {
            false
        };

        Self::do_vendor_tasks(vendor, vfio_enable, &devices, &bus)?;
        Self::toggle_fallback_service(vendor)?;
        Ok(())
    }
}
