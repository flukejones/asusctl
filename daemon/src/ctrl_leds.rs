// Only these two packets must be 17 bytes
static LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

static KBD_BRIGHT_PATH: &str = "/sys/class/leds/asus::kbd_backlight/brightness";

use crate::{
    config_aura::AuraConfig,
    error::RogError,
    laptops::{LaptopLedData, ASUS_KEYBOARD_DEVICES},
};
use log::{error, info, warn};
use rog_types::{LED_MSG_LEN, aura_modes::{AuraEffect, AuraModeNum, LedBrightness}};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;

use crate::GetSupported;

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct LedSupportedFunctions {
    pub brightness_set: bool,
    pub stock_led_modes: Option<Vec<AuraModeNum>>,
    pub multizone_led_mode: bool,
    pub per_key_led_mode: bool,
}

impl GetSupported for CtrlKbdBacklight {
    type A = LedSupportedFunctions;

    fn get_supported() -> Self::A {
        // let mode = <&str>::from(&<AuraModes>::from(*mode));
        let multizone_led_mode = false;
        let per_key_led_mode = false;
        let laptop = LaptopLedData::get_data();
        let stock_led_modes = if let Some(data) = laptop {
            if data.standard.is_empty() {
                None
            } else {
                Some(data.standard)
            }
        } else {
            None
        };

        LedSupportedFunctions {
            brightness_set: CtrlKbdBacklight::get_kbd_bright_path().is_some(),
            stock_led_modes,
            multizone_led_mode,
            per_key_led_mode,
        }
    }
}

pub struct CtrlKbdBacklight {
    led_node: Option<String>,
    pub bright_node: String,
    supported_modes: LaptopLedData,
    flip_effect_write: bool,
    config: AuraConfig,
}

pub struct DbusKbdBacklight {
    inner: Arc<Mutex<CtrlKbdBacklight>>,
}

impl DbusKbdBacklight {
    pub fn new(inner: Arc<Mutex<CtrlKbdBacklight>>) -> Self {
        Self { inner }
    }
}

trait Dbus {
    fn set_led(&mut self, data: String);
    fn ledmode(&self) -> String;
    fn notify_led(&self, data: &str) -> zbus::Result<()>;
}

impl crate::ZbusAdd for DbusKbdBacklight {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at("/org/asuslinux/Led", self)
            .map_err(|err| {
                error!("DbusKbdBacklight: add_to_server {}", err);
            })
            .ok();
    }
}

/// The main interface for changing, reading, or notfying signals
///
/// LED commands are split between Brightness, Modes, Per-Key
#[dbus_interface(name = "org.asuslinux.Daemon")]
impl DbusKbdBacklight {
    fn set_brightness(&mut self, brightness: LedBrightness) {
        if let Ok(ctrl) = self.inner.try_lock() {
            ctrl.set_brightness(brightness)
                .map_err(|err| warn!("{}", err))
                .ok();
        }
    }

    fn set_led_mode(&mut self, effect: AuraEffect) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            let mode_name = effect.mode_name();
            match ctrl.do_command(effect) {
                Ok(_) => {
                    self.notify_led(&mode_name).ok();
                }
                Err(err) => {
                    warn!("{}", err);
                }
            }
        }
    }

    fn next_led_mode(&self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.toggle_mode(false)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                if let Ok(json) = serde_json::to_string(&mode) {
                    self.notify_led(&json)
                        .unwrap_or_else(|err| warn!("{}", err));
                }
            }
        }
    }

    fn prev_led_mode(&self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            ctrl.toggle_mode(true)
                .unwrap_or_else(|err| warn!("{}", err));

            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                if let Ok(json) = serde_json::to_string(&mode) {
                    self.notify_led(&json)
                        .unwrap_or_else(|err| warn!("{}", err));
                }
            }
        }
    }

    /// Return the current mode data
    #[dbus_interface(property)]
    fn led_mode(&self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Some(mode) = ctrl.config.builtins.get(&ctrl.config.current_mode) {
                if let Ok(json) = serde_json::to_string(&mode) {
                    return json;
                }
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not deserialise".to_string()
    }

    /// Return a list of available modes
    #[dbus_interface(property)]
    fn led_modes(&self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(json) = serde_json::to_string(&ctrl.config.builtins) {
                return json;
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not serialise".to_string()
    }

    /// Return the current LED brightness
    #[dbus_interface(property)]
    fn led_brightness(&self) -> i8 {
        if let Ok(ctrl) = self.inner.try_lock() {
            return ctrl.get_brightness().map(|n| n as i8).unwrap_or(-1);
        }
        warn!("SetKeyBacklight could not serialise");
        -1
    }

    #[dbus_interface(signal)]
    fn notify_led(&self, data: &str) -> zbus::Result<()>;
}

impl crate::Reloadable for CtrlKbdBacklight {
    fn reload(&mut self) -> Result<(), RogError> {
        // set current mode (if any)
        if self.supported_modes.standard.len() > 1 {
            let current_mode = self.config.current_mode;
            if self.supported_modes.standard.contains(&(current_mode)) {
                let mode = self
                    .config
                    .builtins
                    .get(&current_mode)
                    .ok_or(RogError::NotSupported)?
                    .to_owned();
                self.write_mode(&mode)?;
                info!("Reloaded last used mode");
            } else {
                warn!(
                    "An unsupported mode was set: {}, reset to first mode available",
                    <&str>::from(&self.config.current_mode)
                );
                self.config.builtins.remove(&current_mode);
                self.config.current_mode = AuraModeNum::Static;
                // TODO: do a recursive call with a boxed dyn future later
                let mode = self
                    .config
                    .builtins
                    .get(&current_mode)
                    .ok_or(RogError::NotSupported)?
                    .to_owned();
                self.write_mode(&mode)?;
                info!("Reloaded last used mode");
            }
        }

        // Reload brightness
        let bright = self.config.brightness;
        self.set_brightness(bright)?;
        info!("Reloaded last used brightness");
        Ok(())
    }
}

impl crate::CtrlTask for CtrlKbdBacklight {
    fn do_task(&mut self) -> Result<(), RogError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.bright_node)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => {
                    RogError::MissingLedBrightNode((&self.bright_node).into(), err)
                }
                _ => RogError::Path((&self.bright_node).into(), err),
            })?;
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)
            .map_err(|err| RogError::Read("buffer".into(), err))?;
        if let Some(num) = char::from(buf[0]).to_digit(10) {
            if self.config.brightness != num.into() {
                self.config.read();
                self.config.brightness = num.into();
                self.config.write();
            }
            return Ok(());
        }
        Err(RogError::ParseLED)
    }
}

impl CtrlKbdBacklight {
    #[inline]
    pub fn new(supported_modes: LaptopLedData, config: AuraConfig) -> Result<Self, RogError> {
        // TODO: return error if *all* nodes are None
        let mut led_node = None;
        for prod in ASUS_KEYBOARD_DEVICES.iter() {
            match Self::find_led_node(prod) {
                Ok(node) => {
                    led_node = Some(node);
                    break;
                }
                Err(err) => warn!("led_node: {}", err),
            }
        }

        let bright_node = Self::get_kbd_bright_path();

        if led_node.is_none() && bright_node.is_none() {
            return Err(RogError::MissingFunction(
                "All keyboard features missing, you may require a v5.11 series kernel or newer"
                    .into(),
            ));
        }

        if bright_node.is_none() {
            return Err(RogError::MissingFunction(
                "No brightness control, you may require a v5.11 series kernel or newer".into(),
            ));
        }

        let ctrl = CtrlKbdBacklight {
            led_node,
            bright_node: bright_node.unwrap(), // If was none then we already returned above
            supported_modes,
            flip_effect_write: false,
            config,
        };
        Ok(ctrl)
    }

    fn get_kbd_bright_path() -> Option<String> {
        if Path::new(KBD_BRIGHT_PATH).exists() {
            return Some(KBD_BRIGHT_PATH.to_string());
        }
        None
    }

    pub fn get_brightness(&self) -> Result<u8, RogError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.bright_node)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => {
                    RogError::MissingLedBrightNode((&self.bright_node).into(), err)
                }
                _ => RogError::Path((&self.bright_node).into(), err),
            })?;
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)
            .map_err(|err| RogError::Read("buffer".into(), err))?;
        Ok(buf[0])
    }

    pub fn set_brightness(&self, brightness: LedBrightness) -> Result<(), RogError> {
        let path = Path::new(&self.bright_node);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => {
                    RogError::MissingLedBrightNode((&self.bright_node).into(), err)
                }
                _ => RogError::Path((&self.bright_node).into(), err),
            })?;
        file.write_all(&[brightness.as_char_code()])
            .map_err(|err| RogError::Read("buffer".into(), err))?;
        Ok(())
    }

    fn find_led_node(id_product: &str) -> Result<String, RogError> {
        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            RogError::Udev("enumerator failed".into(), err)
        })?;
        enumerator.match_subsystem("hidraw").map_err(|err| {
            warn!("{}", err);
            RogError::Udev("match_subsystem failed".into(), err)
        })?;

        for device in enumerator.scan_devices().map_err(|err| {
            warn!("{}", err);
            RogError::Udev("scan_devices failed".into(), err)
        })? {
            if let Some(parent) = device
                .parent_with_subsystem_devtype("usb", "usb_device")
                .map_err(|err| {
                    warn!("{}", err);
                    RogError::Udev("parent_with_subsystem_devtype failed".into(), err)
                })?
            {
                if parent
                    .attribute_value("idProduct")
                    .ok_or_else(|| RogError::NotFound("LED idProduct".into()))?
                    == id_product
                {
                    if let Some(dev_node) = device.devnode() {
                        info!("Using device at: {:?} for LED control", dev_node);
                        return Ok(dev_node.to_string_lossy().to_string());
                    }
                }
            }
        }
        Err(RogError::MissingFunction(
            "ASUS LED device node not found".into(),
        ))
    }

    pub(crate) fn do_command(&mut self, mode: AuraEffect) -> Result<(), RogError> {
        self.set_and_save(mode)
    }

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    fn write_bytes(&self, message: &[u8]) -> Result<(), RogError> {
        if let Some(led_node) = &self.led_node {
            if let Ok(mut file) = OpenOptions::new().write(true).open(led_node) {
                // println!("write: {:02x?}", &message);
                return file
                    .write_all(message)
                    .map_err(|err| RogError::Write("write_bytes".into(), err));
            }
        }
        Err(RogError::NotSupported)
    }

    /// Write an effect block
    #[inline]
    fn write_effect(&mut self, effect: &[Vec<u8>]) -> Result<(), RogError> {
        if self.flip_effect_write {
            for row in effect.iter().rev() {
                self.write_bytes(row)?;
            }
        } else {
            for row in effect.iter() {
                self.write_bytes(row)?;
            }
        }
        self.flip_effect_write = !self.flip_effect_write;
        Ok(())
    }

    /// Used to set a builtin mode and save the settings for it
    ///
    /// This needs to be universal so that settings applied by dbus stick
    #[inline]
    fn set_and_save(&mut self, mode: AuraEffect) -> Result<(), RogError> {
        self.config.read();
        self.write_mode(&mode)?;
        self.config.current_mode = *mode.mode();
        self.config.set_builtin(mode);
        self.config.write();
        Ok(())
    }

    #[inline]
    fn toggle_mode(&mut self, reverse: bool) -> Result<(), RogError> {
        let current = self.config.current_mode;
        if let Some(idx) = self
            .supported_modes
            .standard
            .iter()
            .position(|v| *v == current)
        {
            let mut idx = idx;
            // goes past end of array
            if reverse {
                if idx == 0 {
                    idx = self.supported_modes.standard.len() - 1;
                } else {
                    idx -= 1;
                }
            } else {
                idx += 1;
                if idx == self.supported_modes.standard.len() {
                    idx = 0;
                }
            }
            let next = self.supported_modes.standard[idx];

            self.config.read();
            if let Some(data) = self.config.builtins.get(&next) {
                self.write_mode(&data)?;
                self.config.current_mode = next;
            }
            self.config.write();
        }

        Ok(())
    }

    #[inline]
    fn write_mode(&self, mode: &AuraEffect) -> Result<(), RogError> {
        if !self.supported_modes.standard.contains(&mode.mode()) {
            return Err(RogError::NotSupported);
        }
        let bytes: [u8; LED_MSG_LEN] = mode.into();
        self.write_bytes(&bytes)?;
        self.write_bytes(&LED_SET)?;
        // Changes won't persist unless apply is set
        self.write_bytes(&LED_APPLY)?;
        Ok(())
    }
}
