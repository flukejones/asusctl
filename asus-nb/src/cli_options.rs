use crate::error::AuraError;
use gumdrop::Options;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Options)]
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

#[derive(Deserialize, Serialize)]
pub struct Colour(pub u8, pub u8, pub u8);
impl Default for Colour {
    fn default() -> Self {
        Colour(255, 0, 0)
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

#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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

#[derive(Default, Options, Deserialize, Serialize)]
pub struct TwoColourSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "HEX", help = "set the first RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, meta = "HEX", help = "set the second RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(no_long, help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Default, Options, Deserialize, Serialize)]
pub struct SingleSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "WORD", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Default, Options, Deserialize, Serialize)]
pub struct SingleColour {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
}

#[derive(Default, Options, Deserialize, Serialize)]
pub struct MultiColour {
    #[options(help = "print help message")]
    help: bool,
    #[options(meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour1: Colour,
    #[options(meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour3: Colour,
    #[options(meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour4: Colour,
}

#[derive(Default, Options, Deserialize, Serialize)]
pub struct SingleSpeedDirection {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        no_long,
        meta = "DIR",
        help = "set the direction: up, down, left, right"
    )]
    pub direction: Direction,
    #[options(no_long, help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Default, Options, Deserialize, Serialize)]
pub struct SingleColourSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "HEX", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, help = "set the speed: low, med, high")]
    pub speed: Speed,
}

/// Byte value for setting the built-in mode.
///
/// Enum corresponds to the required integer value
#[derive(Options, Deserialize, Serialize)]
pub enum SetAuraBuiltin {
    #[options(help = "set a single static colour")]
    Stable(SingleColour),
    #[options(help = "pulse between one or two colours")]
    Breathe(TwoColourSpeed),
    #[options(help = "strobe through all colours")]
    Strobe(SingleSpeed),
    #[options(help = "rainbow cycling in one of four directions")]
    Rainbow(SingleSpeedDirection),
    #[options(help = "rain pattern mimicking raindrops")]
    Star(TwoColourSpeed),
    #[options(help = "rain pattern of three preset colours")]
    Rain(SingleSpeed),
    #[options(help = "pressed keys are highlighted to fade")]
    Highlight(SingleColourSpeed),
    #[options(help = "pressed keys generate horizontal laser")]
    Laser(SingleColourSpeed),
    #[options(help = "pressed keys ripple outwards like a splash")]
    Ripple(SingleColourSpeed),
    #[options(help = "set a rapid pulse")]
    Pulse(SingleColour),
    #[options(help = "set a vertical line zooming from left")]
    Comet(SingleColour),
    #[options(help = "set a wide vertical line zooming from left")]
    Flash(SingleColour),
    #[options(help = "4-zone multi-colour")]
    MultiStatic(MultiColour),
}

impl Default for SetAuraBuiltin {
    fn default() -> Self {
        SetAuraBuiltin::Stable(SingleColour {
            help: false,
            colour: Colour(255, 0, 0),
        })
    }
}
