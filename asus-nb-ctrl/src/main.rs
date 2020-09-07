use asus_nb::{
    cli_options::{LedBrightness, SetAuraBuiltin},
    core_dbus::AuraDbusClient,
    profile::{ProfileCommand, ProfileEvent},
};
use daemon::ctrl_fan_cpu::FanLevel;
use gumdrop::Options;
use log::LevelFilter;
use std::io::Write;

#[derive(Options)]
struct CLIStart {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "show program version number")]
    version: bool,
    #[options(meta = "VAL", help = "<off, low, med, high>")]
    kbd_bright: Option<LedBrightness>,
    #[options(meta = "PWR", help = "<silent, normal, boost>")]
    pwr_profile: Option<FanLevel>,
    #[options(meta = "CHRG", help = "<20-100>")]
    chg_limit: Option<u8>,
    #[options(command)]
    command: Option<Command>,
}

#[derive(Options)]
enum Command {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    LedMode(LedModeCommand),
    #[options(help = "Create and configure profiles")]
    Profile(ProfileCommand),
}

#[derive(Options)]
struct LedModeCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(command, required)]
    command: Option<SetAuraBuiltin>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    let parsed = CLIStart::parse_args_default_or_exit();

    if parsed.version {
        println!("Version: {}", daemon::VERSION);
    }

    let writer = AuraDbusClient::new()?;

    match parsed.command {
        Some(Command::LedMode(mode)) => {
            if let Some(command) = mode.command {
                writer.write_builtin_mode(&command.into())?
            }
        }
        Some(Command::Profile(command)) => {
            writer.write_profile_command(&ProfileEvent::Cli(command))?
        }
        None => (),
    }

    if let Some(brightness) = parsed.kbd_bright {
        writer.write_brightness(brightness.level())?;
    }
    if let Some(fan_level) = parsed.pwr_profile {
        writer.write_fan_mode(fan_level.into())?;
    }
    if let Some(chg_limit) = parsed.chg_limit {
        writer.write_charge_limit(chg_limit)?;
    }
    Ok(())
}
