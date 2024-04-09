use gumdrop::Options;
use rog_slash::SlashMode;

#[derive(Options)]
pub struct SlashCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "Enable the Slash Ledbar")]
    pub enable: bool,
    #[options(help = "Ddisable the Slash Ledbar")]
    pub disable: bool,
    #[options(meta = "", help = "Set brightness value <0-255>")]
    pub brightness: Option<u8>,
    #[options(meta = "", help = "Set interval value <0-255>")]
    pub interval: Option<u8>,
    #[options(help = "Set SlashMode (so 'list' for all options)")]
    pub slash_mode: Option<SlashMode>,
    #[options(help = "list available animations")]
    pub list: bool,
}
