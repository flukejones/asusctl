use std::path::PathBuf;

use log::warn;

use crate::{
    attr_u8, attr_u8_array,
    error::{PlatformError, Result},
    to_device,
};

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct KeyboardLed(PathBuf);

impl KeyboardLed {
    pub fn new() -> Result<Self> {
        let mut enumerator = udev::Enumerator::new().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("enumerator failed".into(), err)
        })?;

        enumerator.match_subsystem("leds").map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("match_subsystem failed".into(), err)
        })?;

        enumerator
            .match_sysname("asus::kbd_backlight")
            .map_err(|err| {
                warn!("{}", err);
                PlatformError::Udev("match_subsystem failed".into(), err)
            })?;

        for device in enumerator.scan_devices().map_err(|err| {
            warn!("{}", err);
            PlatformError::Udev("scan_devices failed".into(), err)
        })? {
            return Ok(Self(device.syspath().to_owned()));
        }
        Err(PlatformError::MissingFunction(
            "asus::kbd_backlight not found".into(),
        ))
    }

    attr_u8!(has_brightness, get_brightness, set_brightness, "brightness");

    attr_u8_array!(
        has_keyboard_rgb_mode,
        get_keyboard_rgb_mode,
        set_keyboard_rgb_mode,
        "kbd_rgb_mode"
    );

    attr_u8_array!(
        has_keyboard_rgb_state,
        get_keyboard_rgb_state,
        set_keyboard_rgb_state,
        "kbd_rgb_state"
    );
}
