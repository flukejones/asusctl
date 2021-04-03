use gumdrop::Options;
use rog_fan_curve::{Curve, Fan};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub min_percentage: u8,
    pub max_percentage: u8,
    pub turbo: bool,
    pub fan_preset: u8,
    pub fan_curve: Option<Curve>,
}

#[deprecated]
pub type CpuSettings = Profile;

impl Default for Profile {
    fn default() -> Self {
        Profile {
            min_percentage: 0,
            max_percentage: 100,
            turbo: false,
            fan_preset: 0,
            fan_curve: None,
        }
    }
}

impl Profile {
    pub fn new(
        min_percentage: u8,
        max_percentage: u8,
        turbo: bool,
        fan_preset: u8,
        fan_curve: Option<Curve>,
    ) -> Self {
        Profile {
            min_percentage,
            max_percentage,
            turbo,
            fan_preset,
            fan_curve,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProfileEvent {
    Cli(ProfileCommand),
    ChangeMode(u8),
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FanLevel {
    Normal,
    Boost,
    Silent,
}

impl FromStr for FanLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(FanLevel::Normal),
            "boost" => Ok(FanLevel::Boost),
            "silent" => Ok(FanLevel::Silent),
            _ => Err("Invalid fan level"),
        }
    }
}

impl From<u8> for FanLevel {
    fn from(n: u8) -> Self {
        match n {
            0 => FanLevel::Normal,
            1 => FanLevel::Boost,
            2 => FanLevel::Silent,
            _ => FanLevel::Normal,
        }
    }
}

impl From<FanLevel> for u8 {
    fn from(n: FanLevel) -> Self {
        match n {
            FanLevel::Normal => 0,
            FanLevel::Boost => 1,
            FanLevel::Silent => 2,
        }
    }
}

impl From<&FanLevel> for u8 {
    fn from(n: &FanLevel) -> Self {
        match n {
            FanLevel::Normal => 0,
            FanLevel::Boost => 1,
            FanLevel::Silent => 2,
        }
    }
}

fn parse_fan_curve(data: &str) -> Result<Curve, String> {
    let curve = Curve::from_config_str(data)?;
    if let Err(err) = curve.check_safety(Fan::Cpu) {
        return Err(format!("Unsafe curve {:?}", err));
    }
    if let Err(err) = curve.check_safety(Fan::Gpu) {
        return Err(format!("Unsafe curve {:?}", err));
    }
    Ok(curve)
}

#[derive(Debug, Clone, Options, Serialize, Deserialize)]
pub struct ProfileCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "toggle to next profile in list")]
    pub next: bool,
    #[options(help = "create the profile if it doesn't exist")]
    pub create: bool,
    #[options(meta = "", help = "remove a profile by name")]
    pub remove: Option<String>,
    #[options(help = "list available profiles")]
    pub list: bool,
    #[options(help = "get active profile name")]
    pub active_name: bool,
    #[options(help = "get active profile data")]
    pub active_data: bool,
    #[options(help = "get all profile data")]
    pub profiles_data: bool,

    #[options(meta = "", help = "enable or disable cpu turbo")]
    pub turbo: Option<bool>,
    #[options(meta = "", help = "set min cpu scaling (intel)")]
    pub min_percentage: Option<u8>,
    #[options(meta = "", help = "set max cpu scaling (intel)")]
    pub max_percentage: Option<u8>,

    #[options(meta = "", help = "<silent, normal, boost>")]
    pub fan_preset: Option<FanLevel>,
    #[options(
        meta = "",
        parse(try_from_str = "parse_fan_curve"),
        help = "set fan curve"
    )]
    pub curve: Option<Curve>,
    #[options(free)]
    pub profile: Option<String>,
}
