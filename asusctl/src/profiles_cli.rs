use gumdrop::Options;

#[derive(Debug, Clone, Options)]
pub struct ProfileCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "toggle to next profile in list")]
    pub next: bool,
    #[options(help = "list available profiles")]
    pub list: bool,
}
