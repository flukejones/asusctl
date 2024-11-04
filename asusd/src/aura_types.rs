use std::sync::Arc;

use config_traits::{StdConfig, StdConfigLoad};
use dmi_id::DMIID;
use log::{debug, error, info};
use rog_anime::error::AnimeError;
use rog_anime::usb::get_anime_type;
use rog_anime::AnimeType;
use rog_aura::AuraDeviceType;
use rog_platform::hid_raw::HidRaw;
use rog_platform::keyboard_led::KeyboardBacklight;
use rog_platform::usb_raw::USBRaw;
use rog_slash::error::SlashError;
use rog_slash::SlashType;
use tokio::sync::Mutex;

use crate::aura_anime::config::AnimeConfig;
use crate::aura_anime::AnimeMatrix;
use crate::aura_laptop::config::AuraConfig;
use crate::aura_laptop::Aura;
use crate::aura_slash::config::SlashConfig;
use crate::aura_slash::Slash;
use crate::error::RogError;

pub enum _DeviceHandle {
    /// The AniMe devices require USBRaw as they are not HID devices
    Usb(USBRaw),
    HidRaw(HidRaw),
    LedClass(KeyboardBacklight),
    /// TODO
    MulticolourLed,
    None,
}

#[derive(Debug, Clone)]
pub enum DeviceHandle {
    Aura(Aura),
    Slash(Slash),
    /// The AniMe devices require USBRaw as they are not HID devices
    AniMe(AnimeMatrix),
    Ally(Arc<Mutex<HidRaw>>),
    OldAura(Arc<Mutex<HidRaw>>),
    /// TUF laptops have an aditional set of attributes added to the LED /sysfs/
    TufLedClass(Arc<Mutex<HidRaw>>),
    /// TODO
    MulticolourLed,
    None,
}

impl DeviceHandle {
    fn get_slash_type() -> SlashType {
        let dmi = DMIID::new().unwrap_or_default(); // TODO: better error
        let board_name = dmi.board_name;

        if board_name.contains("GA403") {
            SlashType::GA403
        } else if board_name.contains("GA605") {
            SlashType::GA605
        } else if board_name.contains("GU605") {
            SlashType::GU605
        } else {
            SlashType::Unsupported
        }
    }

    pub async fn maybe_slash_hid(device: Arc<Mutex<HidRaw>>, prod_id: &str) -> Option<Self> {
        debug!("Testing for HIDRAW Slash");
        let slash_type = Self::get_slash_type();
        if matches!(slash_type, SlashType::Unsupported)
            || slash_type
                .prod_id_str()
                .to_lowercase()
                .trim_start_matches("0x")
                != prod_id
        {
            log::info!("Unknown or invalid slash: {prod_id:?}, skipping");
            return None;
        }
        info!("Found slash type {slash_type:?}: {prod_id}");

        let mut config = SlashConfig::new().load();
        config.slash_type = slash_type;
        Some(Self::Slash(Slash::new(
            Some(device),
            None,
            Arc::new(Mutex::new(config)),
        )))
    }

    pub fn maybe_slash_usb() -> Result<Self, RogError> {
        debug!("Testing for USB Slash");
        let slash_type = Self::get_slash_type();
        if matches!(slash_type, SlashType::Unsupported) {
            return Err(RogError::Slash(SlashError::NoDevice));
        }

        if let Ok(usb) = USBRaw::new(slash_type.prod_id()) {
            let mut config = SlashConfig::new().load();
            config.slash_type = slash_type;

            info!("Found Slash USB {slash_type:?}");
            Ok(Self::Slash(Slash::new(
                None,
                Some(Arc::new(Mutex::new(usb))),
                Arc::new(Mutex::new(config)),
            )))
        } else {
            Err(RogError::NotFound("No slash device found".to_string()))
        }
    }

    fn get_anime_type() -> AnimeType {
        let dmi = DMIID::new().unwrap_or_default(); // TODO: better error
        let board_name = dmi.board_name;

        if board_name.contains("GA401I") || board_name.contains("GA401Q") {
            AnimeType::GA401
        } else if board_name.contains("GA402R") || board_name.contains("GA402X") {
            AnimeType::GA402
        } else if board_name.contains("GU604V") {
            AnimeType::GU604
        } else {
            AnimeType::Unsupported
        }
    }

    pub async fn maybe_anime_hid(device: Arc<Mutex<HidRaw>>, prod_id: &str) -> Option<Self> {
        debug!("Testing for HIDRAW AniMe");
        let anime_type = Self::get_anime_type();
        dbg!(prod_id);
        if matches!(anime_type, AnimeType::Unsupported) || prod_id != "193b" {
            log::info!("Unknown or invalid AniMe: {prod_id:?}, skipping");
            return None;
        }
        info!("Found AniMe Matrix HIDRAW {anime_type:?}: {prod_id}");

        let mut config = AnimeConfig::new().load();
        config.anime_type = anime_type;
        Some(Self::AniMe(AnimeMatrix::new(
            Some(device),
            None,
            Arc::new(Mutex::new(config)),
        )))
    }

    pub fn maybe_anime_usb() -> Result<Self, RogError> {
        debug!("Testing for USB AniMe");
        let anime_type = get_anime_type();
        if matches!(anime_type, AnimeType::Unsupported) {
            info!("No Anime Matrix capable laptop found");
            return Err(RogError::Anime(AnimeError::NoDevice));
        }

        if let Ok(usb) = USBRaw::new(0x193b) {
            let mut config = AnimeConfig::new().load();
            config.anime_type = anime_type;
            info!("Found AniMe Matrix USB {anime_type:?}");

            Ok(Self::AniMe(AnimeMatrix::new(
                None,
                Some(Arc::new(Mutex::new(usb))),
                Arc::new(Mutex::new(config)),
            )))
        } else {
            Err(RogError::NotFound(
                "No AnimeMatrix device found".to_string(),
            ))
        }
    }

    pub async fn maybe_laptop_aura(device: Arc<Mutex<HidRaw>>, prod_id: &str) -> Option<Self> {
        debug!("Testing for laptop aura");
        let aura_type = AuraDeviceType::from(prod_id);
        if !matches!(
            aura_type,
            AuraDeviceType::LaptopKeyboard2021
                | AuraDeviceType::LaptopKeyboardPre2021
                | AuraDeviceType::LaptopKeyboardTuf
        ) {
            log::info!("Unknown or invalid laptop aura: {prod_id:?}, skipping");
            return None;
        }
        info!("Found laptop aura type {prod_id:?}");

        let backlight = KeyboardBacklight::new()
            .map_err(|e| error!("Keyboard backlight error: {e:?}"))
            .map_or(None, |k| {
                info!("Found sysfs backlight control");
                Some(Arc::new(Mutex::new(k)))
            });

        let mut config = AuraConfig::load_and_update_config(prod_id);
        config.led_type = aura_type;
        Some(Self::Aura(Aura {
            hid: Some(device),
            backlight,
            config: Arc::new(Mutex::new(config)),
        }))
    }
}
