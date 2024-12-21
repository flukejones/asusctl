use gumdrop::Options;
use rog_platform::platform::ThrottlePolicy;

use crate::anime_cli::AnimeCommand;
use crate::aura_cli::{LedBrightness, LedPowerCommand1, LedPowerCommand2, SetAuraBuiltin};
use crate::fan_curve_cli::FanCurveCommand;
use crate::scsi_cli::ScsiCommand;
use crate::slash_cli::SlashCommand;

#[derive(Default, Options)]
pub struct CliStart {
    #[options(help_flag, help = "print help message")]
    pub help: bool,
    #[options(help = "show program version number")]
    pub version: bool,
    #[options(help = "show supported functions of this laptop")]
    pub show_supported: bool,
    #[options(meta = "", help = "<off, low, med, high>")]
    pub kbd_bright: Option<LedBrightness>,
    #[options(help = "Toggle to next keyboard brightness")]
    pub next_kbd_bright: bool,
    #[options(help = "Toggle to previous keyboard brightness")]
    pub prev_kbd_bright: bool,
    #[options(meta = "", help = "Set your battery charge limit <20-100>")]
    pub chg_limit: Option<u8>,
    #[options(help = "Toggle one-shot battery charge to 100%")]
    pub one_shot_chg: bool,
    #[options(command)]
    pub command: Option<CliCommand>,
}

#[derive(Options)]
pub enum CliCommand {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    Aura(LedModeCommand),
    #[options(help = "Set the LED power states")]
    AuraPowerOld(LedPowerCommand1),
    #[options(help = "Set the LED power states")]
    AuraPower(LedPowerCommand2),
    #[options(help = "Set or select platform_profile")]
    Profile(ProfileCommand),
    #[options(help = "Set, select, or modify fan curves if supported")]
    FanCurve(FanCurveCommand),
    #[options(help = "Set the graphics mode (obsoleted by supergfxctl)")]
    Graphics(GraphicsCommand),
    #[options(name = "anime", help = "Manage AniMe Matrix")]
    Anime(AnimeCommand),
    #[options(name = "slash", help = "Manage Slash Ledbar")]
    Slash(SlashCommand),
    #[options(name = "scsi", help = "Manage SCSI external drive")]
    Scsi(ScsiCommand),
    #[options(help = "Change bios settings")]
    Platform(PlatformCommand),
}

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

    #[options(meta = "", help = "set the active profile")]
    pub profile_set: Option<ThrottlePolicy>,
}

#[derive(Options)]
pub struct LedModeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "switch to next aura mode")]
    pub next_mode: bool,
    #[options(help = "switch to previous aura mode")]
    pub prev_mode: bool,
    #[options(command)]
    pub command: Option<SetAuraBuiltin>,
}

#[derive(Options)]
pub struct GraphicsCommand {
    #[options(help = "print help message")]
    pub help: bool,
}

#[derive(Options, Debug)]
pub struct PlatformCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        short = "S",
        no_long,
        help = "set bios POST sound: asusctl -S <true/false>"
    )]
    pub post_sound_set: Option<bool>,
    #[options(no_long, short = "s", help = "read bios POST sound")]
    pub post_sound_get: bool,
    #[options(
        meta = "",
        short = "D",
        no_long,
        help = "Switch GPU MUX mode: 0 = Discrete, 1 = Optimus, reboot required"
    )]
    pub gpu_mux_mode_set: Option<u8>,
    #[options(no_long, short = "d", help = "get GPU mode")]
    pub gpu_mux_mode_get: bool,
    #[options(
        meta = "",
        short = "O",
        no_long,
        help = "Set device panel overdrive <true/false>"
    )]
    pub panel_overdrive_set: Option<bool>,
    #[options(no_long, short = "o", help = "get panel overdrive")]
    pub panel_overdrive_get: bool,
}
