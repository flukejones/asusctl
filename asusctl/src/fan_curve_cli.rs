use gumdrop::Options;
use rog_platform::platform::ThrottlePolicy;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::FanCurvePU;

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
    pub mod_profile: Option<ThrottlePolicy>,

    #[options(
        meta = "",
        help = "enable or disable <true/false> fan all curves for a profile. `--mod_profile` \
                required"
    )]
    pub enable_fan_curves: Option<bool>,

    #[options(
        meta = "",
        help = "enable or disable <true/false> a single fan curve for a profile. `--mod_profile` \
                and `--fan` required"
    )]
    pub enable_fan_curve: Option<bool>,

    #[options(
        meta = "",
        help = "select fan <cpu/gpu/mid> to modify. `--mod_profile` required"
    )]
    pub fan: Option<FanCurvePU>,

    #[options(
        meta = "",
        help = "data format = 30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%. \
                `--mod-profile` required. If '%' is omitted the fan range is 0-255"
    )]
    pub data: Option<CurveData>
}
