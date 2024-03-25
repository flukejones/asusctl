use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};
use typeshare::typeshare;
#[cfg(feature = "dbus")]
use zbus::zvariant::Type;

use crate::error::SlashError;

#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type), zvariant(signature = "s"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum SlashType {
    GA403,
    Unknown,
}

impl FromStr for SlashType {
    type Err = SlashError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "ga403" | "GA403" => Self::GA403,
            _ => Self::Unknown,
        })
    }
}

#[typeshare]
// #[cfg_attr(feature = "dbus", derive(Type, Value, OwnedValue))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SlashMode {
    Bounce = 0x10,
    Slash = 0x12,
    Loading = 0x13,
    BitStream = 0x1D,
    Transmission = 0x1A,
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

impl SlashMode {
    pub const fn to_string(&self) -> &str
    {
        match &self {
            SlashMode::Bounce => "Bounce",
            SlashMode::Slash => "Slash",
            SlashMode::Loading => "Loading",
            SlashMode::BitStream => "BitStream",
            SlashMode::Transmission => "Transmission",
            SlashMode::Flow => "Flow",
            SlashMode::Flux => "Flux",
            SlashMode::Phantom => "Phantom",
            SlashMode::Spectrum => "Spectrum",
            SlashMode::Hazard => "Hazard",
            SlashMode::Interfacing => "Interfacing",
            SlashMode::Ramp => "Ramp",
            SlashMode::GameOver => "GameOver",
            SlashMode::Start => "Start",
            SlashMode::Buzzer => "Buzzer",
        }
    }

    pub const fn list() -> [&'static str; 15] {
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



// TODO: move this out
#[typeshare]
#[cfg_attr(feature = "dbus", derive(Type))]
#[typeshare]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct DeviceState {
    pub slash_enabled: bool,
    pub slash_brightness: u8,
    pub slash_interval: u8,
    pub slash_mode: u8,
}

