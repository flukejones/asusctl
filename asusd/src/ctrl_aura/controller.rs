use std::collections::HashSet;

use config_traits::StdConfig;
use dmi_id::DMIID;
use inotify::Inotify;
use log::{debug, info, warn};
use rog_aura::aura_detection::LedSupportData;
use rog_aura::keyboard::{LedUsbPackets, UsbPackets};
use rog_aura::usb::{LED_APPLY, LED_SET};
use rog_aura::{AuraDeviceType, AuraEffect, LedBrightness, PowerZones, LED_MSG_LEN};
use rog_platform::hid_raw::HidRaw;
use rog_platform::keyboard_led::KeyboardBacklight;
use udev::Device;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

use super::config::AuraConfig;
use crate::ctrl_aura::manager::{dbus_path_for_dev, dbus_path_for_tuf, start_tasks};
use crate::ctrl_aura::trait_impls::CtrlAuraZbus;
use crate::error::RogError;
use crate::CtrlTask;

#[derive(Debug)]
pub enum LEDNode {
    /// Brightness and/or TUF RGB controls
    KbdLed(KeyboardBacklight),
    /// Raw HID handle
    Rog(Option<KeyboardBacklight>, HidRaw),
}

impl LEDNode {
    // TODO: move various methods upwards to this
    pub fn set_brightness(&self, value: u8) -> Result<(), RogError> {
        match self {
            LEDNode::KbdLed(k) => k.set_brightness(value)?,
            LEDNode::Rog(k, r) => {
                if let Some(k) = k {
                    k.set_brightness(value)?;
                    let x = k.get_brightness()?;
                    if x != value {
                        debug!(
                            "Kernel brightness control didn't read back correct value, setting \
                             with raw hid"
                        );
                        r.write_bytes(&[0x5a, 0xba, 0xc5, 0xc4, value])?;
                    }
                } else {
                    debug!("No brightness control found, trying raw write");
                    r.write_bytes(&[0x5a, 0xba, 0xc5, 0xc4, value])?;
                }
            }
        }
        Ok(())
    }

    pub fn get_brightness(&self) -> Result<u8, RogError> {
        Ok(match self {
            LEDNode::KbdLed(k) => k.get_brightness()?,
            LEDNode::Rog(k, _) => {
                if let Some(k) = k {
                    k.get_brightness()?
                } else {
                    debug!("No brightness control found");
                    return Err(RogError::MissingFunction(
                        "No keyboard brightness control found".to_string(),
                    ));
                }
            }
        })
    }

    pub fn monitor_brightness(&self) -> Result<Inotify, RogError> {
        Ok(match self {
            LEDNode::KbdLed(k) => k.monitor_brightness()?,
            LEDNode::Rog(k, _) => {
                if let Some(k) = k {
                    k.monitor_brightness()?
                } else {
                    debug!("No brightness control found");
                    return Err(RogError::MissingFunction(
                        "No keyboard brightness control found".to_string(),
                    ));
                }
            }
        })
    }

    pub fn has_brightness_control(&self) -> bool {
        match self {
            LEDNode::KbdLed(k) => k.has_brightness(),
            LEDNode::Rog(k, _) => {
                if let Some(k) = k {
                    k.has_brightness()
                } else {
                    false
                }
            }
        }
    }
}

/// Individual controller for one Aura device
pub struct CtrlKbdLed {
    pub led_type: AuraDeviceType,
    pub led_node: LEDNode,
    pub supported_data: LedSupportData, // TODO: is storing this really required?
    pub per_key_mode_active: bool,
    pub config: AuraConfig,
    pub dbus_path: OwnedObjectPath,
}

impl CtrlKbdLed {
    pub fn add_to_dbus_and_start(
        self,
        interfaces: &mut HashSet<OwnedObjectPath>,
        conn: Connection,
    ) -> Result<(), RogError> {
        let dbus_path = self.dbus_path.clone();
        let dbus_path_cpy = self.dbus_path.clone();
        info!(
            "AuraManager starting device at: {:?}, {:?}",
            dbus_path, self.led_type
        );
        let conn_copy = conn.clone();
        let sig_ctx1 = CtrlAuraZbus::signal_context(&conn_copy)?;
        let sig_ctx2 = CtrlAuraZbus::signal_context(&conn_copy)?;
        let zbus = CtrlAuraZbus::new(self, sig_ctx1);
        tokio::spawn(
            async move { start_tasks(zbus, conn_copy.clone(), sig_ctx2, dbus_path).await },
        );
        interfaces.insert(dbus_path_cpy);
        Ok(())
    }

    /// Build and init a `CtrlKbdLed` from a udev device. Maybe.
    /// This will initialise the config also.
    pub fn maybe_device(
        device: Device,
        interfaces: &mut HashSet<OwnedObjectPath>,
    ) -> Result<Option<Self>, RogError> {
        // usb_device gives us a product and vendor ID
        if let Some(usb_device) = device.parent_with_subsystem_devtype("usb", "usb_device")? {
            let dbus_path = dbus_path_for_dev(&usb_device).unwrap_or_default();
            if interfaces.contains(&dbus_path) {
                debug!("Already a ctrl at {dbus_path:?}, ignoring this end-point");
                return Ok(None);
            }

            // The asus_wmi driver latches MCU that controls the USB endpoints
            if let Some(parent) = device.parent() {
                if let Some(driver) = parent.driver() {
                    // There is a tree of devices added so filter by driver
                    if driver != "asus" {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
            // Device is something like 002, while its parent is the MCU
            // Think of it like the device is an endpoint of the USB device attached
            let mut prod_id = String::new();
            if let Some(usb_id) = usb_device.attribute_value("idProduct") {
                prod_id = usb_id.to_string_lossy().to_string();
                let aura_device = AuraDeviceType::from(prod_id.as_str());
                if aura_device == AuraDeviceType::Unknown {
                    log::debug!("Unknown or invalid device: {usb_id:?}, skipping");
                    return Ok(None);
                }
            }

            let dev_node = if let Some(dev_node) = usb_device.devnode() {
                dev_node
            } else {
                debug!("Device has no devnode, skipping");
                return Ok(None);
            };
            info!("AuraControl found device at: {:?}", dev_node);
            let dev = HidRaw::from_device(device)?;
            let mut controller = Self::from_hidraw(dev, dbus_path.clone())?;
            controller.config = AuraConfig::load_and_update_config(&prod_id);
            interfaces.insert(dbus_path);
            return Ok(Some(controller));
        }
        Ok(None)
    }

    pub fn find_all() -> Result<Vec<Self>, RogError> {
        info!("Searching for all Aura devices");
        let mut devices = Vec::new();
        let mut interfaces = HashSet::new(); // track and ensure we use only one hidraw per prod_id

        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            err
        })?;

        enumerator.match_subsystem("hidraw").map_err(|err| {
            warn!("{}", err);
            err
        })?;

        for end_point in enumerator.scan_devices()? {
            // maybe?
            if let Some(device) = Self::maybe_device(end_point, &mut interfaces)? {
                devices.push(device);
            }
        }

        // Check for a TUF laptop LED. Assume there is only ever one.
        if let Ok(kbd_backlight) = KeyboardBacklight::new() {
            if kbd_backlight.has_kbd_rgb_mode() {
                // Extra sure double-check that this isn't a laptop with crap
                // ACPI with borked return on the TUF rgb methods
                let dmi = DMIID::new().unwrap_or_default();
                info!("Found a TUF with product family: {}", dmi.product_family);
                info!("and board name: {}", dmi.board_name);

                if dmi.product_family.contains("TUF") {
                    info!("AuraControl found a TUF laptop keyboard");
                    let ctrl = CtrlKbdLed {
                        led_type: AuraDeviceType::LaptopKeyboardTuf,
                        led_node: LEDNode::KbdLed(kbd_backlight),
                        supported_data: LedSupportData::get_data("tuf"),
                        per_key_mode_active: false,
                        config: AuraConfig::load_and_update_config("tuf"),
                        dbus_path: dbus_path_for_tuf(),
                    };
                    devices.push(ctrl);
                }
            }
        } else {
            let dmi = DMIID::new().unwrap_or_default();
            warn!("No asus::kbd_backlight found for {} ??", dmi.product_family);
        }

        info!("Found {} Aura devices", devices.len());

        Ok(devices)
    }

    /// The generated data from this function has a default config. This config
    /// should be overwritten. The reason for the default config is because
    /// of async issues between this and udev/hidraw
    fn from_hidraw(device: HidRaw, dbus_path: OwnedObjectPath) -> Result<Self, RogError> {
        let rgb_led = KeyboardBacklight::new()
            .map_err(|e| {
                log::error!(
                    "{} is missing a keyboard backlight brightness control: {e:?}",
                    device.prod_id()
                );
            })
            .ok();
        let prod_id = AuraDeviceType::from(device.prod_id());
        if prod_id == AuraDeviceType::Unknown {
            log::error!("{} is AuraDevice::Unknown", device.prod_id());
            return Err(RogError::NoAuraNode);
        }

        // New loads data from the DB also
        // let config = Self::init_config(prod_id, data);

        let data = LedSupportData::get_data(device.prod_id());
        let ctrl = CtrlKbdLed {
            led_type: prod_id,
            led_node: LEDNode::Rog(rgb_led, device),
            supported_data: data.clone(),
            per_key_mode_active: false,
            config: AuraConfig::default(),
            dbus_path,
        };
        Ok(ctrl)
    }

    pub(super) fn fix_ally_power(&mut self) -> Result<(), RogError> {
        if self.led_type == AuraDeviceType::Ally {
            if let LEDNode::Rog(_, hid_raw) = &self.led_node {
                if self.config.ally_fix.is_none() {
                    let msg = [0x5d, 0xbd, 0x01, 0xff, 0xff, 0xff, 0xff];
                    hid_raw.write_bytes(&msg)?;
                    info!("Reset Ally power settings to base");
                    self.config.ally_fix = Some(true);
                }
                self.config.write();
            }
        }
        Ok(())
    }

    /// Set combination state for boot animation/sleep animation/all leds/keys
    /// leds/side leds LED active
    pub(super) fn set_power_states(&mut self) -> Result<(), RogError> {
        if let LEDNode::KbdLed(platform) = &mut self.led_node {
            // TODO: tuf bool array
            let buf = self.config.enabled.to_bytes(self.led_type);
            platform.set_kbd_rgb_state(&buf)?;
        } else if let LEDNode::Rog(_, hid_raw) = &self.led_node {
            if let Some(p) = self.config.enabled.states.first() {
                if p.zone == PowerZones::Ally {
                    let msg = [0x5d, 0xd1, 0x09, 0x01, p.new_to_byte() as u8, 0x0, 0x0];
                    hid_raw.write_bytes(&msg)?;
                    return Ok(());
                }
            }

            let bytes = self.config.enabled.to_bytes(self.led_type);
            let msg = [0x5d, 0xbd, 0x01, bytes[0], bytes[1], bytes[2], bytes[3]];
            hid_raw.write_bytes(&msg)?;
        }
        Ok(())
    }

    /// Write an effect block. This is for per-key, but can be repurposed to
    /// write the raw factory mode packets - when doing this it is expected that
    /// only the first `Vec` (`effect[0]`) is valid.
    pub fn write_effect_block(&mut self, effect: &UsbPackets) -> Result<(), RogError> {
        if self.config.brightness == LedBrightness::Off {
            self.config.brightness = LedBrightness::Med;
            self.config.write();
        }

        let pkt_type = effect[0][1];
        const PER_KEY_TYPE: u8 = 0xbc;

        if pkt_type != PER_KEY_TYPE {
            self.per_key_mode_active = false;
            if let LEDNode::Rog(_, hid_raw) = &self.led_node {
                hid_raw.write_bytes(&effect[0])?;
                hid_raw.write_bytes(&LED_SET)?;
                // hid_raw.write_bytes(&LED_APPLY)?;
            }
        } else {
            if !self.per_key_mode_active {
                if let LEDNode::Rog(_, hid_raw) = &self.led_node {
                    let init = LedUsbPackets::get_init_msg();
                    hid_raw.write_bytes(&init)?;
                }
                self.per_key_mode_active = true;
            }
            if let LEDNode::Rog(_, hid_raw) = &self.led_node {
                for row in effect.iter() {
                    hid_raw.write_bytes(row)?;
                }
            } else if let LEDNode::KbdLed(tuf) = &self.led_node {
                for row in effect.iter() {
                    let r = row[9];
                    let g = row[10];
                    let b = row[11];
                    tuf.set_kbd_rgb_mode(&[0, 0, r, g, b, 0])?;
                }
            }
        }
        Ok(())
    }

    /// Write the AuraEffect to the device
    pub fn write_effect_and_apply(&mut self, mode: &AuraEffect) -> Result<(), RogError> {
        if let LEDNode::KbdLed(platform) = &self.led_node {
            let buf = [
                1,
                mode.mode as u8,
                mode.colour1.r,
                mode.colour1.g,
                mode.colour1.b,
                mode.speed as u8,
            ];
            platform.set_kbd_rgb_mode(&buf)?;
        } else if let LEDNode::Rog(_, hid_raw) = &self.led_node {
            let bytes: [u8; LED_MSG_LEN] = mode.into();
            hid_raw.write_bytes(&bytes)?;
            hid_raw.write_bytes(&LED_SET)?;
            // Changes won't persist unless apply is set
            hid_raw.write_bytes(&LED_APPLY)?;
        } else {
            return Err(RogError::NoAuraKeyboard);
        }
        self.per_key_mode_active = false;
        Ok(())
    }

    pub(super) fn write_current_config_mode(&mut self) -> Result<(), RogError> {
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
                self.config.create_multizone_default(&self.supported_data)?;
            }

            if let Some(multizones) = self.config.multizone.as_mut() {
                if let Some(set) = multizones.get(&mode) {
                    for mode in set.clone() {
                        self.write_effect_and_apply(&mode)?;
                    }
                }
            }
        } else {
            let mode = self.config.current_mode;
            if let Some(effect) = self.config.builtins.get(&mode).cloned() {
                self.write_effect_and_apply(&effect)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rog_aura::aura_detection::LedSupportData;
    use rog_aura::{AuraDeviceType, AuraModeNum, AuraZone, PowerZones};
    use rog_platform::hid_raw::HidRaw;
    use rog_platform::keyboard_led::KeyboardBacklight;
    use zbus::zvariant::OwnedObjectPath;

    use super::CtrlKbdLed;
    use crate::ctrl_aura::config::AuraConfig;
    use crate::ctrl_aura::controller::LEDNode;

    #[test]
    #[ignore = "Unable to run in CI as the HIDRAW device is required"]
    fn create_multizone_if_no_config() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::new("19b6");
        let supported_basic_modes = LedSupportData {
            device_name: String::new(),
            product_id: String::new(),
            layout_name: "ga401".to_owned(),
            basic_modes: vec![AuraModeNum::Static],
            basic_zones: vec![],
            advanced_type: rog_aura::keyboard::AdvancedAuraType::None,
            power_zones: vec![PowerZones::Keyboard, PowerZones::RearGlow],
        };
        let mut controller = CtrlKbdLed {
            led_type: AuraDeviceType::LaptopKeyboard2021,
            led_node: LEDNode::Rog(
                Some(KeyboardBacklight::default()),
                HidRaw::new("19b6").unwrap(),
            ),
            supported_data: supported_basic_modes,
            per_key_mode_active: false,
            config,
            dbus_path: OwnedObjectPath::default(),
        };

        assert!(controller.config.multizone.is_none());
        assert!(controller
            .config
            .create_multizone_default(&controller.supported_data)
            .is_err());
        assert!(controller.config.multizone.is_none());

        controller.supported_data.basic_zones.push(AuraZone::Key1);
        controller.supported_data.basic_zones.push(AuraZone::Key2);
        assert!(controller
            .config
            .create_multizone_default(&controller.supported_data)
            .is_ok());
        assert!(controller.config.multizone.is_some());

        let m = controller.config.multizone.unwrap();
        assert!(m.contains_key(&AuraModeNum::Static));
        let e = m.get(&AuraModeNum::Static).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].zone, AuraZone::Key1);
        assert_eq!(e[1].zone, AuraZone::Key2);
    }

    #[test]
    #[ignore = "Unable to run in CI as the HIDRAW device is required"]
    // TODO: use sim device
    fn next_mode_create_multizone_if_no_config() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::new("19b6");
        let supported_basic_modes = LedSupportData {
            device_name: String::new(),
            product_id: String::new(),
            layout_name: "ga401".to_owned(),
            basic_modes: vec![AuraModeNum::Static],
            basic_zones: vec![AuraZone::Key1, AuraZone::Key2],
            advanced_type: rog_aura::keyboard::AdvancedAuraType::None,
            power_zones: vec![PowerZones::Keyboard, PowerZones::RearGlow],
        };
        let mut controller = CtrlKbdLed {
            led_type: AuraDeviceType::LaptopKeyboard2021,
            led_node: LEDNode::Rog(
                Some(KeyboardBacklight::default()),
                HidRaw::new("19b6").unwrap(),
            ),
            supported_data: supported_basic_modes,
            per_key_mode_active: false,
            config,
            dbus_path: OwnedObjectPath::default(),
        };

        assert!(controller.config.multizone.is_none());
        controller.config.multizone_on = true;
        // This is called in toggle_mode. It will error here because we have no
        // keyboard node in tests.
        assert_eq!(
            controller
                .write_current_config_mode()
                .unwrap_err()
                .to_string(),
            "No supported Aura keyboard"
        );
        assert!(controller.config.multizone.is_some());

        let m = controller.config.multizone.unwrap();
        assert!(m.contains_key(&AuraModeNum::Static));
        let e = m.get(&AuraModeNum::Static).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].zone, AuraZone::Key1);
        assert_eq!(e[1].zone, AuraZone::Key2);
    }
}
