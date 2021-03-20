// static LED_INIT1: [u8; 2] = [0x5d, 0xb9];
// static LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
// static LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
// static LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
// static LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

use crate::LED_MSG_LEN;
use crate::error::AuraError;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use zvariant_derive::Type;

#[derive(Debug, Clone, PartialEq, Copy, Deserialize, Serialize, Type)]
pub struct Colour(pub u8, pub u8, pub u8);

impl Default for Colour {
    fn default() -> Self {
        Colour(166, 0, 0)
    }
}

impl FromStr for Colour {
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 6 {
            return Err(AuraError::ParseColour);
        }
        let r = u8::from_str_radix(&s[0..2], 16).or(Err(AuraError::ParseColour))?;
        let g = u8::from_str_radix(&s[2..4], 16).or(Err(AuraError::ParseColour))?;
        let b = u8::from_str_radix(&s[4..6], 16).or(Err(AuraError::ParseColour))?;
        Ok(Colour(r, g, b))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Type)]
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
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "low" => Ok(Speed::Low),
            "med" => Ok(Speed::Med),
            "high" => Ok(Speed::High),
            _ => Err(AuraError::ParseSpeed),
        }
    }
}

/// Used for Rainbow mode.
///
/// Enum corresponds to the required integer value
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Type)]
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
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "right" => Ok(Direction::Right),
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            _ => Err(AuraError::ParseDirection),
        }
    }
}

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5A, 0xBA, 0xC5, 0xC4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[derive(
    Debug, Type, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Deserialize, Serialize,
)]
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

#[derive(Deserialize, Serialize)]
pub struct AuraMultiZone {
    static_: [AuraEffect; 4],
    breathe: [AuraEffect; 4],
}

impl AuraMultiZone {
    pub fn set(&mut self, effect: AuraEffect) {
        if effect.mode == AuraModeNum::Static {
            match effect.zone {
                AuraZone::None => {}
                AuraZone::One => self.static_[0] = effect,
                AuraZone::Two => self.static_[1] = effect,
                AuraZone::Three => self.static_[2] = effect,
                AuraZone::Four => self.static_[3] = effect,
            }
        } else if effect.mode == AuraModeNum::Breathe {
            match effect.zone {
                AuraZone::None => {}
                AuraZone::One => self.breathe[0] = effect,
                AuraZone::Two => self.breathe[1] = effect,
                AuraZone::Three => self.breathe[2] = effect,
                AuraZone::Four => self.breathe[3] = effect,
            }
        }
    }

    pub fn static_(&self) -> &[AuraEffect; 4] {
        &self.static_
    }

    pub fn breathe(&self) -> &[AuraEffect; 4] {
        &self.breathe
    }
}

impl Default for AuraMultiZone {
    fn default() -> Self {
        Self {
            static_: [
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::One,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Two,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Three,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Static,
                    zone: AuraZone::Four,
                    ..Default::default()
                },
            ],
            breathe: [
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::One,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Two,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Three,
                    ..Default::default()
                },
                AuraEffect {
                    mode: AuraModeNum::Breathe,
                    zone: AuraZone::Four,
                    ..Default::default()
                },
            ],
        }
    }
}

/// Base effects have no zoning, while multizone is 1-4
#[derive(Debug, Type, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum AuraZone {
    None,
    One,
    Two,
    Three,
    Four,
}

/// Default factory modes structure
#[derive(Debug, Type, Clone, Deserialize, Serialize)]
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
/// Byte structure:
/// ```ignore
/// | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10| 11| 12|
/// |---|---|---|---|---|---|---|---|---|---|---|---|---|
/// |5d |b3 |00 |03 |ff |00 |00 |00 |00 |00 |00 |ff |00 |
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
