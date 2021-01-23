use gumdrop::Options;
use rog_fan_curve::{Curve, Fan};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

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

    #[options(help = "enable or disable cpu turbo")]
    pub turbo: Option<bool>,
    #[options(help = "set min cpu scaling (intel)")]
    pub min_percentage: Option<u8>,
    #[options(help = "set max cpu scaling (intel)")]
    pub max_percentage: Option<u8>,

    #[options(meta = "PWR", help = "<silent, normal, boost>")]
    pub preset: Option<FanLevel>,
    #[options(parse(try_from_str = "parse_fan_curve"), help = "set fan curve")]
    pub curve: Option<Curve>,
    #[options(free)]
    pub profile: Option<String>,
}
