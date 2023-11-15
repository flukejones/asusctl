use gumdrop::Options;
use rog_anime::usb::{AnimAwake, AnimBooting, AnimShutdown, AnimSleeping, Brightness};
use rog_anime::AnimeType;

#[derive(Options)]
pub struct AnimeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "override the display type")]
    pub override_type: Option<AnimeType>,
    #[options(meta = "", help = "enable/disable the display")]
    pub enable_display: Option<bool>,
    #[options(meta = "", help = "enable/disable the builtin run/powersave animation")]
    pub enable_powersave_anim: Option<bool>,
    #[options(
        meta = "",
        help = "set global base brightness value <Off, Low, Med, High>"
    )]
    pub brightness: Option<Brightness>,
    #[options(help = "clear the display")]
    pub clear: bool,
    #[options(command)]
    pub command: Option<AnimeActions>,
}

#[derive(Options)]
pub enum AnimeActions {
    #[options(help = "display a PNG image")]
    Image(AnimeImage),
    #[options(help = "display a diagonal/pixel-perfect PNG")]
    PixelImage(AnimeImageDiagonal),
    #[options(help = "display an animated GIF")]
    Gif(AnimeGif),
    #[options(help = "display an animated diagonal/pixel-perfect GIF")]
    PixelGif(AnimeGifDiagonal),
    #[options(help = "change which builtin animations are shown")]
    SetBuiltins(Builtins),
}

#[derive(Options)]
pub struct Builtins {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        help = "Default is used if unspecified, <default:GlitchConstruction, StaticEmergence>"
    )]
    pub boot: AnimBooting,
    #[options(
        meta = "",
        help = "Default is used if unspecified, <default:BinaryBannerScroll, RogLogoGlitch>"
    )]
    pub awake: AnimAwake,
    #[options(
        meta = "",
        help = "Default is used if unspecified, <default:BannerSwipe, Starfield>"
    )]
    pub sleep: AnimSleeping,
    #[options(
        meta = "",
        help = "Default is used if unspecified, <default:GlitchOut, SeeYa>"
    )]
    pub shutdown: AnimShutdown,
    #[options(meta = "", help = "set/apply the animations <true/false>")]
    pub set: Option<bool>,
}

#[derive(Options)]
pub struct AnimeImage {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "1.0", help = "scale 1.0 == normal")]
    pub scale: f32,
    #[options(meta = "", default = "0.0", help = "x position (float)")]
    pub x_pos: f32,
    #[options(meta = "", default = "0.0", help = "y position (float)")]
    pub y_pos: f32,
    #[options(meta = "", default = "0.0", help = "the angle in radians")]
    pub angle: f32,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
}

#[derive(Options)]
pub struct AnimeImageDiagonal {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
}

#[derive(Options)]
pub struct AnimeGif {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "1.0", help = "scale 1.0 == normal")]
    pub scale: f32,
    #[options(meta = "", default = "0.0", help = "x position (float)")]
    pub x_pos: f32,
    #[options(meta = "", default = "0.0", help = "y position (float)")]
    pub y_pos: f32,
    #[options(meta = "", default = "0.0", help = "the angle in radians")]
    pub angle: f32,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
    #[options(
        meta = "",
        default = "1",
        help = "how many loops to play - 0 is infinite"
    )]
    pub loops: u32,
}

#[derive(Options)]
pub struct AnimeGifDiagonal {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(meta = "", help = "full path to the png to display")]
    pub path: String,
    #[options(meta = "", default = "1.0", help = "brightness 0.0-1.0")]
    pub bright: f32,
    #[options(
        meta = "",
        default = "1",
        help = "how many loops to play - 0 is infinite"
    )]
    pub loops: u32,
}
