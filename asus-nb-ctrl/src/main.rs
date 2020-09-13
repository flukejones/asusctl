use asus_nb::{
    cli_options::{LedBrightness, SetAuraBuiltin},
    core_dbus::AuraDbusClient,
    profile::{ProfileCommand, ProfileEvent},
};
use ctrl_gfx::vendors::GfxVendors;
use daemon::ctrl_fan_cpu::FanLevel;
use gumdrop::Options;
use log::LevelFilter;
use std::io::Write;
use yansi_term::Colour::Green;
use yansi_term::Colour::Red;

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
    #[options(help = "Set graphics mode: <nvidia, hybrid, compute, integrated>")]
    graphics: Option<GfxVendors>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    if let Some(gfx) = parsed.graphics {
        println!("Updating settings, please wait...");
        println!("If this takes longer than 30s, ctrl+c then check journalctl");

        writer.write_gfx_mode(gfx)?;
        let res = writer.wait_gfx_changed()?;
        match res.as_str() {
            "reboot" => println!(
                "{}\n{}",
                Green.paint("\nGraphics vendor mode changed successfully\n"),
                Red.paint("\nPlease reboot to complete switch to iGPU\n")
            ),
            "restartx" => {
                println!(
                    "{}",
                    Green.paint("\nGraphics vendor mode changed successfully\n")
                );
                restart_x()?;
                std::process::exit(1)
            }
            _ => std::process::exit(-1),
        }
        std::process::exit(-1)
    }
    Ok(())
}

fn restart_x() -> Result<(), Box<dyn std::error::Error>> {
    println!("Restart X server? y/n");

    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).expect("Input failed");
    let input = buf.chars().next().unwrap() as char;

    if input == 'Y' || input == 'y' {
        println!("Restarting X server");
        let status = std::process::Command::new("systemctl")
            .arg("restart")
            .arg("display-manager.service")
            .status()?;

        if !status.success() {
            println!("systemctl: display-manager returned with {}", status);
        }
    } else {
        println!("{}", Red.paint("Cancelled. Please restart X when ready"));
    }
    Ok(())
}
