use gumdrop::Options;
use rog_profiles::{fan_curve_set::CurveData, FanCurvePU, Profile};

#[derive(Debug, Clone, Options)]
pub struct ProfileCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "toggle to next profile in list")]
    pub next: bool,
    #[options(help = "list available profiles")]
    pub list: bool,

    #[options(help = "get profile")]
    pub profile_get: bool,
    #[options(help = "set the active profile")]
    pub profile_set: Option<Profile>,
}

#[derive(Debug, Clone, Options)]
pub struct FanCurveCommand {
    #[options(help = "print help message")]
    pub help: bool,

    #[options(help = "get enabled fan profiles")]
    pub get_enabled: bool,
    #[options(help = "set the active profile's fan curve to default")]
    pub default: bool,

    #[options(
        meta = "",
        help = "profile to modify fan-curve for. Shows data if no options provided"
    )]
    pub mod_profile: Option<Profile>,
    #[options(
        meta = "",
        help = "enable or disable <true/false> fan curve. `mod-profile` required"
    )]
    pub enabled: Option<bool>,
    #[options(
        meta = "",
        help = "select fan <cpu/gpu> to modify. `mod-profile` required"
    )]
    pub fan: Option<FanCurvePU>,

    #[options(
        meta = "",
        help = "data format = 30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%. `mod-profile` required"
    )]
    pub data: Option<CurveData>,
}
