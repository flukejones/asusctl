use asus_nb::{
    anime_dbus::AniMeDbusWriter,
    cli_options::{AniMeActions, AniMeStatusValue, LedBrightness, SetAuraBuiltin},
    core_dbus::AuraDbusClient,
    profile::{ProfileCommand, ProfileEvent},
};
use ctrl_gfx::vendors::GfxVendors;
use daemon::ctrl_fan_cpu::FanLevel;
use gumdrop::{Opt, Options};
use log::LevelFilter;
use std::{env::args, io::Write, process::Command};
use yansi_term::Colour::Green;
use yansi_term::Colour::Red;

#[derive(Default, Options)]
struct CLIStart {
    #[options(help_flag, help = "print help message")]
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
    command: Option<CliCommand>,
}

#[derive(Options)]
enum CliCommand {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    LedMode(LedModeCommand),
    #[options(help = "Create and configure profiles")]
    Profile(ProfileCommand),
    #[options(help = "Set the graphics mode")]
    Graphics(GraphicsCommand),
    #[options(name = "anime", help = "Manage AniMe Matrix")]
    AniMe(AniMeCommand),
}

#[derive(Options)]
struct LedModeCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "switch to next aura mode")]
    next_mode: bool,
    #[options(help = "switch to previous aura mode")]
    prev_mode: bool,
    #[options(command)]
    command: Option<SetAuraBuiltin>,
}

#[derive(Options)]
struct GraphicsCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "Set graphics mode: <nvidia, hybrid, compute, integrated>")]
    mode: Option<GfxVendors>,
    #[options(help = "Get the current mode")]
    get: bool,
    #[options(help = "Get the current power status")]
    pow: bool,
    #[options(help = "Do not ask for confirmation")]
    force: bool,
}

#[derive(Options)]
struct AniMeCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "turn on/off the panel (accept/reject write requests)")]
    turn: Option<AniMeStatusValue>,
    #[options(help = "turn on/off the panel at boot (with Asus effect)")]
    boot: Option<AniMeStatusValue>,
    #[options(command)]
    command: Option<AniMeActions>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    let mut args: Vec<String> = args().collect();
    args.remove(0);

    let parsed: CLIStart;
    let missing_argument_k = gumdrop::Error::missing_argument(Opt::Short('k'));
    match CLIStart::parse_args_default(&args) {
        Ok(p) => {
            parsed = p;
        }
        Err(err) if err.to_string() == missing_argument_k.to_string() => {
            parsed = CLIStart {
                kbd_bright: Some(LedBrightness::new(None)),
                ..Default::default()
            };
        }
        Err(err) => {
            eprintln!("source {}", err);
            std::process::exit(2);
        }
    }

    if parsed.help_requested() {
        // As help option don't work with `parse_args_default`
        // we will call `parse_args_default_or_exit` instead
        CLIStart::parse_args_default_or_exit();
    }

    if parsed.version {
        println!("Version: {}", daemon::VERSION);
    }

    let writer = AuraDbusClient::new()?;
    let anime_writer = AniMeDbusWriter::new()?;

    match parsed.command {
        Some(CliCommand::LedMode(mode)) => {
            if mode.next_mode && mode.prev_mode {
                println!("Please specify either next or previous")
            }
            if mode.next_mode {
                writer.next_keyboard_led_mode()?;
            } else if mode.prev_mode {
                writer.prev_keyboard_led_mode()?;
            } else if let Some(command) = mode.command {
                writer.write_builtin_mode(&command.into())?
            }
        }
        Some(CliCommand::Profile(command)) => {
            if command.next {
                writer.next_fan_profile()?;
            } else {
                writer.write_profile_command(&ProfileEvent::Cli(command))?
            }
        }
        Some(CliCommand::Graphics(command)) => do_gfx(command, &writer)?,
        Some(CliCommand::AniMe(anime)) => {
            if let Some(anime_turn) = anime.turn {
                anime_writer.turn_on_off(anime_turn.into())?
            }
            if let Some(anime_boot) = anime.boot {
                anime_writer.turn_boot_on_off(anime_boot.into())?
            }
            if let Some(action) = anime.command {
                match action {
                    AniMeActions::Leds(anime_leds) => {
                        let led_brightness = anime_leds.led_brightness();
                        anime_writer.set_leds_brightness(led_brightness)?;
                    }
                }
            }
        }
        None => (),
    }

    if let Some(brightness) = parsed.kbd_bright {
        match brightness.level() {
            None => {
                let level = writer.get_led_brightness()?;
                println!("Current keyboard led brightness: {}", level.to_string());
            }
            Some(level) => writer.write_brightness(level)?,
        }
    }

    if let Some(fan_level) = parsed.pwr_profile {
        writer.write_fan_mode(fan_level.into())?;
    }
    if let Some(chg_limit) = parsed.chg_limit {
        writer.write_charge_limit(chg_limit)?;
    }
    Ok(())
}

fn do_gfx(
    command: GraphicsCommand,
    writer: &AuraDbusClient,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(mode) = command.mode {
        println!("Updating settings, please wait...");
        println!("If this takes longer than 30s, ctrl+c then check `journalctl -b -u asusd`");

        writer.write_gfx_mode(mode)?;
        let res = writer.wait_gfx_changed()?;
        match res.as_str() {
            "reboot" => {
                println!(
                    "{}",
                    Green.paint("\nGraphics vendor mode changed successfully\n"),
                );
                do_gfx_action(
                    command.force,
                    Command::new("systemctl").arg("reboot").arg("-i"),
                    "Reboot Linux PC",
                    "Please reboot when ready",
                )?;
            }
            "restartx" => {
                println!(
                    "{}",
                    Green.paint("\nGraphics vendor mode changed successfully\n")
                );
                do_gfx_action(
                    command.force,
                    Command::new("systemctl")
                        .arg("restart")
                        .arg("display-manager.service"),
                    "Restart display-manager server",
                    "Please restart display-manager when ready",
                )?;
                std::process::exit(1)
            }
            _ => std::process::exit(-1),
        }
        std::process::exit(-1)
    }
    if command.get {
        let res = writer.get_gfx_mode()?;
        println!("Current graphics mode: {}", res);
    }
    if command.pow {
        let res = writer.get_gfx_pwr()?;
        if res.contains("active") {
            println!("Current power status: {}", Red.paint(&format!("{}", res)));
        } else {
            println!("Current power status: {}", Green.paint(&format!("{}", res)));
        }
    }
    Ok(())
}

fn do_gfx_action(
    no_confirm: bool,
    command: &mut Command,
    ask_msg: &str,
    cancel_msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}? y/n", ask_msg);

    let mut buf = String::new();
    if no_confirm {
        let status = command.status()?;

        if !status.success() {
            println!("systemctl: returned with {}", status);
        }
    }

    std::io::stdin().read_line(&mut buf).expect("Input failed");
    let input = buf.chars().next().unwrap() as char;

    if input == 'Y' || input == 'y' || no_confirm {
        let status = command.status()?;

        if !status.success() {
            println!("systemctl: returned with {}", status);
        }
    } else {
        println!("{}", Red.paint(&format!("{}", cancel_msg)));
    }
    Ok(())
}
