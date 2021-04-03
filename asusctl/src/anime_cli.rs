use gumdrop::Options;
use rog_types::error::AuraError;
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub enum AniMeStatusValue {
    On,
    Off,
}
impl FromStr for AniMeStatusValue {
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "on" => Ok(AniMeStatusValue::On),
            "off" => Ok(AniMeStatusValue::Off),
            _ => {
                print!("Invalid argument, must be one of: on, off");
                Err(AuraError::ParseAnime)
            }
        }
    }
}
impl From<AniMeStatusValue> for bool {
    fn from(value: AniMeStatusValue) -> Self {
        match value {
            AniMeStatusValue::On => true,
            AniMeStatusValue::Off => false,
        }
    }
}

#[derive(Options)]
pub struct AniMeLeds {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        no_long,
        required,
        short = "b",
        meta = "",
        help = "set all leds brightness value"
    )]
    led_brightness: u8,
}
impl AniMeLeds {
    pub fn led_brightness(&self) -> u8 {
        self.led_brightness
    }
}

#[derive(Options)]
pub struct AniMeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        help = "turn on/off the panel (accept/reject write requests)"
    )]
    pub turn: Option<AniMeStatusValue>,
    #[options(meta = "", help = "turn on/off the panel at boot (with Asus effect)")]
    pub boot: Option<AniMeStatusValue>,
    #[options(command)]
    pub command: Option<AniMeActions>,
}

#[derive(Options)]
pub enum AniMeActions {
    #[options(help = "change all leds brightness")]
    Leds(AniMeLeds),
    #[options(help = "display an 8bit greyscale png")]
    Image(AniMeImage),
}

#[derive(Options)]
pub struct AniMeImage {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "0.0", help = "x scale 0.0-1.0")]
    pub x_scale: f32,
    #[options(meta = "", default = "0.0", help = "y scale 0.0-1.0")]
    pub y_scale: f32,
    #[options(meta = "", default = "0.0", help = "x position (float)")]
    pub x_pos: f32,
    #[options(meta = "", default = "0.0", help = "y position (float)")]
    pub y_pos: f32,
    #[options(meta = "", default = "0.0", help = "the angle in radians")]
    pub angle: f32,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
}
