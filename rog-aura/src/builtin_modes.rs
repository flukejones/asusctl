pub const LED_INIT1: [u8; 2] = [0x5d, 0xb9];
pub const LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
pub const LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
pub const LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
pub const LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
#[cfg(feature = "dbus")]
use zvariant_derive::Type;

use crate::{error::Error, LED_MSG_LEN};

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub struct LedPowerStates {
    pub enabled: bool,
    pub sleep_anim_enabled: bool,
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum LedBrightness {
    Off,
    Low,
    Med,
    High,
}

impl LedBrightness {
    pub fn as_char_code(&self) -> u8 {
        std::char::from_digit(*self as u32, 10).unwrap() as u8
    }
}

impl From<u32> for LedBrightness {
    fn from(bright: u32) -> Self {
        match bright {
            0 => LedBrightness::Off,
            1 => LedBrightness::Low,
            2 => LedBrightness::Med,
            3 => LedBrightness::High,
            _ => LedBrightness::Med,
        }
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, PartialEq, Copy, Deserialize, Serialize)]
pub struct Colour(pub u8, pub u8, pub u8);

impl Default for Colour {
    fn default() -> Self {
        Colour(166, 0, 0)
    }
}

impl FromStr for Colour {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 6 {
            return Err(Error::ParseColour);
        }
        let r = u8::from_str_radix(&s[0..2], 16).or(Err(Error::ParseColour))?;
        let g = u8::from_str_radix(&s[2..4], 16).or(Err(Error::ParseColour))?;
        let b = u8::from_str_radix(&s[4..6], 16).or(Err(Error::ParseColour))?;
        Ok(Colour(r, g, b))
    }
}

#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum Speed {
    Low = 0xe1,
    Med = 0xeb,
    High = 0xf5,
}
impl Default for Speed {
    fn default() -> Self {
        Speed::Med
    }
}
impl FromStr for Speed {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "low" => Ok(Speed::Low),
            "med" => Ok(Speed::Med),
            "high" => Ok(Speed::High),
            _ => Err(Error::ParseSpeed),
        }
    }
}

/// Used for Rainbow mode.
///
/// Enum corresponds to the required integer value
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}
impl FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "right" => Ok(Direction::Right),
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            _ => Err(Error::ParseDirection),
        }
    }
}

/// Enum of modes that convert to the actual number required by a USB HID packet
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Deserialize, Serialize)]
pub enum AuraModeNum {
    Static = 0,
    Breathe = 1,
    Strobe = 2,
    Rainbow = 3,
    Star = 4,
    Rain = 5,
    Highlight = 6,
    Laser = 7,
    Ripple = 8,
    Pulse = 10,
    Comet = 11,
    Flash = 12,
}

impl From<&AuraModeNum> for &str {
    fn from(mode: &AuraModeNum) -> Self {
        match mode {
            AuraModeNum::Static => "Static",
            AuraModeNum::Breathe => "Breathing",
            AuraModeNum::Strobe => "Strobing",
            AuraModeNum::Rainbow => "Rainbow",
            AuraModeNum::Star => "Stars",
            AuraModeNum::Rain => "Rain",
            AuraModeNum::Highlight => "Keypress Highlight",
            AuraModeNum::Laser => "Keypress Laser",
            AuraModeNum::Ripple => "Keypress Ripple",
            AuraModeNum::Pulse => "Pulse",
            AuraModeNum::Comet => "Comet",
            AuraModeNum::Flash => "Flash",
        }
    }
}
impl From<&str> for AuraModeNum {
    fn from(mode: &str) -> Self {
        match mode {
            "Static" => AuraModeNum::Static,
            "Breathing" => AuraModeNum::Breathe,
            "Strobing" => AuraModeNum::Strobe,
            "Rainbow" => AuraModeNum::Rainbow,
            "Stars" => AuraModeNum::Star,
            "Rain" => AuraModeNum::Rain,
            "Keypress Highlight" => AuraModeNum::Highlight,
            "Keypress Laser" => AuraModeNum::Laser,
            "Keypress Ripple" => AuraModeNum::Ripple,
            "Pulse" => AuraModeNum::Pulse,
            "Comet" => AuraModeNum::Comet,
            "Flash" => AuraModeNum::Flash,
            _ => AuraModeNum::Static,
        }
    }
}

impl From<u8> for AuraModeNum {
    fn from(mode: u8) -> Self {
        match mode {
            0 => AuraModeNum::Static,
            1 => AuraModeNum::Breathe,
            2 => AuraModeNum::Strobe,
            3 => AuraModeNum::Rainbow,
            4 => AuraModeNum::Star,
            5 => AuraModeNum::Rain,
            6 => AuraModeNum::Highlight,
            7 => AuraModeNum::Laser,
            8 => AuraModeNum::Ripple,
            10 => AuraModeNum::Pulse,
            11 => AuraModeNum::Comet,
            12 => AuraModeNum::Flash,
            _ => AuraModeNum::Static,
        }
    }
}

/// Base effects have no zoning, while multizone is 1-4
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum AuraZone {
    None,
    One,
    Two,
    Three,
    Four,
}

/// Default factory modes structure. This easily converts to an USB HID packet with:
/// ```rust
/// let bytes: [u8; LED_MSG_LEN] = mode.into();
/// ```
#[cfg_attr(feature = "dbus", derive(Type))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuraEffect {
    /// The effect type
    pub mode: AuraModeNum,
    /// `AuraZone::None` for no zone or zoneless keyboards
    pub zone: AuraZone,
    /// Primary colour for all modes
    pub colour1: Colour,
    /// Secondary colour in some modes like Breathing or Stars
    pub colour2: Colour,
    /// One of three speeds for modes that support speed (most that animate)
    pub speed: Speed,
    /// Up, down, left, right. Only Rainbow mode seems to use this
    pub direction: Direction,
}

impl AuraEffect {
    pub fn mode(&self) -> &AuraModeNum {
        &self.mode
    }

    pub fn mode_name(&self) -> String {
        (<&str>::from(&self.mode)).to_string()
    }

    pub fn mode_num(&self) -> u8 {
        self.mode as u8
    }

    pub fn default_with_mode(mode: AuraModeNum) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn zone(&self) -> AuraZone {
        self.zone
    }
}

impl Default for AuraEffect {
    fn default() -> Self {
        Self {
            mode: AuraModeNum::Static,
            zone: AuraZone::None,
            colour1: Colour(166, 0, 0),
            colour2: Colour(0, 0, 0),
            speed: Speed::Med,
            direction: Direction::Right,
        }
    }
}

/// Parses `AuraEffect` in to packet data for writing to the USB interface
///
/// Byte structure where colour is RGB, one byte per R, G, B:
/// ```ignore
/// | 0 | 1 | 2   | 3   | 4, 5, 6 | 7    | 8        | 9 | 10, 11, 12|
/// |---|---|-----|-----|---------|------|----------|---|-----------|
/// |5d |b3 |Zone |Mode |Colour 1 |Speed |Direction |00 |Colour 2   |
/// ```
impl From<&AuraEffect> for [u8; LED_MSG_LEN] {
    fn from(aura: &AuraEffect) -> Self {
        let mut msg = [0u8; LED_MSG_LEN];
        msg[0] = 0x5d;
        msg[1] = 0xb3;
        msg[2] = aura.zone as u8;
        msg[3] = aura.mode as u8;
        msg[4] = aura.colour1.0;
        msg[5] = aura.colour1.1;
        msg[6] = aura.colour1.2;
        msg[7] = aura.speed as u8;
        msg[8] = aura.direction as u8;
        msg[10] = aura.colour2.0;
        msg[11] = aura.colour2.1;
        msg[12] = aura.colour2.2;
        msg
    }
}
