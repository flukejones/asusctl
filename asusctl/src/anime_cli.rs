use gumdrop::Options;
use rog_aura::error::Error;
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub enum AnimeStatusValue {
    On,
    Off,
}
impl FromStr for AnimeStatusValue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "on" => Ok(AnimeStatusValue::On),
            "off" => Ok(AnimeStatusValue::Off),
            _ => {
                print!("Invalid argument, must be one of: on, off");
                Err(Error::ParseAnime)
            }
        }
    }
}
impl From<AnimeStatusValue> for bool {
    fn from(value: AnimeStatusValue) -> Self {
        match value {
            AnimeStatusValue::On => true,
            AnimeStatusValue::Off => false,
        }
    }
}

#[derive(Options)]
pub struct AnimeLeds {
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
impl AnimeLeds {
    pub fn led_brightness(&self) -> u8 {
        self.led_brightness
    }
}

#[derive(Options)]
pub struct AnimeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        help = "turn on/off the panel (accept/reject write requests)"
    )]
    pub turn: Option<AnimeStatusValue>,
    #[options(meta = "", help = "turn on/off the panel at boot (with Asus effect)")]
    pub boot: Option<AnimeStatusValue>,
    #[options(command)]
    pub command: Option<AnimeActions>,
}

#[derive(Options)]
pub enum AnimeActions {
    #[options(help = "change all leds brightness")]
    Leds(AnimeLeds),
    #[options(help = "display an image png")]
    Image(AnimeImage),
}

#[derive(Options)]
pub struct AnimeImage {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "1.0", help = "scale 1.0 == normal")]
    pub scale: f32,
    #[options(meta = "", default = "0.0", help = "x position (float)")]
    pub x_pos: f32,
    #[options(meta = "", default = "0.0", help = "y position (float)")]
    pub y_pos: f32,
    #[options(meta = "", default = "0.0", help = "the angle in radians")]
    pub angle: f32,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
}
