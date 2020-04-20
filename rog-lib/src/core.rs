// Return show-stopping errors, otherwise map error to a log level

use crate::{aura::BuiltInModeByte, config::Config, error::AuraError, laptops::*};
use aho_corasick::AhoCorasick;
use gumdrop::Options;
use log::{debug, warn};
use rusb::DeviceHandle;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use sysfs_class::{Brightness, SysClass};
use uhid_virt::{Bus, CreateParams, UHIDDevice};

pub const LED_MSG_LEN: usize = 17;
static LED_INIT1: [u8; 2] = [0x5d, 0xb9];
static LED_INIT2: &'static str = "]ASUS Tech.Inc."; // ] == 0x5d
static LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
static LED_INIT4: &'static str = "^ASUS Tech.Inc."; // ^ == 0x5e
static LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

// Only these two packets must be 17 bytes
static LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// ROG device controller
///
/// For the GX502GW the LED setup sequence looks like:
///
/// -` LED_INIT1`
/// - `LED_INIT3`
/// - `LED_INIT4`
/// - `LED_INIT2`
/// - `LED_INIT4`
pub struct RogCore {
    handle: DeviceHandle<rusb::GlobalContext>,
    initialised: bool,
    led_iface_num: u8,
    keys_endpoint: u8,
    config: Config,
    virt_keys: VirtKeys,
}

impl RogCore {
    pub fn new(laptop: &dyn Laptop) -> Result<RogCore, AuraError> {
        let mut dev_handle = RogCore::get_device(laptop.usb_vendor(), laptop.usb_product())?;
        dev_handle.set_active_configuration(0).unwrap_or(());

        let dev_config = dev_handle.device().config_descriptor(0).unwrap();
        // Interface with outputs
        let mut led_interface_num = 0;
        let mut keys_interface_num = 0;
        let keys_endpoint = 0x83;
        for iface in dev_config.interfaces() {
            for desc in iface.descriptors() {
                for endpoint in desc.endpoint_descriptors() {
                    if endpoint.address() == keys_endpoint {
                        keys_interface_num = desc.interface_number();
                    } else if endpoint.address() == laptop.led_iface_num() {
                        led_interface_num = desc.interface_number();
                        break;
                    }
                }
            }
        }

        dev_handle.set_auto_detach_kernel_driver(true).unwrap();
        dev_handle
            .claim_interface(keys_interface_num)
            .map_err(|err| AuraError::UsbError(err))?;

        Ok(RogCore {
            handle: dev_handle,
            initialised: false,
            led_iface_num: led_interface_num,
            keys_endpoint,
            config: Config::default().read(),
            virt_keys: VirtKeys::new(),
        })
    }

    pub fn virt_keys(&mut self) -> &mut VirtKeys {
        &mut self.virt_keys
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    fn get_device(
        vendor: u16,
        product: u16,
    ) -> Result<DeviceHandle<rusb::GlobalContext>, AuraError> {
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();
            if device_desc.vendor_id() == vendor && device_desc.product_id() == product {
                return device.open().map_err(|err| AuraError::UsbError(err));
            }
        }
        Err(AuraError::UsbError(rusb::Error::NoDevice))
    }

    fn aura_write(&mut self, message: &[u8]) -> Result<(), AuraError> {
        self.handle
            .write_control(0x21, 0x09, 0x035D, 0, message, Duration::new(0, 5))
            .map_err(|err| AuraError::UsbError(err))?;
        Ok(())
    }

    fn aura_write_messages(&mut self, messages: &[&[u8]]) -> Result<(), AuraError> {
        self.handle
            .claim_interface(self.led_iface_num)
            .map_err(|err| AuraError::UsbError(err))?;

        if !self.initialised {
            self.aura_write(&LED_INIT1)?;
            self.aura_write(LED_INIT2.as_bytes())?;
            self.aura_write(&LED_INIT3)?;
            self.aura_write(LED_INIT4.as_bytes())?;
            self.aura_write(&LED_INIT5)?;
            self.initialised = true;
        }

        for message in messages {
            self.aura_write(*message)?;
            self.aura_write(&LED_SET)?;
        }
        // Changes won't persist unless apply is set
        self.aura_write(&LED_APPLY)?;

        self.handle
            .release_interface(self.led_iface_num)
            .map_err(|err| AuraError::UsbError(err))?;
        Ok(())
    }

    pub fn aura_brightness_bytes(brightness: u8) -> Result<[u8; 17], AuraError> {
        // TODO: check brightness range
        let mut bright = [0u8; LED_MSG_LEN];
        bright[0] = 0x5a;
        bright[1] = 0xba;
        bright[2] = 0xc5;
        bright[3] = 0xc4;
        bright[4] = brightness;
        Ok(bright)
    }

    pub fn aura_set_and_save(
        &mut self,
        supported_modes: &[BuiltInModeByte],
        bytes: &[u8],
    ) -> Result<(), AuraError> {
        let mode = BuiltInModeByte::from(bytes[3]);
        if supported_modes.contains(&mode) || bytes[1] == 0xba {
            let messages = [bytes];
            self.aura_write_messages(&messages)?;
            self.config.set_field_from(bytes);
            self.config.write();
            return Ok(());
        }
        warn!("{:?} not supported", BuiltInModeByte::from(mode));
        Err(AuraError::NotSupported)
    }

    pub fn poll_keyboard(
        &mut self,
        hotkey_group_bytes: &[u8],
        buf: &mut [u8; 32],
    ) -> Result<Option<usize>, AuraError> {
        let res =
            match self
                .handle
                .read_interrupt(self.keys_endpoint, buf, Duration::from_micros(1))
            {
                Ok(o) => {
                    if hotkey_group_bytes.contains(&buf[0]) {
                        Ok(Some(o))
                    } else {
                        Ok(None)
                    }
                }
                Err(err) => Err(AuraError::UsbError(err)),
            };
        res
    }

    pub fn suspend(&self) {
        std::process::Command::new("systemctl")
            .arg("suspend")
            .spawn()
            .map_or_else(|err| warn!("Failed to suspend: {}", err), |_| {});
    }

    pub fn toggle_airplane_mode(&self) {
        match Command::new("rfkill").arg("list").output() {
            Ok(output) => {
                if output.status.success() {
                    let patterns = &["yes"];
                    let ac = AhoCorasick::new(patterns);
                    if ac.earliest_find(output.stdout).is_some() {
                        Command::new("rfkill")
                            .arg("unblock")
                            .arg("all")
                            .spawn()
                            .map_or_else(
                                |err| warn!("Could not unblock rf devices: {}", err),
                                |_| {},
                            );
                    } else {
                        let _ = Command::new("rfkill")
                            .arg("block")
                            .arg("all")
                            .spawn()
                            .map_or_else(
                                |err| warn!("Could not block rf devices: {}", err),
                                |_| {},
                            );
                    }
                } else {
                    warn!("Could not list rf devices");
                }
            }
            Err(err) => {
                warn!("Could not list rf devices: {}", err);
            }
        }
    }
}

pub struct Backlight {
    backlight: sysfs_class::Backlight,
    step: u64,
    max: u64,
}

impl Backlight {
    pub fn new(id: &str) -> Result<Backlight, std::io::Error> {
        for bl in sysfs_class::Backlight::iter() {
            let bl = bl?;
            if bl.id() == id {
                let max = bl.max_brightness()?;
                let step = max / 50;
                return Ok(Backlight {
                    backlight: bl,
                    step,
                    max,
                });
            }
        }
        panic!("Backlight not found")
    }
    pub fn step_up(&self) {
        let brightness = self
            .backlight
            .brightness()
            .map_err(|err| warn!("Failed to fetch backlight level: {}", err))
            .unwrap();
        if brightness + self.step <= self.max {
            self.backlight
                .set_brightness(brightness + self.step)
                .map_or_else(
                    |err| warn!("Failed to increment backlight level: {}", err),
                    |_| {},
                );
        }
    }
    pub fn step_down(&self) {
        let brightness = self
            .backlight
            .brightness()
            .map_err(|err| warn!("Failed to fetch backlight level: {}", err))
            .unwrap();
        if brightness > self.step {
            self.backlight
                .set_brightness(brightness - self.step)
                .map_or_else(
                    |err| warn!("Failed to increment backlight level: {}", err),
                    |_| {},
                );
        }
    }
}

#[derive(Debug, Options)]
pub struct LedBrightness {
    level: u8,
}
impl LedBrightness {
    pub fn level(&self) -> u8 {
        self.level
    }
}
impl FromStr for LedBrightness {
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "off" => Ok(LedBrightness { level: 0x00 }),
            "low" => Ok(LedBrightness { level: 0x01 }),
            "med" => Ok(LedBrightness { level: 0x02 }),
            "high" => Ok(LedBrightness { level: 0x03 }),
            _ => {
                println!("Missing required argument, must be one of:\noff,low,med,high\n");
                Err(AuraError::ParseBrightness)
            }
        }
    }
}

pub struct VirtKeys {
    pub device: UHIDDevice<std::fs::File>,
}

impl VirtKeys {
    pub fn new() -> Self {
        VirtKeys {
            device: UHIDDevice::create(CreateParams {
                name: String::from("Virtual ROG buttons"),
                phys: String::from(""),
                uniq: String::from(""),
                bus: Bus::USB,
                vendor: 0x0b05,
                product: 0x1866,
                version: 0,
                country: 0,
                rd_data: CONSUMER.to_vec(),
            })
            .unwrap(),
        }
    }

    pub fn press(&mut self, input: [u8; 2]) {
        let mut bytes = [0u8; 8];
        bytes[0] = 0x02;
        bytes[1] = input[0];
        bytes[2] = input[1];
        self.device.write(&bytes).unwrap();
        bytes[1] = 0;
        bytes[2] = 0;
        self.device.write(&bytes).unwrap();
    }
}

pub const CONSUMER: [u8; 25] = [
    0x05, 0x0C, // Usage Page (Consumer)
    0x09, 0x01, // Usage (Consumer Control)
    0xA1, 0x01, // Collection (Application)
    0x85, 0x02, //   Report ID (2)
    0x19, 0x00, //   Usage Minimum (Unassigned)
    0x2A, 0x3C, 0x02, //   Usage Maximum (AC Format)
    0x15, 0x00, //   Logical Minimum (0)
    0x26, 0x3C, 0x02, //   Logical Maximum (572)
    0x75, 0x10, //   Report Size (16)
    0x95, 0x02, //   Report Count (2)
    0x81, 0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0,
];

// Usage 04 for microphone
// Needs another Usage (80) for system control
// B5 for toggle int/ext display
// b2 for external
#[derive(Copy, Clone)]
pub enum ConsumerKeys {
    VolUp = 0x0e9,     // USAGE (Volume up)
    VolDown = 0x0ea,   // USAGE (Volume down)
    VolMute = 0x0e2,   // USAGE (Volume mute)
    TrackNext = 0x0b6, // USAGE (Track next)
    PlayToggl = 0x0cd, // USAGE (Play/Pause)
    TrackPrev = 0x0b5, // USAGE (Track prev)
    TrackStop = 0x0b7,
    Power = 0x030,
    Reset = 0x031,
    Sleep = 0x032,        // USAGE (Sleep)
    BacklightInc = 0x06f, // USAGE (Backlight Inc)
    BacklightDec = 0x070, // USAGE (Backlight Dec)
    BacklightTog = 0x072, // USAGE (Backlight toggle? display toggle?)
    Present = 0x188,
}

impl From<ConsumerKeys> for [u8; 2] {
    fn from(key: ConsumerKeys) -> Self {
        let mut bytes = [0u8; 2];
        bytes[0] = key as u8;
        bytes[1] = (key as u16 >> 8) as u8;
        bytes
    }
}