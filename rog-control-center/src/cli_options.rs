use gumdrop::Options;

#[derive(Default, Options)]
pub struct CliStart {
    #[options(help_flag, help = "print help message")]
    pub help: bool,
    #[options(help = "show program version number")]
    pub version: bool,
    #[options(
        meta = "",
        help = "set board name for testing, this will make ROGCC show only the keyboard page"
    )]
    pub board_name: Option<String>,
    #[options(
        help = "put ROGCC in layout viewing mode - this is helpful for finding existing layouts \
                that might match your laptop"
    )]
    pub layout_viewing: bool,
}
