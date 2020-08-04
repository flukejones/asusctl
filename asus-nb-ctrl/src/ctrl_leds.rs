// Only these two packets must be 17 bytes
static LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

use crate::{config::Config, error::RogError};
use asus_nb::{
    aura_brightness_bytes, aura_modes::AuraModes, fancy::KeyColourArray, DBUS_IFACE, DBUS_PATH,
    LED_MSG_LEN,
};
use dbus::{channel::Sender, nonblock::SyncConnection, tree::Signal};
use log::{info, warn};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct CtrlKbdBacklight {
    dev_node: String,
    supported_modes: Vec<u8>,
    flip_effect_write: bool,
}

use async_trait::async_trait;

#[async_trait]
impl crate::Controller for CtrlKbdBacklight {
    type A = AuraModes;

    /// Spawns two tasks which continuously check for changes
    fn spawn_task(
        mut self,
        config: Arc<Mutex<Config>>,
        mut recv: Receiver<Self::A>,
        connection: Option<Arc<SyncConnection>>,
        signal: Option<Arc<Signal<()>>>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(command) = recv.recv().await {
                let mut config = config.lock().await;
                match &command {
                    AuraModes::RGB(_) => {
                        self.do_command(command, &mut config)
                            .await
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                    _ => {
                        let json = serde_json::to_string(&command).unwrap();
                        self.do_command(command, &mut config)
                            .await
                            .unwrap_or_else(|err| warn!("{}", err));
                        connection
                            .as_ref()
                            .expect("LED Controller must have DBUS connection")
                            .send(
                                signal
                                    .as_ref()
                                    .expect("LED Controller must have DBUS signal")
                                    .msg(&DBUS_PATH.into(), &DBUS_IFACE.into())
                                    .append1(json),
                            )
                            .unwrap_or_else(|_| 0);
                    }
                }
            }
        })
    }

    async fn reload_from_config(&mut self, config: &mut Config) -> Result<(), Box<dyn Error>> {
        // set current mode (if any)
        if self.supported_modes.len() > 1 {
            if self.supported_modes.contains(&config.kbd_backlight_mode) {
                let mode = config
                    .get_led_mode_data(config.kbd_backlight_mode)
                    .ok_or(RogError::NotSupported)?
                    .to_owned();
                self.write_mode(&mode).await?;
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
                self.write_mode(&mode).await?;
                info!("Reloaded last used mode");
            }
        }

        // Reload brightness
        let bright = config.kbd_boot_brightness;
        let bytes = aura_brightness_bytes(bright);
        self.write_bytes(&bytes).await?;
        info!("Reloaded last used brightness");
        Ok(())
    }
}

impl CtrlKbdBacklight {
    #[inline]
    pub fn new(id_product: &str, supported_modes: Vec<u8>) -> Result<Self, std::io::Error> {
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("hidraw")?;

        for device in enumerator.scan_devices()? {
            if let Some(parent) = device.parent_with_subsystem_devtype("usb", "usb_device")? {
                if parent.attribute_value("idProduct").unwrap() == id_product
                // && device.parent().unwrap().sysnum().unwrap() == 3
                {
                    if let Some(dev_node) = device.devnode() {
                        info!("Device has keyboard backlight control");
                        info!("Using device at: {:?} for LED control", dev_node);
                        return Ok(CtrlKbdBacklight {
                            dev_node: dev_node.to_string_lossy().to_string(),
                            supported_modes,
                            flip_effect_write: false,
                        });
                    }
                }
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Device node not found",
        ))
    }

    pub async fn do_command(
        &mut self,
        mode: AuraModes,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        self.set_and_save(mode, config).await
    }

    /// Should only be used if the bytes you are writing are verified correct
    #[inline]
    async fn write_bytes(&self, message: &[u8]) -> Result<(), Box<dyn Error>> {
        if let Ok(mut file) = OpenOptions::new().write(true).open(&self.dev_node) {
            file.write_all(message).unwrap();
            return Ok(());
        }
        Err(Box::new(RogError::NotSupported))
    }

    /// Write an effect block
    #[inline]
    async fn write_effect(&mut self, effect: &[Vec<u8>]) -> Result<(), Box<dyn Error>> {
        if self.flip_effect_write {
            for row in effect.iter().rev() {
                self.write_bytes(row).await?;
            }
        } else {
            for row in effect.iter() {
                self.write_bytes(row).await?;
            }
        }
        self.flip_effect_write = !self.flip_effect_write;
        Ok(())
    }

    /// Used to set a builtin mode and save the settings for it
    ///
    /// This needs to be universal so that settings applied by dbus stick
    #[inline]
    async fn set_and_save(
        &mut self,
        mode: AuraModes,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        match mode {
            AuraModes::LedBrightness(n) => {
                let bytes: [u8; LED_MSG_LEN] = (&mode).into();
                self.write_bytes(&bytes).await?;
                config.kbd_boot_brightness = n;
                config.write();
                info!("LED brightness set to {:#?}", n);
            }
            AuraModes::RGB(v) => {
                if v.is_empty() || v[0].is_empty() {
                    let bytes = KeyColourArray::get_init_msg();
                    self.write_bytes(&bytes).await?;
                } else {
                    self.write_effect(&v).await?;
                }
            }
            _ => {
                let mode_num: u8 = u8::from(&mode);
                self.write_mode(&mode).await?;
                config.kbd_backlight_mode = mode_num;
                config.set_mode_data(mode);
                config.write();
            }
        }
        Ok(())
    }

    #[inline]
    async fn write_mode(&mut self, mode: &AuraModes) -> Result<(), Box<dyn Error>> {
        match mode {
            AuraModes::RGB(v) => {
                if v.is_empty() || v[0].is_empty() {
                    let bytes = KeyColourArray::get_init_msg();
                    self.write_bytes(&bytes).await?;
                } else {
                    self.write_effect(v).await?;
                }
            }
            _ => {
                let mode_num: u8 = u8::from(mode);
                match mode {
                    AuraModes::MultiStatic(_) => {
                        if self.supported_modes.contains(&mode_num) {
                            let bytes: [[u8; LED_MSG_LEN]; 4] = mode.into();
                            for array in bytes.iter() {
                                self.write_bytes(array).await?;
                            }
                        }
                    }
                    _ => {
                        if self.supported_modes.contains(&mode_num) {
                            let bytes: [u8; LED_MSG_LEN] = mode.into();
                            self.write_bytes(&bytes).await?;
                        }
                    }
                }
                self.write_bytes(&LED_SET).await?;
                // Changes won't persist unless apply is set
                self.write_bytes(&LED_APPLY).await?;
            }
        }
        Ok(())
    }

    // #[inline]
    // pub async fn reload_from_config(&mut self, config: &mut Config) -> Result<(), RogError> {

    // }
}
