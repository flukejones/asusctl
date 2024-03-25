use gumdrop::Options;

#[derive(Options)]
pub struct SlashCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "enable/disable the display")]
    pub enable_display: Option<bool>,
    #[options(meta = "", help = "set brightness value <0-255>")]
    pub brightness: Option<u8>,
    #[options(meta = "", help = "set interval value <0-255>")]
    pub interval: Option<u8>,
    #[options(help = "toggle to next animation in list")]
    pub next: bool,
    #[options(help = "toggle to previous animation in list")]
    pub prev: bool,
    #[options(help = "list available animations")]
    pub list: bool,
}