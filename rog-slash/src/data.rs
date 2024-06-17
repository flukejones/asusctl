use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;
use zbus::zvariant::{OwnedValue, Value};

use crate::error::SlashError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum SlashType {
    GA403,
    GA605,
    GU605,
    Unsupported,
}

impl FromStr for SlashType {
    type Err = SlashError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "ga403" | "GA403" => Self::GA403,
            "ga605" | "GA605" => Self::GA605,
            "gu605" | "GU605" => Self::GU605,
            _ => Self::Unsupported,
        })
    }
}

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub enum SlashMode {
    Bounce = 0x10,
    Slash = 0x12,
    Loading = 0x13,
    BitStream = 0x1d,
    Transmission = 0x1a,
    #[default]
    Flow = 0x19,
    Flux = 0x25,
    Phantom = 0x24,
    Spectrum = 0x26,
    Hazard = 0x32,
    Interfacing = 0x33,
    Ramp = 0x34,
    GameOver = 0x42,
    Start = 0x43,
    Buzzer = 0x44,
}

impl FromStr for SlashMode {
    type Err = SlashError;

    fn from_str(s: &str) -> Result<Self, SlashError> {
        match s {
            "Bounce" => Ok(SlashMode::Bounce),
            "Slash" => Ok(SlashMode::Slash),
            "Loading" => Ok(SlashMode::Loading),
            "BitStream" => Ok(SlashMode::BitStream),
            "Transmission" => Ok(SlashMode::Transmission),
            "Flow" => Ok(SlashMode::Flow),
            "Flux" => Ok(SlashMode::Flux),
            "Phantom" => Ok(SlashMode::Phantom),
            "Spectrum" => Ok(SlashMode::Spectrum),
            "Hazard" => Ok(SlashMode::Hazard),
            "Interfacing" => Ok(SlashMode::Interfacing),
            "Ramp" => Ok(SlashMode::Ramp),
            "GameOver" => Ok(SlashMode::GameOver),
            "Start" => Ok(SlashMode::Start),
            "Buzzer" => Ok(SlashMode::Buzzer),
            _ => Ok(SlashMode::Bounce),
        }
    }
}

impl Display for SlashMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            SlashMode::Bounce => String::from("Bounce"),
            SlashMode::Slash => String::from("Slash"),
            SlashMode::Loading => String::from("Loading"),
            SlashMode::BitStream => String::from("BitStream"),
            SlashMode::Transmission => String::from("Transmission"),
            SlashMode::Flow => String::from("Flow"),
            SlashMode::Flux => String::from("Flux"),
            SlashMode::Phantom => String::from("Phantom"),
            SlashMode::Spectrum => String::from("Spectrum"),
            SlashMode::Hazard => String::from("Hazard"),
            SlashMode::Interfacing => String::from("Interfacing"),
            SlashMode::Ramp => String::from("Ramp"),
            SlashMode::GameOver => String::from("GameOver"),
            SlashMode::Start => String::from("Start"),
            SlashMode::Buzzer => String::from("Buzzer"),
        };
        write!(f, "{}", str)
    }
}

impl SlashMode {
    pub fn list() -> [String; 15] {
        [
            SlashMode::Bounce.to_string(),
            SlashMode::Slash.to_string(),
            SlashMode::Loading.to_string(),
            SlashMode::BitStream.to_string(),
            SlashMode::Transmission.to_string(),
            SlashMode::Flow.to_string(),
            SlashMode::Flux.to_string(),
            SlashMode::Phantom.to_string(),
            SlashMode::Spectrum.to_string(),
            SlashMode::Hazard.to_string(),
            SlashMode::Interfacing.to_string(),
            SlashMode::Ramp.to_string(),
            SlashMode::GameOver.to_string(),
            SlashMode::Start.to_string(),
            SlashMode::Buzzer.to_string(),
        ]
    }
}

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type))]
#[typeshare]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct DeviceState {
    pub slash_enabled: bool,
    pub slash_brightness: u8,
    pub slash_interval: u8,
    pub slash_mode: SlashMode,
}
