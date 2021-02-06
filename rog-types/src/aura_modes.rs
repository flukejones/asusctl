use crate::cli_options::SetAuraBuiltin;
use crate::error::AuraError;
use gumdrop::Options;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

pub const STATIC: u8 = 0x00;
pub const BREATHING: u8 = 0x01;
pub const STROBE: u8 = 0x02;
pub const RAINBOW: u8 = 0x03;
pub const STAR: u8 = 0x04;
pub const RAIN: u8 = 0x05;
pub const HIGHLIGHT: u8 = 0x06;
pub const LASER: u8 = 0x07;
pub const RIPPLE: u8 = 0x08;
pub const PULSE: u8 = 0x0a;
pub const COMET: u8 = 0x0b;
pub const FLASH: u8 = 0x0c;
pub const MULTISTATIC: u8 = 0x0d;
pub const MULTIBREATHE: u8 = 0x0e;
pub const PER_KEY: u8 = 0xff;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Colour(pub u8, pub u8, pub u8);

impl Default for Colour {
    fn default() -> Self {
        Colour(128, 0, 0)
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

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Options, Default, Deserialize, Serialize)]
pub struct SingleSpeed {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "WORD", help = "set the speed: low, med, high")]
    pub speed: Speed,
}
#[derive(Debug, Clone, Options, Default, Deserialize, Serialize)]
pub struct SingleSpeedDirection {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the direction: up, down, left, right")]
    pub direction: Direction,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Default, Options, Deserialize, Serialize)]
pub struct SingleColour {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
}

#[derive(Debug, Clone, Default, Options, Deserialize, Serialize)]
pub struct SingleColourSpeed {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Options, Default, Deserialize, Serialize)]
pub struct TwoColourSpeed {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the first RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, meta = "", help = "set the second RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Default, Options, Deserialize, Serialize)]
pub struct MultiColour {
    #[serde(skip)]
    #[options(help = "print help message")]
    help: bool,
    #[options(short = "a", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour1: Colour,
    #[options(short = "b", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(short = "c", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour3: Colour,
    #[options(short = "d", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour4: Colour,
}

#[derive(Debug, Clone, Default, Options, Deserialize, Serialize)]
pub struct MultiColourSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(short = "a", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour1: Colour,
    #[options(short = "b", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(short = "c", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour3: Colour,
    #[options(short = "d", meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour4: Colour,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AuraModes {
    Static(SingleColour),
    Breathe(TwoColourSpeed),
    Strobe(SingleSpeed),
    Rainbow(SingleSpeedDirection),
    Star(TwoColourSpeed),
    Rain(SingleSpeed),
    Highlight(SingleColourSpeed),
    Laser(SingleColourSpeed),
    Ripple(SingleColourSpeed),
    Pulse(SingleColour),
    Comet(SingleColour),
    Flash(SingleColour),
    MultiStatic(MultiColour),
    MultiBreathe(MultiColourSpeed),
    LedBrightness(u8),
    // TODO: use a serializable structure for this (KeyColourArray)
    PerKey(Vec<Vec<u8>>),
}

impl From<SetAuraBuiltin> for AuraModes {
    fn from(mode: SetAuraBuiltin) -> Self {
        match mode {
            SetAuraBuiltin::Static(x) => AuraModes::Static(x),
            SetAuraBuiltin::Breathe(x) => AuraModes::Breathe(x),
            SetAuraBuiltin::Strobe(x) => AuraModes::Strobe(x),
            SetAuraBuiltin::Rainbow(x) => AuraModes::Rainbow(x),
            SetAuraBuiltin::Star(x) => AuraModes::Star(x),
            SetAuraBuiltin::Rain(x) => AuraModes::Rain(x),
            SetAuraBuiltin::Highlight(x) => AuraModes::Highlight(x),
            SetAuraBuiltin::Laser(x) => AuraModes::Laser(x),
            SetAuraBuiltin::Ripple(x) => AuraModes::Ripple(x),
            SetAuraBuiltin::Pulse(x) => AuraModes::Pulse(x),
            SetAuraBuiltin::Comet(x) => AuraModes::Comet(x),
            SetAuraBuiltin::Flash(x) => AuraModes::Flash(x),
            SetAuraBuiltin::MultiStatic(x) => AuraModes::MultiStatic(x),
            SetAuraBuiltin::MultiBreathe(x) => AuraModes::MultiBreathe(x),
        }
    }
}

/// Very specific mode conversion required because numbering isn't linear
impl From<AuraModes> for u8 {
    fn from(mode: AuraModes) -> Self {
        u8::from(&mode)
    }
}

/// Very specific mode conversion required because numbering isn't linear
impl From<&mut AuraModes> for u8 {
    fn from(mode: &mut AuraModes) -> Self {
        u8::from(&*mode)
    }
}

/// Very specific mode conversion required because numbering isn't linear
impl From<&AuraModes> for u8 {
    fn from(mode: &AuraModes) -> Self {
        match mode {
            AuraModes::Static(_) => STATIC,
            AuraModes::Breathe(_) => BREATHING,
            AuraModes::Strobe(_) => STROBE,
            AuraModes::Rainbow(_) => RAINBOW,
            AuraModes::Star(_) => STAR,
            AuraModes::Rain(_) => RAIN,
            AuraModes::Highlight(_) => HIGHLIGHT,
            AuraModes::Laser(_) => LASER,
            AuraModes::Ripple(_) => RIPPLE,
            AuraModes::Pulse(_) => PULSE,
            AuraModes::Comet(_) => COMET,
            AuraModes::Flash(_) => FLASH,
            AuraModes::MultiStatic(_) => MULTISTATIC,
            AuraModes::MultiBreathe(_) => MULTIBREATHE,
            AuraModes::PerKey(_) => PER_KEY,
            _ => panic!("Invalid mode"),
        }
    }
}

impl From<&AuraModes> for &str {
    fn from(mode: &AuraModes) -> Self {
        match mode {
            AuraModes::Static(_) => "Static",
            AuraModes::Breathe(_) => "Breathing",
            AuraModes::Strobe(_) => "Strobing",
            AuraModes::Rainbow(_) => "Rainbow",
            AuraModes::Star(_) => "Stars",
            AuraModes::Rain(_) => "Rain",
            AuraModes::Highlight(_) => "Keypress Highlight",
            AuraModes::Laser(_) => "Keypress Laser",
            AuraModes::Ripple(_) => "Keypress Ripple",
            AuraModes::Pulse(_) => "Pulse",
            AuraModes::Comet(_) => "Comet",
            AuraModes::Flash(_) => "Flash",
            AuraModes::MultiStatic(_) => "4-Zone Static Colours",
            AuraModes::MultiBreathe(_) => "4-Zone Breathing Colours",
            AuraModes::PerKey(_) => "RGB per-key",
            _ => panic!("Invalid mode"),
        }
    }
}

/// Exists to convert back from correct bytes. PER_KEY byte intentionally left off as it
/// does not correspond to an actual pre-set mode, nor does brightness.
impl From<u8> for AuraModes {
    fn from(byte: u8) -> Self {
        match byte {
            STATIC => AuraModes::Static(SingleColour::default()),
            BREATHING => AuraModes::Breathe(TwoColourSpeed::default()),
            STROBE => AuraModes::Strobe(SingleSpeed::default()),
            RAINBOW => AuraModes::Rainbow(SingleSpeedDirection::default()),
            STAR => AuraModes::Star(TwoColourSpeed::default()),
            RAIN => AuraModes::Rain(SingleSpeed::default()),
            HIGHLIGHT => AuraModes::Highlight(SingleColourSpeed::default()),
            LASER => AuraModes::Laser(SingleColourSpeed::default()),
            RIPPLE => AuraModes::Ripple(SingleColourSpeed::default()),
            PULSE => AuraModes::Pulse(SingleColour::default()),
            COMET => AuraModes::Comet(SingleColour::default()),
            FLASH => AuraModes::Flash(SingleColour::default()),
            MULTISTATIC => AuraModes::MultiStatic(MultiColour::default()),
            MULTIBREATHE => AuraModes::MultiBreathe(MultiColourSpeed::default()),
            PER_KEY => AuraModes::PerKey(vec![]),
            _ => panic!("Invalid mode byte"),
        }
    }
}
