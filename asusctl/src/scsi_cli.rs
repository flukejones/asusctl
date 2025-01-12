use gumdrop::Options;
use rog_scsi::{AuraMode, Colour, Direction, Speed};

#[derive(Options)]
pub struct ScsiCommand {
    #[options(help = "print help message")]
    pub help: bool,

    #[options(help = "Enable the SCSI drive LEDs")]
    pub enable: Option<bool>,

    #[options(meta = "", help = "Set LED mode (so 'list' for all options)")]
    pub mode: Option<AuraMode>,

    #[options(
        meta = "",
        help = "Set LED mode speed <slowest, slow, med, fast, fastest> (does not apply to all)"
    )]
    pub speed: Option<Speed>,

    #[options(
        meta = "",
        help = "Set LED mode direction <forward, reverse> (does not apply to all)"
    )]
    pub direction: Option<Direction>,

    #[options(
        meta = "",
        help = "Set LED colours <hex>, specify up to 4 with repeated arg"
    )]
    pub colours: Vec<Colour>,

    #[options(help = "list available animations")]
    pub list: bool
}
