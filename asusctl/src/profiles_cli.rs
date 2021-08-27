use gumdrop::Options;

#[derive(Debug, Clone, Options)]
pub struct ProfileCommand {
    #[options(help = "print help message")]
    pub help: bool,
    #[options(help = "toggle to next profile in list")]
    pub next: bool,
    #[options(help = "list available profiles")]
    pub list: bool,
    #[options(help = "get active profile name")]
    pub active_name: bool,
    #[options(help = "get active profile data")]
    pub active_data: bool,
    #[options(help = "get all profile data")]
    pub profiles_data: bool,
}
