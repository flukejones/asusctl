use gumdrop::Options;
use rog_types::{
    aura_modes::{AuraEffect, AuraModeNum, AuraZone, Colour, Direction, Speed},
    error::AuraError,
};
use std::str::FromStr;

#[derive(Options)]
pub struct LedBrightness {
    level: Option<u8>,
}
impl LedBrightness {
    pub fn new(level: Option<u8>) -> Self {
        LedBrightness { level }
    }

    pub fn level(&self) -> Option<u8> {
        self.level
    }
}
impl FromStr for LedBrightness {
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "off" => Ok(LedBrightness { level: Some(0x00) }),
            "low" => Ok(LedBrightness { level: Some(0x01) }),
            "med" => Ok(LedBrightness { level: Some(0x02) }),
            "high" => Ok(LedBrightness { level: Some(0x03) }),
            _ => {
                print!("Invalid argument, must be one of: off, low, med, high");
                Err(AuraError::ParseBrightness)
            }
        }
    }
}
impl ToString for LedBrightness {
    fn to_string(&self) -> String {
        let s = match self.level {
            Some(0x00) => "low",
            Some(0x01) => "med",
            Some(0x02) => "high",
            _ => "unknown",
        };
        s.to_string()
    }
}

#[derive(Debug, Clone, Options, Default)]
pub struct SingleSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "WORD", help = "set the speed: low, med, high")]
    pub speed: Speed,
}
#[derive(Debug, Clone, Options, Default)]
pub struct SingleSpeedDirection {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the direction: up, down, left, right")]
    pub direction: Direction,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Default, Options)]
pub struct SingleColour {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
}

#[derive(Debug, Clone, Default, Options)]
pub struct SingleColourSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Options, Default)]
pub struct TwoColourSpeed {
    #[options(help = "print help message")]
    help: bool,
    #[options(no_long, meta = "", help = "set the first RGB value e.g, ff00ff")]
    pub colour: Colour,
    #[options(no_long, meta = "", help = "set the second RGB value e.g, ff00ff")]
    pub colour2: Colour,
    #[options(no_long, meta = "", help = "set the speed: low, med, high")]
    pub speed: Speed,
}

#[derive(Debug, Clone, Default, Options)]
pub struct MultiColour {
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

#[derive(Debug, Clone, Default, Options)]
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

/// Byte value for setting the built-in mode.
///
/// Enum corresponds to the required integer value
#[derive(Options)]
pub enum SetAuraBuiltin {
    #[options(help = "set a single static colour")]
    Static(SingleColour),
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
    #[options(help = "4-zone multi-colour breathing")]
    MultiBreathe(MultiColourSpeed),
}

impl Default for SetAuraBuiltin {
    fn default() -> Self {
        SetAuraBuiltin::Static(SingleColour::default())
    }
}

impl From<&SingleColour> for AuraEffect {
    fn from(aura: &SingleColour) -> Self {
        Self {
            colour1: aura.colour,
            ..Default::default()
        }
    }
}

impl From<&SingleSpeed> for AuraEffect {
    fn from(aura: &SingleSpeed) -> Self {
        Self {
            speed: aura.speed,
            ..Default::default()
        }
    }
}

impl From<&SingleColourSpeed> for AuraEffect {
    fn from(aura: &SingleColourSpeed) -> Self {
        Self {
            colour1: aura.colour,
            speed: aura.speed,
            ..Default::default()
        }
    }
}

impl From<&TwoColourSpeed> for AuraEffect {
    fn from(aura: &TwoColourSpeed) -> Self {
        Self {
            colour1: aura.colour,
            colour2: aura.colour2,
            ..Default::default()
        }
    }
}

impl From<&SingleSpeedDirection> for AuraEffect {
    fn from(aura: &SingleSpeedDirection) -> Self {
        Self {
            speed: aura.speed,
            direction: aura.direction,
            ..Default::default()
        }
    }
}

impl From<&SetAuraBuiltin> for AuraEffect {
    fn from(aura: &SetAuraBuiltin) -> Self {
        match aura {
            SetAuraBuiltin::Static(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Static;
                data
            }
            SetAuraBuiltin::Breathe(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Breathe;
                data
            }
            SetAuraBuiltin::Strobe(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Strobe;
                data
            }
            SetAuraBuiltin::Rainbow(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Rainbow;
                data
            }
            SetAuraBuiltin::Star(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Star;
                data
            }
            SetAuraBuiltin::Rain(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Rain;
                data
            }
            SetAuraBuiltin::Highlight(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Highlight;
                data
            }
            SetAuraBuiltin::Laser(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Laser;
                data
            }
            SetAuraBuiltin::Ripple(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Ripple;
                data
            }
            SetAuraBuiltin::Pulse(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Pulse;
                data
            }
            SetAuraBuiltin::Comet(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Comet;
                data
            }
            SetAuraBuiltin::Flash(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Flash;
                data
            }
            _ => AuraEffect::default(),
        }
    }
}

impl From<&SetAuraBuiltin> for Vec<AuraEffect> {
    fn from(aura: &SetAuraBuiltin) -> Vec<AuraEffect> {
        let mut zones = vec![AuraEffect::default(); 4];
        match aura {
            SetAuraBuiltin::MultiStatic(data) => {
                zones[0].mode = AuraModeNum::Static;
                zones[0].zone = AuraZone::One;
                zones[0].colour1 = data.colour1;

                zones[1].mode = AuraModeNum::Static;
                zones[1].zone = AuraZone::Two;
                zones[1].colour1 = data.colour2;

                zones[2].mode = AuraModeNum::Static;
                zones[2].zone = AuraZone::Three;
                zones[2].colour1 = data.colour3;

                zones[3].mode = AuraModeNum::Static;
                zones[3].zone = AuraZone::Four;
                zones[3].colour1 = data.colour4;
            }
            SetAuraBuiltin::MultiBreathe(data) => {
                zones[0].mode = AuraModeNum::Breathe;
                zones[0].zone = AuraZone::One;
                zones[0].colour1 = data.colour1;
                zones[0].speed = data.speed;

                zones[1].mode = AuraModeNum::Breathe;
                zones[1].zone = AuraZone::Two;
                zones[1].colour1 = data.colour2;
                zones[1].speed = data.speed;

                zones[2].mode = AuraModeNum::Breathe;
                zones[2].zone = AuraZone::Three;
                zones[2].colour1 = data.colour3;
                zones[2].speed = data.speed;

                zones[3].mode = AuraModeNum::Breathe;
                zones[3].zone = AuraZone::Four;
                zones[3].colour1 = data.colour4;
                zones[3].speed = data.speed;
            }
            _ => {}
        }
        zones
    }
}
