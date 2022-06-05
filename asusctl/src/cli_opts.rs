use crate::{
    anime_cli::AnimeCommand,
    aura_cli::{LedBrightness, SetAuraBuiltin},
    profiles_cli::{FanCurveCommand, ProfileCommand},
};
use gumdrop::Options;

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
    #[options(command)]
    pub command: Option<CliCommand>,
}

#[derive(Options)]
pub enum CliCommand {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    LedMode(LedModeCommand),
    #[options(help = "Set or select platform_profile")]
    Profile(ProfileCommand),
    #[options(help = "Set, select, or modify fan curves if supported")]
    FanCurve(FanCurveCommand),
    #[options(help = "Set the graphics mode (obsoleted by supergfxctl)")]
    Graphics(GraphicsCommand),
    #[options(name = "anime", help = "Manage AniMe Matrix")]
    Anime(AnimeCommand),
    #[options(help = "Change bios settings")]
    Bios(BiosCommand),
}

#[derive(Options)]
pub struct LedModeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "switch to next aura mode")]
    pub next_mode: bool,
    #[options(help = "switch to previous aura mode")]
    pub prev_mode: bool,
    #[options(
        meta = "",
        help = "set the keyboard LED to enabled while the device is awake"
    )]
    pub awake_enable: Option<bool>,
    #[options(
        meta = "",
        help = "set the keyboard LED suspend animation to enabled while the device is suspended"
    )]
    pub sleep_enable: Option<bool>,
    #[options(
    meta = "",
    help = "set the keyboard side LEDs to enabled"
    )]
    pub side_leds_enable: Option<bool>,
    #[options(command)]
    pub command: Option<SetAuraBuiltin>,
}

#[derive(Options)]
pub struct GraphicsCommand {
    #[options(help = "print help message")]
    pub help: bool,
}

#[derive(Options, Debug)]
pub struct BiosCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        no_long,
        help = "set bios POST sound: asusctl -p <true/false>"
    )]
    pub post_sound_set: Option<bool>,
    #[options(no_long, help = "read bios POST sound")]
    pub post_sound_get: bool,
    #[options(
        meta = "",
        no_long,
        help = "activate dGPU dedicated/G-Sync: asusctl -d <true/false>, reboot required"
    )]
    pub dedicated_gfx_set: Option<bool>,
    #[options(no_long, help = "get GPU mode")]
    pub dedicated_gfx_get: bool,
}
