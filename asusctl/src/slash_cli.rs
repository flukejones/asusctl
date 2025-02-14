use gumdrop::Options;
use rog_slash::SlashMode;

#[derive(Options)]
pub struct SlashCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "Enable the Slash Ledbar")]
    pub enable: bool,
    #[options(help = "Disable the Slash Ledbar")]
    pub disable: bool,
    #[options(short = "l", meta = "", help = "Set brightness value <0-255>")]
    pub brightness: Option<u8>,
    #[options(meta = "", help = "Set interval value <0-5>")]
    pub interval: Option<u8>,
    #[options(meta = "", help = "Set SlashMode (so 'list' for all options)")]
    pub mode: Option<SlashMode>,
    #[options(help = "list available animations")]
    pub list: bool,

    #[options(short = "B", meta = "", help = "Show the animation on boot")]
    pub show_on_boot: Option<bool>,
    #[options(short = "S", meta = "", help = "Show the animation on shutdown")]
    pub show_on_shutdown: Option<bool>,
    #[options(short = "s", meta = "", help = "Show the animation on sleep")]
    pub show_on_sleep: Option<bool>,
    #[options(short = "b", meta = "", help = "Show the animation on battery")]
    pub show_on_battery: Option<bool>,
    #[options(
        short = "w",
        meta = "",
        help = "Show the low-battery warning animation"
    )]
    pub show_battery_warning: Option<bool>,
}
