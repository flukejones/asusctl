use std::collections::BTreeMap;

use config_traits::{StdConfig, StdConfigLoad};
use dmi_id::DMIID;
use log::{info, warn};
use rog_aura::advanced::{LedUsbPackets, UsbPackets};
use rog_aura::aura_detection::{LaptopLedData, ASUS_KEYBOARD_DEVICES};
use rog_aura::usb::{AuraDevice, LED_APPLY, LED_SET};
use rog_aura::{AuraEffect, Direction, LedBrightness, Speed, GRADIENT, LED_MSG_LEN};
use rog_platform::hid_raw::HidRaw;
use rog_platform::keyboard_led::KeyboardLed;

use super::config::{AuraConfig, AuraPowerConfig};
use crate::error::RogError;

#[derive(Debug)]
pub enum LEDNode {
    KbdLed(KeyboardLed),
    Rog(HidRaw),
    None,
}

pub struct CtrlKbdLed {
    // TODO: config stores the keyboard type as an AuraPower, use or update this
    pub led_prod: AuraDevice,
    pub led_node: LEDNode,
    pub sysfs_node: KeyboardLed,
    pub supported_modes: LaptopLedData,
    pub per_key_mode_active: bool,
    pub config: AuraConfig,
}

impl CtrlKbdLed {
    pub fn new(supported_modes: LaptopLedData) -> Result<Self, RogError> {
        let mut led_prod = AuraDevice::Unknown;
        let mut usb_node = None;
        for prod in ASUS_KEYBOARD_DEVICES {
            match HidRaw::new(prod.into()) {
                Ok(node) => {
                    led_prod = prod;
                    usb_node = Some(node);
                    info!(
                        "Looked for keyboard controller 0x{}: Found",
                        <&str>::from(prod)
                    );
                    break;
                }
                Err(err) => info!(
                    "Looked for keyboard controller 0x{}: {err}",
                    <&str>::from(prod)
                ),
            }
        }

        let rgb_led = KeyboardLed::new()?;

        if usb_node.is_none() && !rgb_led.has_kbd_rgb_mode() {
            let dmi = DMIID::new().unwrap_or_default();
            if dmi.dmi_family.contains("TUF") {
                warn!(
                    "kbd_rgb_mode was not found in the /sys/. You require a minimum 6.1 kernel \
                     and a supported TUF laptop"
                );
            }
            return Err(RogError::NoAuraKeyboard);
        }

        let led_node = if let Some(rog) = usb_node {
            info!("Found ROG USB keyboard");
            LEDNode::Rog(rog)
        } else if rgb_led.has_kbd_rgb_mode() {
            info!("Found TUF keyboard");
            LEDNode::KbdLed(rgb_led.clone())
        } else {
            LEDNode::None
        };

        // New loads data fromt he DB also
        let mut config_init = AuraConfig::new();
        let mut config_loaded = config_init.clone().load();
        // update the initialised data with what we loaded from disk
        for mode in &mut config_init.builtins {
            // update init values from loaded values if they exist
            if let Some(loaded) = config_loaded.builtins.get(mode.0) {
                *mode.1 = loaded.clone();
            }
        }
        // Then replace just incase the initialised data contains new modes added
        config_loaded.builtins = config_init.builtins;

        if let (Some(mut multizone_init), Some(multizone_loaded)) =
            (config_init.multizone, config_loaded.multizone.as_mut())
        {
            for mode in multizone_init.iter_mut() {
                // update init values from loaded values if they exist
                if let Some(loaded) = multizone_loaded.get(mode.0) {
                    let mut new_set = Vec::new();
                    // only reuse a zone mode if the mode is supported
                    for mode in loaded {
                        if supported_modes.basic_modes.contains(&mode.mode) {
                            new_set.push(mode.clone());
                        }
                    }
                    *mode.1 = new_set;
                }
            }
            *multizone_loaded = multizone_init;
        }

        let ctrl = CtrlKbdLed {
            led_prod,
            led_node,            // on TUF this is the same as rgb_led / kd_brightness
            sysfs_node: rgb_led, // If was none then we already returned above
            supported_modes,
            per_key_mode_active: false,
            config: config_loaded,
        };
        Ok(ctrl)
    }

    /// Set combination state for boot animation/sleep animation/all leds/keys
    /// leds/side leds LED active
    pub(super) fn set_power_states(&mut self) -> Result<(), RogError> {
        if let LEDNode::KbdLed(platform) = &mut self.led_node {
            if let Some(pwr) = AuraPowerConfig::to_tuf_bool_array(&self.config.enabled) {
                let buf = [1, pwr[1] as u8, pwr[2] as u8, pwr[3] as u8, pwr[4] as u8];
                platform.set_kbd_rgb_state(&buf)?;
            }
        } else if let LEDNode::Rog(hid_raw) = &self.led_node {
            let bytes = AuraPowerConfig::to_bytes(&self.config.enabled);
            let message = [0x5d, 0xbd, 0x01, bytes[0], bytes[1], bytes[2], bytes[3]];

            hid_raw.write_bytes(&message)?;
            hid_raw.write_bytes(&LED_SET)?;
            // Changes won't persist unless apply is set
            hid_raw.write_bytes(&LED_APPLY)?;
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
            if let LEDNode::Rog(hid_raw) = &self.led_node {
                hid_raw.write_bytes(&effect[0])?;
                hid_raw.write_bytes(&LED_SET)?;
                // hid_raw.write_bytes(&LED_APPLY)?;
            }
        } else {
            if !self.per_key_mode_active {
                if let LEDNode::Rog(hid_raw) = &self.led_node {
                    let init = LedUsbPackets::get_init_msg();
                    hid_raw.write_bytes(&init)?;
                }
                self.per_key_mode_active = true;
            }
            if let LEDNode::Rog(hid_raw) = &self.led_node {
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

    pub fn write_mode(&mut self, mode: &AuraEffect) -> Result<(), RogError> {
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
        } else if let LEDNode::Rog(hid_raw) = &self.led_node {
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
                self.create_multizone_default()?;
            }

            if let Some(multizones) = self.config.multizone.as_mut() {
                if let Some(set) = multizones.get(&mode) {
                    for mode in set.clone() {
                        self.write_mode(&mode)?;
                    }
                }
            }
        } else {
            let mode = self.config.current_mode;
            if let Some(effect) = self.config.builtins.get(&mode).cloned() {
                self.write_mode(&effect)?;
            }
        }

        Ok(())
    }

    /// Create a default for the `current_mode` if multizone and no config
    /// exists.
    fn create_multizone_default(&mut self) -> Result<(), RogError> {
        let mut default = vec![];
        for (i, tmp) in self.supported_modes.basic_zones.iter().enumerate() {
            default.push(AuraEffect {
                mode: self.config.current_mode,
                zone: *tmp,
                colour1: *GRADIENT.get(i).unwrap_or(&GRADIENT[0]),
                colour2: *GRADIENT.get(GRADIENT.len() - i).unwrap_or(&GRADIENT[6]),
                speed: Speed::Med,
                direction: Direction::Left,
            });
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
    use rog_aura::aura_detection::{LaptopLedData, PowerZones};
    use rog_aura::usb::AuraDevice;
    use rog_aura::{AuraModeNum, AuraZone};
    use rog_platform::keyboard_led::KeyboardLed;

    use super::CtrlKbdLed;
    use crate::ctrl_aura::config::AuraConfig;
    use crate::ctrl_aura::controller::LEDNode;

    #[test]
    fn create_multizone_if_no_config() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::from_default_support(AuraDevice::X19b6, &LaptopLedData::default());
        let supported_modes = LaptopLedData {
            board_name: String::new(),
            layout_name: "ga401".to_owned(),
            basic_modes: vec![AuraModeNum::Static],
            basic_zones: vec![],
            advanced_type: rog_aura::AdvancedAuraType::None,
            power_zones: vec![PowerZones::Keyboard, PowerZones::RearGlow],
        };
        let mut controller = CtrlKbdLed {
            led_prod: AuraDevice::X19b6,
            led_node: LEDNode::None,
            sysfs_node: KeyboardLed::default(),
            supported_modes,
            per_key_mode_active: false,
            config,
        };

        assert!(controller.config.multizone.is_none());
        assert!(controller.create_multizone_default().is_err());
        assert!(controller.config.multizone.is_none());

        controller.supported_modes.basic_zones.push(AuraZone::Key1);
        controller.supported_modes.basic_zones.push(AuraZone::Key2);
        assert!(controller.create_multizone_default().is_ok());
        assert!(controller.config.multizone.is_some());

        let m = controller.config.multizone.unwrap();
        assert!(m.contains_key(&AuraModeNum::Static));
        let e = m.get(&AuraModeNum::Static).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].zone, AuraZone::Key1);
        assert_eq!(e[1].zone, AuraZone::Key2);
    }

    #[test]
    fn next_mode_create_multizone_if_no_config() {
        // Checking to ensure set_mode errors when unsupported modes are tried
        let config = AuraConfig::from_default_support(AuraDevice::X19b6, &LaptopLedData::default());
        let supported_modes = LaptopLedData {
            board_name: String::new(),
            layout_name: "ga401".to_owned(),
            basic_modes: vec![AuraModeNum::Static],
            basic_zones: vec![AuraZone::Key1, AuraZone::Key2],
            advanced_type: rog_aura::AdvancedAuraType::None,
            power_zones: vec![PowerZones::Keyboard, PowerZones::RearGlow],
        };
        let mut controller = CtrlKbdLed {
            led_prod: AuraDevice::X19b6,
            led_node: LEDNode::None,
            sysfs_node: KeyboardLed::default(),
            supported_modes,
            per_key_mode_active: false,
            config,
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
