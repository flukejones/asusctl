use std::convert::TryFrom;
use std::ops::{BitAnd, BitOr};
use crate::usb::LedCfgState::{Off, On};

pub const LED_INIT1: [u8; 2] = [0x5d, 0xb9];
pub const LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
pub const LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
pub const LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
pub const LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

// Only these two packets must be 17 bytes
pub const LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const BOOT_MASK:i32 = 0xc31309;
pub const SLEEP_MASK:i32 = 0x300904;
pub const ALL_LEDS_MASK:i32 = 0x000002;
pub const KBD_LEDS_MASK:i32 = 0x080000;
pub const SIDE_LEDS_MASK:i32 = 0x040500;
pub const LEDS_STATE_MASK:i32 = ALL_LEDS_MASK | KBD_LEDS_MASK | SIDE_LEDS_MASK;

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5A, 0xBA, 0xC5, 0xC4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[derive(Clone, Copy)]
pub enum LedCfgState {
    On = 0xffffff,
    Off = 0x0
}

impl From<i32> for LedCfgState {
    fn from(state: i32) -> Self {
        match state {
            0xffffff => On,
            0x0 => Off,
            _ => Off
        }
    }
}

impl From<bool> for LedCfgState {
    fn from(state: bool) -> Self {
        match state {
            true => On,
            false => Off
        }
    }
}

impl TryFrom <[u8; 3]> for LedCfgState {
    type Error = &'static str;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match value {
            [0xff, 0xff, 0xff] => Ok(On),
            [0, 0, 0] => Ok(Off),
            _ => Err("Unconvertible value")
        }
    }
}

impl BitAnd<LedCfgState> for i32 {
    type Output = i32;

    fn bitand(self, rhs: LedCfgState) -> i32 {
        return self & rhs as i32
    }
}
impl BitOr<LedCfgState> for i32 {
    type Output = i32;

    fn bitor(self, rhs: LedCfgState) -> Self::Output {
        return self | rhs as i32
    }
}

impl BitOr<LedCfgState> for LedCfgState {
    type Output = i32;

    fn bitor(self, rhs: LedCfgState) -> i32 {
        return self as i32 | rhs as i32;
    }
}

impl BitAnd<LedCfgState> for LedCfgState {
    type Output = LedCfgState;

    fn bitand(self, rhs: LedCfgState) -> LedCfgState {
        return (self as i32 & rhs as i32).into();
    }
}

pub fn leds_message (boot_state: bool, sleep_state: bool, all_leds_state: bool, kbd_leds_state: bool, side_leds_state: bool) -> [u8; 3] {
    let raw_message = _leds_message(boot_state.into(), sleep_state.into(), all_leds_state.into(), kbd_leds_state.into(), side_leds_state.into());

    let [_, lows @ ..] = i32::to_be_bytes(raw_message);
    return lows;
}

fn _leds_message (boot_state: LedCfgState, sleep_state: LedCfgState, all_leds_state: LedCfgState, kbd_leds_state: LedCfgState, side_leds_state: LedCfgState) -> i32 {

    let full_leds_state = match all_leds_state {
        On => (ALL_LEDS_MASK & all_leds_state) | (KBD_LEDS_MASK & kbd_leds_state) | (SIDE_LEDS_MASK & side_leds_state),
        Off => 0x0100 & side_leds_state,
    };

    let boot_xor_sleep = (BOOT_MASK & boot_state) ^ (SLEEP_MASK & sleep_state);

    return match (all_leds_state | kbd_leds_state | side_leds_state).into() {
        On => boot_xor_sleep ^ ((boot_xor_sleep ^ full_leds_state) & LEDS_STATE_MASK),
        _ => boot_xor_sleep
    }
}
