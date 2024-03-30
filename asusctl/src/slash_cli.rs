use gumdrop::Options;
use rog_slash::SlashMode;

#[derive(Options)]
pub struct SlashCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "enable/disable the display")]
    pub enabled: Option<bool>,
    #[options(meta = "", help = "set brightness value <0-255>")]
    pub brightness: Option<u8>,
    #[options(meta = "", help = "set interval value <0-255>")]
    pub interval: Option<u8>,
    #[options(help = "Set the SlashMode")]
    pub slash_mode: Option<SlashMode>,
    #[options(help = "list available animations")]
    pub list: bool,
}