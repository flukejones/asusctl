use gumdrop::Options;

#[derive(Options)]
pub struct AnimeCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(
        meta = "",
        help = "enable/disable the panel LEDs (does not erase last image)"
    )]
    pub enable: Option<bool>,
    #[options(
        meta = "",
        help = "enable/disable system animations (boot/sleep/shutdown)"
    )]
    pub boot_enable: Option<bool>,
    #[options(meta = "", help = "set global AniMe brightness value")]
    pub brightness: Option<f32>,
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
