// Only these two packets must be 17 bytes
static KBD_BRIGHT_PATH: &str = "/sys/class/leds/asus::kbd_backlight/brightness";

use crate::{
    error::RogError,
    laptops::{LaptopLedData, ASUS_KEYBOARD_DEVICES},
    CtrlTask,
};
use async_trait::async_trait;
use log::{error, info, warn};
use logind_zbus::manager::ManagerProxy;
use rog_aura::{usb::AuraControl, AuraZone, Direction, Speed, GRADIENT};
use rog_aura::{
    usb::{LED_APPLY, LED_SET},
    AuraEffect, LedBrightness, LED_MSG_LEN,
};
use rog_supported::LedSupportedFunctions;
use smol::{stream::StreamExt, Executor};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};
use std::{fs::OpenOptions, sync::MutexGuard};
use zbus::Connection;

use crate::GetSupported;

use super::config::AuraConfig;

impl GetSupported for CtrlKbdLed {
    type A = LedSupportedFunctions;

    fn get_supported() -> Self::A {
        // let mode = <&str>::from(&<AuraModes>::from(*mode));
        let laptop = LaptopLedData::get_data();
        let stock_led_modes = laptop.standard;
        let multizone_led_mode = laptop.multizone;
        let per_key_led_mode = laptop.per_key;

        LedSupportedFunctions {
            brightness_set: CtrlKbdLed::get_kbd_bright_path().is_some(),
            stock_led_modes,
            multizone_led_mode,
            per_key_led_mode,
        }
    }
}

pub struct CtrlKbdLed {
    pub led_node: Option<String>,
    pub bright_node: String,
    pub supported_modes: LaptopLedData,
    pub flip_effect_write: bool,
    pub config: AuraConfig,
}

pub struct CtrlKbdLedTask {
    inner: Arc<Mutex<CtrlKbdLed>>,
}

impl CtrlKbdLedTask {
    pub fn new(inner: Arc<Mutex<CtrlKbdLed>>) -> Self {
        Self { inner }
    }

    fn update_config(lock: &mut CtrlKbdLed) -> Result<(), RogError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&lock.bright_node)
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => {
                    RogError::MissingLedBrightNode((&lock.bright_node).into(), err)
                }
                _ => RogError::Path((&lock.bright_node).into(), err),
            })?;
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)
            .map_err(|err| RogError::Read("buffer".into(), err))?;
        if let Some(num) = char::from(buf[0]).to_digit(10) {
            if lock.config.brightness != num.into() {
                lock.config.read();
                lock.config.brightness = num.into();
                lock.config.write();
            }
            return Ok(());
        }
        Err(RogError::ParseLed)
    }
}

#[async_trait]
impl CtrlTask for CtrlKbdLedTask {
    async fn create_tasks(&self, executor: &mut Executor) -> Result<(), RogError> {
        let connection = Connection::system()
            .await
            .expect("CtrlKbdLedTask could not create dbus connection");

        let manager = ManagerProxy::new(&connection)
            .await
            .expect("CtrlKbdLedTask could not create ManagerProxy");

        let load_save = |start: bool, mut lock: MutexGuard<CtrlKbdLed>| {
            // If waking up
            if !start {
                info!("CtrlKbdLedTask reloading brightness and modes");
                lock.set_brightness(lock.config.brightness)
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
                lock.write_current_config_mode()
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
            } else if start {
                info!("CtrlKbdLedTask saving last brightness");
                Self::update_config(&mut lock)
                    .map_err(|e| error!("CtrlKbdLedTask: {e}"))
                    .ok();
            }
        };

        let inner = self.inner.clone();
        executor
            .spawn(async move {
                if let Ok(notif) = manager.receive_prepare_for_sleep().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                loop {
                                    // Loop so that we do aquire the lock but also don't block other
                                    // threads (prevents potential deadlocks)
                                    if let Ok(lock) = inner.clone().try_lock() {
                                        load_save(args.start, lock);
                                        break;
                                    }
                                }
                            }
                        })
                        .await;
                }
                if let Ok(notif) = manager.receive_prepare_for_shutdown().await {
                    notif
                        .for_each(|event| {
                            if let Ok(args) = event.args() {
                                loop {
                                    if let Ok(lock) = inner.clone().try_lock() {
                                        load_save(args.start, lock);
                                        break;
                                    }
                                }
                            }
                        })
                        .await;
                }
            })
            .detach();
        Ok(())
    }
}

pub struct CtrlKbdLedReloader(pub Arc<Mutex<CtrlKbdLed>>);

impl crate::Reloadable for CtrlKbdLedReloader {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut ctrl) = self.0.try_lock() {
            ctrl.write_current_config_mode()?;
            ctrl.set_power_states(&ctrl.config)
                .map_err(|err| warn!("{err}"))
                .ok();
        }
        Ok(())
    }
}

pub struct CtrlKbdLedZbus(pub Arc<Mutex<CtrlKbdLed>>);

impl CtrlKbdLedZbus {
    pub fn new(inner: Arc<Mutex<CtrlKbdLed>>) -> Self {
        Self(inner)
    }
}

impl CtrlKbdLed {
    pub fn new(supported_modes: LaptopLedData, config: AuraConfig) -> Result<Self, RogError> {
        // TODO: return error if *all* nodes are None
        let mut led_node = None;
        for prod in ASUS_KEYBOARD_DEVICES.iter() {
            match Self::find_led_node(prod) {
                Ok(node) => {
                    led_node = Some(node);
                    info!("Looked for keyboard controller 0x{prod}: Found");
                    break;
                }
                Err(err) => info!("Looked for keyboard controller 0x{prod}: {err}"),
            }
        }

        let bright_node = Self::get_kbd_bright_path();

        if led_node.is_none() {
            return Err(RogError::NoAuraKeyboard);
        }

        if bright_node.is_none() {
            return Err(RogError::MissingFunction(
                "No brightness control, you may require a v5.11 series kernel or newer".into(),
            ));
        }

        let ctrl = CtrlKbdLed {
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

    pub(super) fn get_brightness(&self) -> Result<u8, RogError> {
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

    pub(super) fn set_brightness(&self, brightness: LedBrightness) -> Result<(), RogError> {
        let path = Path::new(&self.bright_node);
        let mut file =
            OpenOptions::new()
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

    pub fn next_brightness(&mut self) -> Result<(), RogError> {
        let mut bright = (self.config.brightness as u32) + 1;
        if bright > 3 {
            bright = 0;
        }
        self.config.brightness = <LedBrightness>::from(bright);
        self.config.write();
        self.set_brightness(self.config.brightness)
    }

    pub fn prev_brightness(&mut self) -> Result<(), RogError> {
        let mut bright = self.config.brightness as u32;
        if bright == 0 {
            bright = 3;
        } else {
            bright -= 1;
        }
        self.config.brightness = <LedBrightness>::from(bright);
        self.config.write();
        self.set_brightness(self.config.brightness)
    }

    /// Set combination state for boot animation/sleep animation/all leds/keys leds/side leds LED active
    pub(super) fn set_power_states(&self, config: &AuraConfig) -> Result<(), RogError> {
        let set: Vec<AuraControl> = config.enabled.iter().map(|v| *v).collect();
        let bytes = AuraControl::to_bytes(&set);

        let message = [
            0x5d, 0xbd, 0x01, bytes[0], bytes[1], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        self.write_bytes(&message)?;
        self.write_bytes(&LED_SET)?;
        // Changes won't persist unless apply is set
        self.write_bytes(&LED_APPLY)?;
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

    /// Set an Aura effect if the effect mode or zone is supported.
    ///
    /// On success the aura config file is read to refresh cached values, then the effect is
    /// stored and config written to disk.
    pub(crate) fn set_effect(&mut self, effect: AuraEffect) -> Result<(), RogError> {
        if !self.supported_modes.standard.contains(&effect.mode) {
            return Err(RogError::AuraEffectNotSupported);
        } else if effect.zone != AuraZone::None
            && !self.supported_modes.multizone.contains(&effect.zone)
        {
            return Err(RogError::AuraEffectNotSupported);
        }

        self.write_mode(&effect)?;
        self.config.read(); // refresh config if successful
        self.config.set_builtin(effect);
        self.config.write();
        Ok(())
    }

    /// Should only be used if the bytes you are writing are verified correct
    fn write_bytes(&self, message: &[u8]) -> Result<(), RogError> {
        if let Some(led_node) = &self.led_node {
            if let Ok(mut file) = OpenOptions::new().write(true).open(led_node) {
                // println!("write: {:02x?}", &message);
                return file
                    .write_all(message)
                    .map_err(|err| RogError::Write("write_bytes".into(), err));
            }
        }
        Err(RogError::NoAuraNode)
    }

    /// Write an effect block. This is for per-key
    fn _write_effect(&mut self, effect: &[Vec<u8>]) -> Result<(), RogError> {
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

    pub(super) fn toggle_mode(&mut self, reverse: bool) -> Result<(), RogError> {
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
            if self.config.builtins.contains_key(&next) {
                self.config.current_mode = next;
                self.write_current_config_mode()?;
            }
            self.config.write();
        }

        Ok(())
    }

    fn write_mode(&self, mode: &AuraEffect) -> Result<(), RogError> {
        let bytes: [u8; LED_MSG_LEN] = mode.into();
        self.write_bytes(&bytes)?;
        self.write_bytes(&LED_SET)?;
        // Changes won't persist unless apply is set
        self.write_bytes(&LED_APPLY)?;
        Ok(())
    }

    fn write_current_config_mode(&mut self) -> Result<(), RogError> {
        if self.config.multizone_on {
            let mode = self.config.current_mode;
            let mut create = false;
            // There is no multizone config for this mode so create one here
            // using the colours of rainbow if it exists, or first available
            // mode, or random
            if self.config.multizone.is_none() {
                create = true;
            } else if let Some(multizones) = self.config.multizone.as_ref() {
                if !multizones.contains_key(&mode) {
                    create = true;
                }
            }
            if create {
                info!("No user-set config for zone founding, attempting a default");
                self.create_multizone_default()?;
            }

            if let Some(multizones) = self.config.multizone.as_ref() {
                if let Some(set) = multizones.get(&mode) {
                    for mode in set {
                        self.write_mode(mode)?;
                    }
                }
            }
        } else {
            let mode = self.config.current_mode;
            if let Some(effect) = self.config.builtins.get(&mode) {
                self.write_mode(effect)?;
            }
        }

        Ok(())
    }

    /// Create a default for the `current_mode` if multizone and no config exists.
    fn create_multizone_default(&mut self) -> Result<(), RogError> {
        let mut default = vec![];
        for (i, tmp) in self.supported_modes.multizone.iter().enumerate() {
            default.push(AuraEffect {
                mode: self.config.current_mode,
                zone: *tmp,
                colour1: *GRADIENT.get(i).unwrap_or(&GRADIENT[0]),
                colour2: *GRADIENT.get(GRADIENT.len() - i).unwrap_or(&GRADIENT[6]),
                speed: Speed::Med,
                direction: Direction::Left,
            })
        }
        if default.is_empty() {
            return Err(RogError::AuraEffectNotSupported);
        }

        if let Some(multizones) = self.config.multizone.as_mut() {
            multizones.insert(self.config.current_mode, default);
        } else {
            let mut tmp = BTreeMap::new();
            tmp.insert(self.config.current_mode, default);
            self.config.multizone = Some(tmp);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour};

    use crate::{ctrl_aura::config::AuraConfig, laptops::LaptopLedData};

    use super::CtrlKbdLed;

    #[test]
    // #[ignore = "Must be manually run due to detection stage"]
    fn check_set_mode_errors() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::default();
        let supported_modes = LaptopLedData {
            prod_family: "".into(),
            board_names: vec![],
            standard: vec![AuraModeNum::Static],
            multizone: vec![],
            per_key: false,
        };
        let mut controller = CtrlKbdLed {
            led_node: None,
            bright_node: String::new(),
            supported_modes,
            flip_effect_write: false,
            config,
        };

        let mut effect = AuraEffect::default();
        effect.colour1 = Colour(0xff, 0x00, 0xff);
        effect.zone = AuraZone::None;

        // This error comes from write_bytes because we don't have a keyboard node stored
        assert_eq!(
            controller
                .set_effect(effect.clone())
                .unwrap_err()
                .to_string(),
            "No Aura keyboard node found"
        );

        effect.mode = AuraModeNum::Laser;
        assert_eq!(
            controller
                .set_effect(effect.clone())
                .unwrap_err()
                .to_string(),
            "Aura effect not supported"
        );

        effect.mode = AuraModeNum::Static;
        effect.zone = AuraZone::Key2;
        assert_eq!(
            controller
                .set_effect(effect.clone())
                .unwrap_err()
                .to_string(),
            "Aura effect not supported"
        );

        controller.supported_modes.multizone.push(AuraZone::Key2);
        assert_eq!(
            controller
                .set_effect(effect.clone())
                .unwrap_err()
                .to_string(),
            "No Aura keyboard node found"
        );
    }

    #[test]
    fn create_multizone_if_no_config() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::default();
        let supported_modes = LaptopLedData {
            prod_family: "".into(),
            board_names: vec![],
            standard: vec![AuraModeNum::Static],
            multizone: vec![],
            per_key: false,
        };
        let mut controller = CtrlKbdLed {
            led_node: None,
            bright_node: String::new(),
            supported_modes,
            flip_effect_write: false,
            config,
        };

        assert!(controller.config.multizone.is_none());
        assert!(controller.create_multizone_default().is_err());
        assert!(controller.config.multizone.is_none());

        controller.supported_modes.multizone.push(AuraZone::Key1);
        controller.supported_modes.multizone.push(AuraZone::Key2);
        assert!(controller.create_multizone_default().is_ok());
        assert!(controller.config.multizone.is_some());

        let m = controller.config.multizone.unwrap();
        assert!(m.contains_key(&AuraModeNum::Static));
        let e = m.get(&AuraModeNum::Static).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].zone, AuraZone::Key1);
        assert_eq!(e[1].zone, AuraZone::Key2);
    }
}
