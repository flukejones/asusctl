use gumdrop::Options;
use rog_fan_curve::{Curve, Fan};
use rog_profiles::profiles::FanLevel;

#[derive(Debug, Clone, Options)]
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

    // Options for profile
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
