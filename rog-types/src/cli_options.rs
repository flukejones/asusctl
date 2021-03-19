use crate::{
    error::AuraError,
};
use gumdrop::Options;
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
pub enum AniMeActions {
    #[options(help = "change all leds brightness")]
    Leds(AniMeLeds),
}
