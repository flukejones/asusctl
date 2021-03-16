// Only these two packets must be 17 bytes
static LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

static KBD_BRIGHT_PATH: &str = "/sys/class/leds/asus::kbd_backlight/brightness";

use crate::{
    config::Config,
    error::RogError,
    laptops::{match_laptop, HELP_ADDRESS},
};
use log::{error, info, warn};
use rog_types::{
    aura_brightness_bytes,
    aura_modes::{AuraModes, PER_KEY},
    fancy::KeyColourArray,
    LED_MSG_LEN,
};
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
    pub stock_led_modes: Option<Vec<u8>>,
    pub per_key_led_mode: bool,
}

impl GetSupported for CtrlKbdBacklight {
    type A = LedSupportedFunctions;

    fn get_supported() -> Self::A {
        // let mode = <&str>::from(&<AuraModes>::from(*mode));
        let mut stock_led_modes = None;
        let mut per_key_led_mode = false;
        if let Some(laptop) = match_laptop() {
            let modes = laptop.supported_modes().to_vec();
            if modes.contains(&PER_KEY) {
                per_key_led_mode = true;
                let modes = modes.iter().filter(|x| **x != PER_KEY).copied().collect();
                stock_led_modes = Some(modes);
            } else {
                stock_led_modes = Some(modes);
            }
        }

        LedSupportedFunctions {
            brightness_set: CtrlKbdBacklight::get_kbd_bright_path().is_ok(),
            stock_led_modes,
            per_key_led_mode,
        }
    }
}

pub struct CtrlKbdBacklight {
    led_node: Option<String>,
    #[allow(dead_code)]
    kbd_node: Option<String>,
    pub bright_node: String,
    supported_modes: Vec<u8>,
    flip_effect_write: bool,
    config: Arc<Mutex<Config>>,
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

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl DbusKbdBacklight {
    fn set_led_mode(&mut self, data: String) {
        if let Ok(data) = serde_json::from_str(&data) {
            if let Ok(mut ctrl) = self.inner.try_lock() {
                if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                    match &data {
                        AuraModes::PerKey(_) => {
                            ctrl.do_command(data, &mut cfg)
                                .unwrap_or_else(|err| warn!("{}", err));
                        }
                        _ => {
                            if let Ok(json) = serde_json::to_string(&data) {
                                match ctrl.do_command(data, &mut cfg) {
                                    Ok(_) => {
                                        self.notify_led(&json).ok();
                                    }
                                    Err(err) => {
                                        warn!("{}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            warn!("SetKeyBacklight could not deserialise");
        }
    }

    fn next_led_mode(&self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                ctrl.toggle_mode(false, &mut cfg)
                    .unwrap_or_else(|err| warn!("{}", err));

                if let Some(mode) = cfg.get_led_mode_data(cfg.kbd_backlight_mode) {
                    if let Ok(json) = serde_json::to_string(&mode) {
                        self.notify_led(&json)
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
            }
        }
    }

    fn prev_led_mode(&self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                ctrl.toggle_mode(true, &mut cfg)
                    .unwrap_or_else(|err| warn!("{}", err));

                if let Some(mode) = cfg.get_led_mode_data(cfg.kbd_backlight_mode) {
                    if let Ok(json) = serde_json::to_string(&mode) {
                        self.notify_led(&json)
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
            }
        }
    }

    /// Return the current mode data
    fn led_mode(&self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(cfg) = ctrl.config.clone().try_lock() {
                if let Some(mode) = cfg.get_led_mode_data(cfg.kbd_backlight_mode) {
                    if let Ok(json) = serde_json::to_string(&mode) {
                        return json;
                    }
                }
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not deserialise".to_string()
    }

    /// Return a list of available modes
    fn led_modes(&self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(cfg) = ctrl.config.clone().try_lock() {
                if let Ok(json) = serde_json::to_string(&cfg.kbd_backlight_modes) {
                    return json;
                }
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        "SetKeyBacklight could not deserialise".to_string()
    }

    /// Return the current LED brightness
    fn led_brightness(&self) -> i8 {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(cfg) = ctrl.config.clone().try_lock() {
                return cfg.kbd_led_brightness as i8;
            }
        }
        warn!("SetKeyBacklight could not deserialise");
        -1
    }

    #[dbus_interface(signal)]
    fn notify_led(&self, data: &str) -> zbus::Result<()>;
}

impl crate::Reloadable for CtrlKbdBacklight {
    fn reload(&mut self) -> Result<(), RogError> {
        // set current mode (if any)
        if let Ok(mut config) = self.config.clone().try_lock() {
            if self.supported_modes.len() > 1 {
                if self.supported_modes.contains(&config.kbd_backlight_mode) {
                    let mode = config
                        .get_led_mode_data(config.kbd_backlight_mode)
                        .ok_or(RogError::NotSupported)?
                        .to_owned();
                    self.write_mode(&mode)?;
                    info!("Reloaded last used mode");
                } else {
                    warn!(
                        "An unsupported mode was set: {}, reset to first mode available",
                        <&str>::from(&<AuraModes>::from(config.kbd_backlight_mode))
                    );
                    for (idx, mode) in config.kbd_backlight_modes.iter_mut().enumerate() {
                        if !self.supported_modes.contains(&mode.into()) {
                            config.kbd_backlight_modes.remove(idx);
                            config.write();
                            break;
                        }
                    }
                    config.kbd_backlight_mode = self.supported_modes[0];
                    // TODO: do a recursive call with a boxed dyn future later
                    let mode = config
                        .get_led_mode_data(config.kbd_backlight_mode)
                        .ok_or(RogError::NotSupported)?
                        .to_owned();
                    self.write_mode(&mode)?;
                    info!("Reloaded last used mode");
                }
            }

            // Reload brightness
            let bright = config.kbd_led_brightness;
            let bytes = aura_brightness_bytes(bright);
            self.write_bytes(&bytes)?;
            info!("Reloaded last used brightness");
        }
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
            if let Ok(mut config) = self.config.clone().try_lock() {
                if config.kbd_led_brightness != num as u8 {
                    config.read();
                    config.kbd_led_brightness = num as u8;
                    config.write();
                }
            }
            return Ok(());
        }
        Err(RogError::ParseLED)
    }
}

impl CtrlKbdBacklight {
    #[inline]
    pub fn new(
        id_product: &str,
        condev_iface: Option<&String>,
        supported_modes: Vec<u8>,
        config: Arc<Mutex<Config>>,
    ) -> Result<Self, RogError> {
        // TODO: return error if *all* nodes are None
        let led_node = Self::get_node_failover(id_product, None, Self::scan_led_node).map_or_else(
            |err| {
                warn!("led_node: {}", err);
                None
            },
            Some,
        );

        let kbd_node = Self::get_node_failover(id_product, condev_iface, Self::scan_kbd_node)
            .map_or_else(
                |err| {
                    warn!("kbd_node: {}", err);
                    None
                },
                Some,
            );

        let bright_node = Self::get_kbd_bright_path();

        if led_node.is_none() && kbd_node.is_none() && Self::get_kbd_bright_path().is_err() {
            return Err(RogError::MissingFunction(
                "All keyboard features missing, you may require a v5.11 series kernel or newer"
                    .into(),
            ));
        }

        let ctrl = CtrlKbdBacklight {
            // Using `ok` here so we can continue without keyboard features but
            // still get brightness control at least... maybe...
            led_node,
            kbd_node,
            // TODO: Check for existance
            bright_node: bright_node?.to_owned(),
            supported_modes,
            flip_effect_write: false,
            config,
        };
        Ok(ctrl)
    }

    fn get_kbd_bright_path() -> Result<&'static str, RogError> {
        if Path::new(KBD_BRIGHT_PATH).exists() {
            Ok(KBD_BRIGHT_PATH)
        } else {
            Err(RogError::MissingFunction(
                "Keyboard features missing, you may require a v5.11 series kernel or newer".into(),
            ))
        }
    }

    fn get_node_failover(
        id_product: &str,
        iface: Option<&String>,
        fun: fn(&str, Option<&String>) -> Result<String, RogError>,
    ) -> Result<String, RogError> {
        match fun(id_product, iface) {
            Ok(o) => return Ok(o),
            Err(e) => {
                warn!("Looking for node: {}", e.to_string());
            }
        }
        Err(RogError::NotFound(format!("{}, {:?}", id_product, iface)))
    }

    fn scan_led_node(id_product: &str, _: Option<&String>) -> Result<String, RogError> {
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
        warn!("Did not find a hidraw node for LED control, your device may be unsupported or require a kernel patch, see: {}", HELP_ADDRESS);
        Err(RogError::MissingFunction(
            "ASUS LED device node not found".into(),
        ))
    }

    fn scan_kbd_node(id_product: &str, iface: Option<&String>) -> Result<String, RogError> {
        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            RogError::Udev("enumerator failed".into(), err)
        })?;
        enumerator.match_subsystem("input").map_err(|err| {
            warn!("{}", err);
            RogError::Udev("match_subsystem failed".into(), err)
        })?;
        enumerator
            .match_property("ID_MODEL_ID", id_product)
            .map_err(|err| {
                warn!("{}", err);
                RogError::Udev("match_property failed".into(), err)
            })?;

        for device in enumerator
            .scan_devices()
            .map_err(|err| {
                warn!("{}", err);
                err
            })
            .map_err(|err| {
                warn!("{}", err);
                RogError::Udev("scan_devices failed".into(), err)
            })?
        {
            if let Some(dev_node) = device.devnode() {
                if let Some(inum) = device.property_value("ID_USB_INTERFACE_NUM") {
                    if let Some(iface) = iface {
                        if inum == iface.as_str() {
                            info!("Using device at: {:?} for keyboard polling", dev_node);
                            return Ok(dev_node.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        warn!("Did not find keyboard consumer device node, if expected functions are missing please file an issue at {}", HELP_ADDRESS);
        Err(RogError::MissingFunction(
            "ASUS keyboard 'Consumer Device' node not found".into(),
        ))
    }

    pub fn do_command(&mut self, mode: AuraModes, config: &mut Config) -> Result<(), RogError> {
        self.set_and_save(mode, config)
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
    fn set_and_save(&mut self, mode: AuraModes, config: &mut Config) -> Result<(), RogError> {
        match mode {
            AuraModes::LedBrightness(n) => {
                let bytes: [u8; LED_MSG_LEN] = (&mode).into();
                self.write_bytes(&bytes)?;
                config.read();
                config.kbd_led_brightness = n;
                config.write();
                info!("LED brightness set to {:#?}", n);
            }
            AuraModes::PerKey(v) => {
                if v.is_empty() || v[0].is_empty() {
                    let bytes = KeyColourArray::get_init_msg();
                    self.write_bytes(&bytes)?;
                } else {
                    self.write_effect(&v)?;
                }
            }
            _ => {
                config.read();
                let mode_num: u8 = u8::from(&mode);
                self.write_mode(&mode)?;
                config.kbd_backlight_mode = mode_num;
                config.set_mode_data(mode);
                config.write();
            }
        }
        Ok(())
    }

    #[inline]
    fn toggle_mode(&mut self, reverse: bool, config: &mut Config) -> Result<(), RogError> {
        let current = config.kbd_backlight_mode;
        if let Some(idx) = self.supported_modes.iter().position(|v| *v == current) {
            let mut idx = idx;
            // goes past end of array
            if reverse {
                if idx == 0 {
                    idx = self.supported_modes.len() - 1;
                } else {
                    idx -= 1;
                }
            } else {
                idx += 1;
                if idx == self.supported_modes.len() {
                    idx = 0;
                }
            }
            let next = self.supported_modes[idx];

            config.read();
            if let Some(data) = config.get_led_mode_data(next) {
                self.write_mode(&data)?;
                config.kbd_backlight_mode = next;
            }
            config.write();
        }

        Ok(())
    }

    #[inline]
    fn write_mode(&mut self, mode: &AuraModes) -> Result<(), RogError> {
        let mode_num: u8 = u8::from(mode);
        if !self.supported_modes.contains(&mode_num) {
            return Err(RogError::NotSupported);
        }
        match mode {
            AuraModes::PerKey(v) => {
                if v.is_empty() || v[0].is_empty() {
                    let bytes = KeyColourArray::get_init_msg();
                    self.write_bytes(&bytes)?;
                } else {
                    self.write_effect(v)?;
                }
            }
            AuraModes::MultiStatic(_) | AuraModes::MultiBreathe(_) => {
                let bytes: [[u8; LED_MSG_LEN]; 4] = mode.into();
                for array in bytes.iter() {
                    self.write_bytes(array)?;
                }
                self.write_bytes(&LED_SET)?;
                // Changes won't persist unless apply is set
                self.write_bytes(&LED_APPLY)?;
                return Ok(());
            }
            _ => {
                let bytes: [u8; LED_MSG_LEN] = mode.into();
                self.write_bytes(&bytes)?;
                self.write_bytes(&LED_SET)?;
                // Changes won't persist unless apply is set
                self.write_bytes(&LED_APPLY)?;
            }
        }
        Ok(())
    }
}
