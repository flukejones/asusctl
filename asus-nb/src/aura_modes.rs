use crate::cli_options;
use crate::cli_options::SetAuraBuiltin;
use serde_derive::{Deserialize, Serialize};

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
pub const PER_KEY: u8 = 0xff;

#[derive(Clone, Deserialize, Serialize)]
pub struct Colour(pub u8, pub u8, pub u8);
impl From<cli_options::Colour> for Colour {
    fn from(c: cli_options::Colour) -> Self {
        Colour(c.0, c.1, c.2)
    }
}
impl Default for Colour {
    fn default() -> Self {
        Colour(128, 0, 0)
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum Speed {
    Low = 0xe1,
    Med = 0xeb,
    High = 0xf5,
}
impl From<cli_options::Speed> for Speed {
    fn from(s: cli_options::Speed) -> Self {
        match s {
            cli_options::Speed::Low => Speed::Low,
            cli_options::Speed::Med => Speed::Med,
            cli_options::Speed::High => Speed::High,
        }
    }
}
impl Default for Speed {
    fn default() -> Self {
        Speed::Med
    }
}

/// Used for Rainbow mode.
///
/// Enum corresponds to the required integer value
#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}
impl From<cli_options::Direction> for Direction {
    fn from(s: cli_options::Direction) -> Self {
        match s {
            cli_options::Direction::Right => Direction::Right,
            cli_options::Direction::Left => Direction::Left,
            cli_options::Direction::Up => Direction::Up,
            cli_options::Direction::Down => Direction::Down,
        }
    }
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Right
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct TwoColourSpeed {
    pub colour: Colour,
    pub colour2: Colour,
    pub speed: Speed,
}
impl From<cli_options::TwoColourSpeed> for TwoColourSpeed {
    fn from(mode: cli_options::TwoColourSpeed) -> Self {
        TwoColourSpeed {
            colour: mode.colour.into(),
            colour2: mode.colour2.into(),
            speed: mode.speed.into(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SingleSpeed {
    pub speed: Speed,
}
impl From<cli_options::SingleSpeed> for SingleSpeed {
    fn from(mode: cli_options::SingleSpeed) -> Self {
        SingleSpeed {
            speed: mode.speed.into(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SingleColour {
    pub colour: Colour,
}
impl From<cli_options::SingleColour> for SingleColour {
    fn from(mode: cli_options::SingleColour) -> Self {
        SingleColour {
            colour: mode.colour.into(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct MultiColour {
    pub colour1: Colour,
    pub colour2: Colour,
    pub colour3: Colour,
    pub colour4: Colour,
}
impl From<cli_options::MultiColour> for MultiColour {
    fn from(mode: cli_options::MultiColour) -> Self {
        MultiColour {
            colour1: mode.colour1.into(),
            colour2: mode.colour2.into(),
            colour3: mode.colour3.into(),
            colour4: mode.colour4.into(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SingleSpeedDirection {
    pub direction: Direction,
    pub speed: Speed,
}
impl From<cli_options::SingleSpeedDirection> for SingleSpeedDirection {
    fn from(mode: cli_options::SingleSpeedDirection) -> Self {
        SingleSpeedDirection {
            direction: mode.direction.into(),
            speed: mode.speed.into(),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SingleColourSpeed {
    pub colour: Colour,
    pub speed: Speed,
}
impl From<cli_options::SingleColourSpeed> for SingleColourSpeed {
    fn from(mode: cli_options::SingleColourSpeed) -> Self {
        SingleColourSpeed {
            colour: mode.colour.into(),
            speed: mode.speed.into(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
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
    LedBrightness(u8),
    // TODO: use a serializable structure for this (KeyColourArray)
    PerKey(Vec<Vec<u8>>),
}

impl From<SetAuraBuiltin> for AuraModes {
    fn from(mode: SetAuraBuiltin) -> Self {
        match mode {
            SetAuraBuiltin::Static(x) => AuraModes::Static(x.into()),
            SetAuraBuiltin::Breathe(x) => AuraModes::Breathe(x.into()),
            SetAuraBuiltin::Strobe(x) => AuraModes::Strobe(x.into()),
            SetAuraBuiltin::Rainbow(x) => AuraModes::Rainbow(x.into()),
            SetAuraBuiltin::Star(x) => AuraModes::Star(x.into()),
            SetAuraBuiltin::Rain(x) => AuraModes::Rain(x.into()),
            SetAuraBuiltin::Highlight(x) => AuraModes::Highlight(x.into()),
            SetAuraBuiltin::Laser(x) => AuraModes::Laser(x.into()),
            SetAuraBuiltin::Ripple(x) => AuraModes::Ripple(x.into()),
            SetAuraBuiltin::Pulse(x) => AuraModes::Pulse(x.into()),
            SetAuraBuiltin::Comet(x) => AuraModes::Comet(x.into()),
            SetAuraBuiltin::Flash(x) => AuraModes::Flash(x.into()),
            SetAuraBuiltin::MultiStatic(x) => AuraModes::MultiStatic(x.into()),
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
            PER_KEY => AuraModes::PerKey(vec![]),
            _ => panic!("Invalid mode byte"),
        }
    }
}
