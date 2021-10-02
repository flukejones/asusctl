mod anime_cli;
mod aura_cli;
mod cli_opts;
mod profiles_cli;

use crate::aura_cli::{LedBrightness, SetAuraBuiltin};
use crate::cli_opts::*;
use anime_cli::{AnimeActions, AnimeCommand};
use gumdrop::{Opt, Options};
use profiles_cli::{FanCurveCommand, ProfileCommand};
use rog_anime::{AnimeDataBuffer, AnimeImage, Vec2, ANIME_DATA_LEN};
use rog_aura::{self, AuraEffect};
use rog_dbus::RogDbusClient;
use rog_profiles::error::ProfileError;
use rog_supported::SupportedFunctions;
use rog_supported::{
    AnimeSupportedFunctions, LedSupportedFunctions, PlatformProfileFunctions,
    RogBiosSupportedFunctions,
};
use std::process::Command;
use std::{env::args, path::Path};

const CONFIG_ADVICE: &str = "A config file need to be removed so a new one can be generated";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().skip(1).collect();

    let parsed: CliStart;
    let missing_argument_k = gumdrop::Error::missing_argument(Opt::Short('k'));
    match CliStart::parse_args_default(&args) {
        Ok(p) => {
            parsed = p;
        }
        Err(err) if err.to_string() == missing_argument_k.to_string() => {
            parsed = CliStart {
                kbd_bright: Some(LedBrightness::new(None)),
                ..Default::default()
            };
        }
        Err(err) => {
            eprintln!("source {}", err);
            std::process::exit(2);
        }
    }

    let (dbus, _) = RogDbusClient::new()
        .map_err(|e| {
            print_error_help(Box::new(e), None);
            std::process::exit(3);
        })
        .unwrap();

    let supported = dbus
        .proxies()
        .supported()
        .get_supported_functions()
        .map_err(|e| {
            print_error_help(Box::new(e), None);
            std::process::exit(4);
        })
        .unwrap();

    if parsed.version {
        print_versions();
        println!();
        print_laptop_info();
        return Ok(());
    }

    if let Err(err) = do_parsed(&parsed, &supported, &dbus) {
        print_error_help(err, Some(&supported));
    }

    Ok(())
}

fn print_error_help(err: Box<dyn std::error::Error>, supported: Option<&SupportedFunctions>) {
    if do_diagnose("asusd") {
        println!("\nError: {}\n", err);
        print_versions();
        println!();
        print_laptop_info();
        if let Some(supported) = supported {
            println!();
            println!("Supported laptop functions:\n\n{}", supported);
        }
    }
}

fn print_versions() {
    println!("App and daemon versions:");
    println!("      asusctl v{}", env!("CARGO_PKG_VERSION"));
    println!("        asusd v{}", daemon::VERSION);
    println!("\nComponent crate versions:");
    println!("    rog-anime v{}", rog_anime::VERSION);
    println!("     rog-aura v{}", rog_aura::VERSION);
    println!("     rog-dbus v{}", rog_dbus::VERSION);
    println!(" rog-profiles v{}", rog_profiles::VERSION);
    println!("rog-supported v{}", rog_supported::VERSION);
}

fn print_laptop_info() {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");

    println!("Product family: {}", prod_family.trim());
    println!("Board name: {}", board_name.trim());
}

fn do_diagnose(name: &str) -> bool {
    if name != "asusd" && !check_systemd_unit_enabled(name) {
        println!(
            "\n\x1b[0;31m{} is not enabled, enable it with `systemctl enable {}\x1b[0m",
            name, name
        );
        return true;
    } else if !check_systemd_unit_active(name) {
        println!(
            "\n\x1b[0;31m{} is not running, start it with `systemctl start {}\x1b[0m",
            name, name
        );
        return true;
    } else {
        println!("\nSome error happened (sorry)");
        println!(
            "Please use `systemctl status {}` and `journalctl -b -u {}` for more information",
            name, name
        );
        println!("{}", CONFIG_ADVICE);
    }
    false
}

fn do_parsed(
    parsed: &CliStart,
    supported: &SupportedFunctions,
    dbus: &RogDbusClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(dbus, &supported.keyboard_led, mode)?,
        Some(CliCommand::Profile(cmd)) => handle_profile(dbus, &supported.platform_profile, cmd)?,
        Some(CliCommand::FanCurve(cmd)) => {
            handle_fan_curve(dbus, &supported.platform_profile, cmd)?
        }
        Some(CliCommand::Graphics(_)) => do_gfx()?,
        Some(CliCommand::Anime(cmd)) => handle_anime(dbus, &supported.anime_ctrl, cmd)?,
        Some(CliCommand::Bios(cmd)) => handle_bios_option(dbus, &supported.rog_bios_ctrl, cmd)?,
        None => {
            if (!parsed.show_supported
                && parsed.kbd_bright.is_none()
                && parsed.chg_limit.is_none()
                && !parsed.next_kbd_bright
                && !parsed.prev_kbd_bright)
                || parsed.help
            {
                println!("{}", CliStart::usage());
                println!();
                if let Some(cmdlist) = CliStart::command_list() {
                    println!("{}", cmdlist);
                }
            }
        }
    }

    if let Some(brightness) = &parsed.kbd_bright {
        match brightness.level() {
            None => {
                let level = dbus.proxies().led().get_led_brightness()?;
                println!("Current keyboard led brightness: {}", level.to_string());
            }
            Some(level) => dbus
                .proxies()
                .led()
                .set_led_brightness(<rog_aura::LedBrightness>::from(level))?,
        }
    }

    if parsed.next_kbd_bright {
        dbus.proxies().led().next_led_brightness()?;
    }

    if parsed.prev_kbd_bright {
        dbus.proxies().led().prev_led_brightness()?;
    }

    if parsed.show_supported {
        println!("Supported laptop functions:\n\n{}", supported);
    }

    if let Some(chg_limit) = parsed.chg_limit {
        dbus.proxies().charge().write_limit(chg_limit)?;
    }

    Ok(())
}

fn do_gfx() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please use supergfxctl for graphics switching. supergfxctl is the result of making asusctl graphics switching generic so all laptops can use it");
    println!("This command will be removed in future");
    Ok(())
}

fn handle_anime(
    dbus: &RogDbusClient,
    _supported: &AnimeSupportedFunctions,
    cmd: &AnimeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if (cmd.command.is_none() && cmd.boot.is_none() && cmd.turn.is_none()) || cmd.help {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }
    if let Some(anime_turn) = cmd.turn {
        dbus.proxies().anime().set_led_power(anime_turn.into())?
    }
    if let Some(anime_boot) = cmd.boot {
        dbus.proxies()
            .anime()
            .set_system_animations(anime_boot.into())?
    }
    if let Some(action) = cmd.command.as_ref() {
        match action {
            AnimeActions::Leds(anime_leds) => {
                let data = AnimeDataBuffer::from_vec(
                    [anime_leds.led_brightness(); ANIME_DATA_LEN].to_vec(),
                );
                dbus.proxies().anime().write(data)?;
            }
            AnimeActions::Image(image) => {
                if image.help_requested() || image.path.is_empty() {
                    println!("Missing arg or command\n\n{}", image.self_usage());
                    if let Some(lst) = image.self_command_list() {
                        println!("\n{}", lst);
                    }
                    std::process::exit(1);
                }

                let matrix = AnimeImage::from_png(
                    Path::new(&image.path),
                    image.scale,
                    image.angle,
                    Vec2::new(image.x_pos, image.y_pos),
                    image.bright,
                )?;

                dbus.proxies()
                    .anime()
                    .write(<AnimeDataBuffer>::from(&matrix))?;
            }
        }
    }
    Ok(())
}

fn handle_led_mode(
    dbus: &RogDbusClient,
    supported: &LedSupportedFunctions,
    mode: &LedModeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if mode.command.is_none()
        && !mode.prev_mode
        && !mode.next_mode
        && mode.sleep_enable.is_none()
        && mode.awake_enable.is_none()
    {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedModeCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_string()).collect();
            for command in commands.iter().filter(|command| {
                for mode in &supported.stock_led_modes {
                    if command.contains(&<&str>::from(mode).to_lowercase()) {
                        return true;
                    }
                }
                if supported.multizone_led_mode {
                    return true;
                }
                false
            }) {
                println!("{}", command);
            }
        }

        println!("\nHelp can also be requested on modes, e.g: static --help");
        return Ok(());
    }

    if mode.next_mode && mode.prev_mode {
        println!("Please specify either next or previous");
        return Ok(());
    }
    if mode.next_mode {
        dbus.proxies().led().next_led_mode()?;
    } else if mode.prev_mode {
        dbus.proxies().led().prev_led_mode()?;
    } else if let Some(mode) = mode.command.as_ref() {
        if mode.help_requested() {
            println!("{}", mode.self_usage());
            return Ok(());
        }
        match mode {
            SetAuraBuiltin::MultiStatic(_) | SetAuraBuiltin::MultiBreathe(_) => {
                let zones = <Vec<AuraEffect>>::from(mode);
                for eff in zones {
                    dbus.proxies().led().set_led_mode(&eff)?
                }
            }
            _ => dbus
                .proxies()
                .led()
                .set_led_mode(&<AuraEffect>::from(mode))?,
        }
    }

    if let Some(enable) = mode.awake_enable {
        dbus.proxies().led().set_awake_enabled(enable)?;
    }

    if let Some(enable) = mode.sleep_enable {
        dbus.proxies().led().set_sleep_enabled(enable)?;
    }

    Ok(())
}

fn handle_profile(
    dbus: &RogDbusClient,
    supported: &PlatformProfileFunctions,
    cmd: &ProfileCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.fan_curves {
        println!("Profiles not supported by either this kernel or by the laptop.");
        return Err(ProfileError::NotSupported.into());
    }

    println!("Warning: Profiles now depend on power-profiles-daemon v0.9+ which may not be in your install");
    println!("         If you have unexpected behaviour or have only two profiles in your desktop control");
    println!("         you need to manually install from https://gitlab.freedesktop.org/hadess/power-profiles-daemon");
    println!("         Fedora and Arch distros will get the update soon...\n");
    if !cmd.next && !cmd.list && cmd.profile_set.is_none() && !cmd.profile_get {
        if !cmd.help {
            println!("Missing arg or command\n");
        }
        println!("{}", ProfileCommand::usage());

        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
        std::process::exit(1);
    }

    if cmd.next {
        dbus.proxies().profile().next_profile()?;
    } else if let Some(profile) = cmd.profile_set {
        dbus.proxies().profile().set_active_profile(profile)?;
    }

    if cmd.list {
        let res = dbus.proxies().profile().profiles()?;
        res.iter().for_each(|p| println!("{:?}", p));
    }

    if cmd.profile_get {
        let res = dbus.proxies().profile().active_profile()?;
        println!("Active profile is {:?}", res);
    }

    Ok(())
}

fn handle_fan_curve(
    dbus: &RogDbusClient,
    supported: &PlatformProfileFunctions,
    cmd: &FanCurveCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.fan_curves {
        println!("Fan-curves not supported by either this kernel or by the laptop.");
        return Err(ProfileError::NotSupported.into());
    }

    if !cmd.get_enabled && !cmd.default && cmd.mod_profile.is_none() {
        if !cmd.help {
            println!("Missing arg or command\n");
        }
        println!("{}", FanCurveCommand::usage());

        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
        std::process::exit(1);
    }

    if (cmd.enabled.is_some() || cmd.fan.is_some() || cmd.data.is_some())
        && cmd.mod_profile.is_none()
    {
        println!("--enabled, --fan, and --data options require --mod-profile");
        std::process::exit(666);
    }

    if cmd.get_enabled {
        let res = dbus.proxies().profile().enabled_fan_profiles()?;
        println!("{:?}", res);
    }

    if cmd.default {
        dbus.proxies().profile().set_active_curve_to_defaults()?;
    }

    if let Some(profile) = cmd.mod_profile {
        if cmd.enabled.is_none() && cmd.data.is_none() {
            let data = dbus.proxies().profile().fan_curve_data(profile)?;
            let data = toml::to_string(&data)?;
            println!("\nFan curves for {:?}\n\n{}", profile, data);
        }

        if let Some(enabled) = cmd.enabled {
            dbus.proxies()
                .profile()
                .set_fan_curve_enabled(profile, enabled)?;
        }

        if let Some(mut curve) = cmd.data.clone() {
            let fan = cmd.fan.unwrap_or_default();
            curve.set_fan(fan);
            dbus.proxies().profile().set_fan_curve(curve, profile)?;
        }
    }

    Ok(())
}

fn handle_bios_option(
    dbus: &RogDbusClient,
    supported: &RogBiosSupportedFunctions,
    cmd: &BiosCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        if (cmd.dedicated_gfx_set.is_none()
            && !cmd.dedicated_gfx_get
            && cmd.post_sound_set.is_none()
            && !cmd.post_sound_get)
            || cmd.help
        {
            println!("Missing arg or command\n");

            let usage: Vec<String> = BiosCommand::usage()
                .lines()
                .map(|s| s.to_string())
                .collect();

            for line in usage.iter().filter(|line| {
                !line.contains("sound") && !supported.post_sound_toggle
                    || !line.contains("GPU") && !supported.dedicated_gfx_toggle
            }) {
                println!("{}", line);
            }
        }

        if let Some(opt) = cmd.post_sound_set {
            dbus.proxies().rog_bios().set_post_sound(opt)?;
        }
        if cmd.post_sound_get {
            let res = dbus.proxies().rog_bios().get_post_sound()? == 1;
            println!("Bios POST sound on: {}", res);
        }
        if let Some(opt) = cmd.dedicated_gfx_set {
            println!("Rebuilding initrd to include drivers");
            dbus.proxies().rog_bios().set_dedicated_gfx(opt)?;
            println!("The mode change is not active until you reboot, on boot the bios will make the required change");
            if opt {
                println!(
                    "NOTE: on reboot your display manager will be forced to use Nvidia drivers"
                );
            } else {
                println!("NOTE: after reboot you can then select regular graphics modes");
            }
        }
        if cmd.dedicated_gfx_get {
            let res = dbus.proxies().rog_bios().get_dedicated_gfx()? == 1;
            println!("Bios dedicated GPU on: {}", res);
        }
    }
    Ok(())
}

fn check_systemd_unit_active(name: &str) -> bool {
    if let Ok(out) = Command::new("systemctl")
        .arg("is-active")
        .arg(name)
        .output()
    {
        let buf = String::from_utf8_lossy(&out.stdout);
        return !buf.contains("inactive") && !buf.contains("failed");
    }
    false
}

fn check_systemd_unit_enabled(name: &str) -> bool {
    if let Ok(out) = Command::new("systemctl")
        .arg("is-enabled")
        .arg(name)
        .output()
    {
        let buf = String::from_utf8_lossy(&out.stdout);
        return buf.contains("enabled");
    }
    false
}
