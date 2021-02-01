pub static DBUS_NAME: &str = "org.asuslinux.Daemon";
pub static DBUS_PATH: &str = "/org/asuslinux/Daemon";
pub static DBUS_IFACE: &str = "org.asuslinux.Daemon";
pub const LED_MSG_LEN: usize = 17;

pub mod aura_modes;
use aura_modes::AuraModes;

pub mod profile;

/// Contains mostly only what is required for parsing CLI options
pub mod cli_options;

/// Enables you to create fancy RGB effects
pub mod fancy;

/// The main dbus group for system controls, e.g, fan control, keyboard LED's
pub mod core_dbus;

/// Specific dbus for writing to the AniMe Matrix display (if supported)
pub mod anime_dbus;

/// Helper functions for the AniMe display
pub mod anime_matrix;

pub mod error;

pub mod dbus_anime;
pub mod dbus_charge;
pub mod dbus_gfx;
pub mod dbus_ledmode;
pub mod dbus_profile;
pub mod dbus_rogbios;
pub mod dbus_supported;

// static LED_INIT1: [u8; 2] = [0x5d, 0xb9];
// static LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
// static LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
// static LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
// static LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

/// Writes aout the correct byte string for brightness
///
/// The HID descriptor looks like:
///
/// ```ignore
/// 0x06, 0x31, 0xFF,  // Usage Page (Vendor Defined 0xFF31)
/// 0x09, 0x76,        // Usage (0x76)
/// 0xA1, 0x01,        // Collection (Application)
/// 0x85, 0x5A,        //   Report ID (90)
/// 0x19, 0x00,        //   Usage Minimum (0x00)
/// 0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
/// 0x15, 0x00,        //   Logical Minimum (0)
/// 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
/// 0x75, 0x08,        //   Report Size (8)
/// 0x95, 0x05,        //   Report Count (5)
/// 0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
/// 0x19, 0x00,        //   Usage Minimum (0x00)
/// 0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
/// 0x15, 0x00,        //   Logical Minimum (0)
/// 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
/// 0x75, 0x08,        //   Report Size (8)
/// 0x95, 0x3F,        //   Report Count (63)
/// 0xB1, 0x00,        //   Feature (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
/// 0xC0,              // End Collection
/// ```
pub fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    // TODO: check brightness range
    [
        0x5A, 0xBA, 0xC5, 0xC4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

/// Parses `AuraCommands` in to packet data
///
/// Byte structure:
///
/// ```ignore
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10| 11| 12|
/// |---|---|---|---|---|---|---|---|---|---|---|---|---|
/// |5d |b3 |00 |03 |ff |00 |00 |00 |00 |00 |00 |ff |00 |
/// ```
///
/// Bytes 0 and 1 should always be 5d, b3
///
/// On multizone laptops byte 2 is the zone number, RGB in usual
/// place, byte 3 set to zero
///
/// Byte 3 sets the mode type:
/// - 00 = static
/// - 01 = breathe (can set two colours)
/// - 02 = strobe (through all colours)
/// - 03 = rainbow
/// - 04 = star (byte 9 sets rain colour)
/// - 05 = rain keys, red, white, turquoise
/// - 06 = pressed keys light up and fade
/// - 07 = pressed key emits laser
/// - 08 = pressed key emits water ripple
/// - 09 = no effect/not used
/// - 0a fast pulse (no speed setting)
/// - 0b vertical line racing to right (no speed setting)
/// - 0c wider vertical line racing to right (no speed setting)
///
/// Bytes 4, 5, 6 are Red, Green, Blue
///
/// Byte 7 sets speed from
/// - 0x00 = Off
/// - 0xe1 = Slow
/// - 0xeb = Medium
/// - 0xf5 = Fast
///
/// Byte 8 sets rainbow direction:
/// - 0x00 = rightwards
/// - 0x01 = leftwards
/// - 0x02 = upwards
/// - 0x03 = downwards
///
/// Bytes 10, 11, 12 are Red, Green, Blue for second colour if mode supports it
///
/// The HID descriptor looks like:
/// ```ignore
/// 0x06, 0x31, 0xFF,  // Usage Page (Vendor Defined 0xFF31)
/// 0x09, 0x79,        // Usage (0x79)
/// 0xA1, 0x01,        // Collection (Application)
/// 0x85, 0x5D,        //   Report ID (93)
/// 0x19, 0x00,        //   Usage Minimum (0x00)
/// 0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
/// 0x15, 0x00,        //   Logical Minimum (0)
/// 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
/// 0x75, 0x08,        //   Report Size (8)
/// 0x95, 0x1F,        //   Report Count (31)
/// 0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
/// 0x19, 0x00,        //   Usage Minimum (0x00)
/// 0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
/// 0x15, 0x00,        //   Logical Minimum (0)
/// 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
/// 0x75, 0x08,        //   Report Size (8)
/// 0x95, 0x3F,        //   Report Count (63)
/// 0x91, 0x00,        //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
/// 0x19, 0x00,        //   Usage Minimum (0x00)
/// 0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
/// 0x15, 0x00,        //   Logical Minimum (0)
/// 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
/// 0x75, 0x08,        //   Report Size (8)
/// 0x95, 0x3F,        //   Report Count (63)
/// 0xB1, 0x00,        //   Feature (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
/// 0xC0,              // End Collection
/// ```
///
/// This descriptor is also used for the per-key LED settings
impl From<&AuraModes> for [u8; LED_MSG_LEN] {
    fn from(mode: &AuraModes) -> Self {
        let mut msg = [0u8; LED_MSG_LEN];
        msg[0] = 0x5d;
        msg[1] = 0xb3;
        msg[7] = 0xeb;
        match mode {
            AuraModes::LedBrightness(n) => return aura_brightness_bytes(*n),
            AuraModes::Static(_) => msg[3] = 0x00,
            AuraModes::Breathe(_) => msg[3] = 0x01,
            AuraModes::Strobe(_) => msg[3] = 0x02,
            AuraModes::Rainbow(_) => msg[3] = 0x03,
            AuraModes::Star(_) => msg[3] = 0x04,
            AuraModes::Rain(_) => msg[3] = 0x05,
            AuraModes::Highlight(_) => msg[3] = 0x06,
            AuraModes::Laser(_) => msg[3] = 0x07,
            AuraModes::Ripple(_) => msg[3] = 0x08,
            AuraModes::Pulse(_) => msg[3] = 0x0a,
            AuraModes::Comet(_) => msg[3] = 0x0b,
            AuraModes::Flash(_) => msg[3] = 0x0c,
            _ => panic!("Mode not convertable to 1D array: {}", <&str>::from(mode)),
        }

        match mode {
            AuraModes::Rainbow(settings) => {
                msg[7] = settings.speed as u8;
                msg[8] = settings.direction as u8;
            }
            AuraModes::Star(settings) => {
                msg[4] = settings.colour.0;
                msg[5] = settings.colour.1;
                msg[6] = settings.colour.2;
                msg[7] = settings.speed as u8;
                msg[9] = settings.colour2.2;
            }
            AuraModes::Breathe(settings) => {
                msg[4] = settings.colour.0;
                msg[5] = settings.colour.1;
                msg[6] = settings.colour.2;
                msg[7] = settings.speed as u8;
                msg[10] = settings.colour2.0;
                msg[11] = settings.colour2.1;
                msg[12] = settings.colour2.2;
            }
            AuraModes::Strobe(settings) | AuraModes::Rain(settings) => {
                msg[7] = settings.speed as u8;
            }
            AuraModes::Highlight(settings)
            | AuraModes::Laser(settings)
            | AuraModes::Ripple(settings) => {
                msg[4] = settings.colour.0;
                msg[5] = settings.colour.1;
                msg[6] = settings.colour.2;
                msg[7] = settings.speed as u8;
            }
            AuraModes::Static(settings)
            | AuraModes::Pulse(settings)
            | AuraModes::Comet(settings)
            | AuraModes::Flash(settings) => {
                msg[4] = settings.colour.0;
                msg[5] = settings.colour.1;
                msg[6] = settings.colour.2;
            }
            _ => panic!("Mode not convertable to 1D array: {}", <&str>::from(mode)),
        }
        msg
    }
}

impl From<AuraModes> for [u8; LED_MSG_LEN] {
    #[inline]
    fn from(mode: AuraModes) -> Self {
        <[u8; LED_MSG_LEN]>::from(&mode)
    }
}

impl From<AuraModes> for [[u8; LED_MSG_LEN]; 4] {
    #[inline]
    fn from(mode: AuraModes) -> Self {
        <[[u8; LED_MSG_LEN]; 4]>::from(&mode)
    }
}

impl From<&AuraModes> for [[u8; LED_MSG_LEN]; 4] {
    #[inline]
    fn from(mode: &AuraModes) -> Self {
        let mut msg = [[0u8; LED_MSG_LEN]; 4];
        match mode {
            AuraModes::MultiStatic(settings) => {
                for (i, row) in msg.iter_mut().enumerate() {
                    row[0] = 0x5d;
                    row[1] = 0xb3;
                    row[2] = i as u8 + 1; // zone
                    row[3] = 0x00; // mode
                    row[7] = 0xeb; // static needs speed?
                }
                msg[0][4] = settings.colour1.0;
                msg[0][5] = settings.colour1.1;
                msg[0][6] = settings.colour1.2;
                msg[1][4] = settings.colour2.0;
                msg[1][5] = settings.colour2.1;
                msg[1][6] = settings.colour2.2;
                msg[2][4] = settings.colour3.0;
                msg[2][5] = settings.colour3.1;
                msg[2][6] = settings.colour3.2;
                msg[3][4] = settings.colour4.0;
                msg[3][5] = settings.colour4.1;
                msg[3][6] = settings.colour4.2;
            }
            AuraModes::MultiBreathe(settings) => {
                for (i, row) in msg.iter_mut().enumerate() {
                    row[0] = 0x5d;
                    row[1] = 0xb3;
                    row[2] = i as u8 + 1; // zone
                    row[3] = 0x01; // mode
                }
                let speed = match settings.speed {
                    aura_modes::Speed::Low => 0xfd,
                    aura_modes::Speed::Med => 0xfe,
                    aura_modes::Speed::High => 0xff,
                };
                msg[0][4] = settings.colour1.0;
                msg[0][5] = settings.colour1.1;
                msg[0][6] = settings.colour1.2;
                msg[0][7] = speed; // fd, fe, ff
                msg[1][4] = settings.colour2.0;
                msg[1][5] = settings.colour2.1;
                msg[1][6] = settings.colour2.2;
                msg[1][7] = speed;
                msg[2][4] = settings.colour3.0;
                msg[2][5] = settings.colour3.1;
                msg[2][6] = settings.colour3.2;
                msg[2][7] = speed;
                msg[3][4] = settings.colour4.0;
                msg[3][5] = settings.colour4.1;
                msg[3][6] = settings.colour4.2;
                msg[3][7] = speed;
            }
            _ => panic!("Mode not convertable to 2D array: {}", <&str>::from(mode)),
        }
        msg
    }
}
