use std::process::Command;
use std::thread::sleep;
use std::{env::args, path::Path};

use aura_cli::LedPowerCommand;
use gumdrop::{Opt, Options};

use anime_cli::{AnimeActions, AnimeCommand};
use profiles_cli::{FanCurveCommand, ProfileCommand};
use rog_anime::usb::get_anime_type;
use rog_anime::{AnimTime, AnimeDataBuffer, AnimeDiagonal, AnimeGif, AnimeImage, Vec2};
use rog_aura::usb::AuraControl;
use rog_aura::{self, AuraEffect};
use rog_dbus::RogDbusClientBlocking;
use rog_profiles::error::ProfileError;
use rog_supported::SupportedFunctions;
use rog_supported::{
    AnimeSupportedFunctions, LedSupportedFunctions, PlatformProfileFunctions,
    RogBiosSupportedFunctions,
};

use crate::aura_cli::LedBrightness;
use crate::cli_opts::*;

mod anime_cli;
mod aura_cli;
mod cli_opts;
mod profiles_cli;

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

    let (dbus, _) = RogDbusClientBlocking::new()
        .map_err(|e| {
            print_error_help(Box::new(e), None);
            std::process::exit(3);
        })
        .unwrap();

    let supported = dbus
        .proxies()
        .supported()
        .supported_functions()
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
    dbus: &RogDbusClientBlocking,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(dbus, &supported.keyboard_led, mode)?,
        Some(CliCommand::LedPower(pow)) => handle_led_power(dbus, &supported.keyboard_led, pow)?,
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
                let level = dbus.proxies().led().led_brightness()?;
                println!("Current keyboard led brightness: {}", level);
            }
            Some(level) => dbus
                .proxies()
                .led()
                .set_brightness(<rog_aura::LedBrightness>::from(level))?,
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
        dbus.proxies().charge().set_limit(chg_limit)?;
    }

    Ok(())
}

fn do_gfx() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please use supergfxctl for graphics switching. supergfxctl is the result of making asusctl graphics switching generic so all laptops can use it");
    println!("This command will be removed in future");
    Ok(())
}

fn handle_anime(
    dbus: &RogDbusClientBlocking,
    _supported: &AnimeSupportedFunctions,
    cmd: &AnimeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if (cmd.command.is_none()
        && cmd.enable.is_none()
        && cmd.boot_enable.is_none()
        && cmd.brightness.is_none())
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }
    if let Some(anime_turn) = cmd.enable {
        dbus.proxies().anime().set_on_off(anime_turn)?
    }
    if let Some(anime_boot) = cmd.boot_enable {
        dbus.proxies().anime().set_boot_on_off(anime_boot)?
    }
    if let Some(bright) = cmd.brightness {
        verify_brightness(bright);
        dbus.proxies().anime().set_brightness(bright)?
    }
    if let Some(action) = cmd.command.as_ref() {
        let anime_type = get_anime_type()?;
        match action {
            AnimeActions::Image(image) => {
                if image.help_requested() || image.path.is_empty() {
                    println!("Missing arg or command\n\n{}", image.self_usage());
                    if let Some(lst) = image.self_command_list() {
                        println!("\n{}", lst);
                    }
                    std::process::exit(1);
                }
                verify_brightness(image.bright);

                let matrix = AnimeImage::from_png(
                    Path::new(&image.path),
                    image.scale,
                    image.angle,
                    Vec2::new(image.x_pos, image.y_pos),
                    image.bright,
                    anime_type,
                )?;

                dbus.proxies()
                    .anime()
                    .write(<AnimeDataBuffer>::from(&matrix))?;
            }
            AnimeActions::PixelImage(image) => {
                if image.help_requested() || image.path.is_empty() {
                    println!("Missing arg or command\n\n{}", image.self_usage());
                    if let Some(lst) = image.self_command_list() {
                        println!("\n{}", lst);
                    }
                    std::process::exit(1);
                }
                verify_brightness(image.bright);

                let matrix = AnimeDiagonal::from_png(
                    Path::new(&image.path),
                    None,
                    image.bright,
                    anime_type,
                )?;

                dbus.proxies()
                    .anime()
                    .write(matrix.into_data_buffer(anime_type))?;
            }
            AnimeActions::Gif(gif) => {
                if gif.help_requested() || gif.path.is_empty() {
                    println!("Missing arg or command\n\n{}", gif.self_usage());
                    if let Some(lst) = gif.self_command_list() {
                        println!("\n{}", lst);
                    }
                    std::process::exit(1);
                }
                verify_brightness(gif.bright);

                let matrix = AnimeGif::from_gif(
                    Path::new(&gif.path),
                    gif.scale,
                    gif.angle,
                    Vec2::new(gif.x_pos, gif.y_pos),
                    AnimTime::Count(1),
                    gif.bright,
                    anime_type,
                )?;

                let mut loops = gif.loops as i32;
                loop {
                    for frame in matrix.frames() {
                        dbus.proxies().anime().write(frame.frame().clone())?;
                        sleep(frame.delay());
                    }
                    if loops >= 0 {
                        loops -= 1;
                    }
                    if loops == 0 {
                        break;
                    }
                }
            }
            AnimeActions::PixelGif(gif) => {
                if gif.help_requested() || gif.path.is_empty() {
                    println!("Missing arg or command\n\n{}", gif.self_usage());
                    if let Some(lst) = gif.self_command_list() {
                        println!("\n{}", lst);
                    }
                    std::process::exit(1);
                }
                verify_brightness(gif.bright);

                let matrix = AnimeGif::from_diagonal_gif(
                    Path::new(&gif.path),
                    AnimTime::Count(1),
                    gif.bright,
                    anime_type,
                )?;

                let mut loops = gif.loops as i32;
                loop {
                    for frame in matrix.frames() {
                        dbus.proxies().anime().write(frame.frame().clone())?;
                        sleep(frame.delay());
                    }
                    if loops >= 0 {
                        loops -= 1;
                    }
                    if loops == 0 {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

fn verify_brightness(brightness: f32) {
    if brightness < 0.0 || brightness > 1.0 {
        println!(
            "Image and global brightness must be between 0.0 and 1.0 (inclusive), was {}",
            brightness
        );
        std::process::exit(1);
    }
}

fn handle_led_mode(
    dbus: &RogDbusClientBlocking,
    supported: &LedSupportedFunctions,
    mode: &LedModeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if mode.command.is_none() && !mode.prev_mode && !mode.next_mode {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedModeCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_string()).collect();
            for command in commands.iter().filter(|command| {
                for mode in &supported.stock_led_modes {
                    if command
                        .trim()
                        .starts_with(&<&str>::from(mode).to_lowercase())
                    {
                        return true;
                    }
                }
                if !supported.multizone_led_mode.is_empty() && command.trim().starts_with("multi") {
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
        dbus.proxies()
            .led()
            .set_led_mode(&<AuraEffect>::from(mode))?;
    }

    Ok(())
}

fn handle_led_power(
    dbus: &RogDbusClientBlocking,
    _supported: &LedSupportedFunctions,
    power: &LedPowerCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if power.command().is_none() {
        if !power.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", power.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedPowerCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_string()).collect();
            for command in commands.iter() {
                println!("{}", command);
            }
        }

        println!("\nHelp can also be requested on commands, e.g: boot --help");
        return Ok(());
    }

    if let Some(pow) = power.command.as_ref() {
        if pow.help_requested() {
            println!("{}", pow.self_usage());
            return Ok(());
        }

        match pow {
            // TODO: make this a macro or something
            aura_cli::SetAuraEnabled::Boot(arg) => {
                let mut enabled: Vec<AuraControl> = Vec::new();
                let mut disabled: Vec<AuraControl> = Vec::new();
                arg.keyboard.map(|v| {
                    if v {
                        enabled.push(AuraControl::BootKeyb)
                    } else {
                        disabled.push(AuraControl::BootKeyb)
                    }
                });
                arg.logo.map(|v| {
                    if v {
                        enabled.push(AuraControl::BootLogo)
                    } else {
                        disabled.push(AuraControl::BootLogo)
                    }
                });
                arg.lightbar.map(|v| {
                    if v {
                        enabled.push(AuraControl::BootBar)
                    } else {
                        disabled.push(AuraControl::BootBar)
                    }
                });
                if !enabled.is_empty() {
                    dbus.proxies().led().set_leds_enabled(enabled)?;
                }
                if !disabled.is_empty() {
                    dbus.proxies().led().set_leds_disabled(disabled)?;
                }
            }
            aura_cli::SetAuraEnabled::Sleep(arg) => {
                let mut enabled: Vec<AuraControl> = Vec::new();
                let mut disabled: Vec<AuraControl> = Vec::new();
                arg.keyboard.map(|v| {
                    if v {
                        enabled.push(AuraControl::SleepKeyb)
                    } else {
                        disabled.push(AuraControl::SleepKeyb)
                    }
                });
                arg.logo.map(|v| {
                    if v {
                        enabled.push(AuraControl::SleepLogo)
                    } else {
                        disabled.push(AuraControl::SleepLogo)
                    }
                });
                arg.lightbar.map(|v| {
                    if v {
                        enabled.push(AuraControl::SleepBar)
                    } else {
                        disabled.push(AuraControl::SleepBar)
                    }
                });
                if !enabled.is_empty() {
                    dbus.proxies().led().set_leds_enabled(enabled)?;
                }
                if !disabled.is_empty() {
                    dbus.proxies().led().set_leds_disabled(disabled)?;
                }
            }
            aura_cli::SetAuraEnabled::Awake(arg) => {
                let mut enabled: Vec<AuraControl> = Vec::new();
                let mut disabled: Vec<AuraControl> = Vec::new();
                arg.keyboard.map(|v| {
                    if v {
                        enabled.push(AuraControl::AwakeKeyb)
                    } else {
                        disabled.push(AuraControl::AwakeKeyb)
                    }
                });
                arg.logo.map(|v| {
                    if v {
                        enabled.push(AuraControl::AwakeLogo)
                    } else {
                        disabled.push(AuraControl::AwakeLogo)
                    }
                });
                arg.lightbar.map(|v| {
                    if v {
                        enabled.push(AuraControl::AwakeBar)
                    } else {
                        disabled.push(AuraControl::AwakeBar)
                    }
                });
                if !enabled.is_empty() {
                    dbus.proxies().led().set_leds_enabled(enabled)?;
                }
                if !disabled.is_empty() {
                    dbus.proxies().led().set_leds_disabled(disabled)?;
                }
            }
            aura_cli::SetAuraEnabled::Shutdown(arg) => {
                let mut enabled: Vec<AuraControl> = Vec::new();
                let mut disabled: Vec<AuraControl> = Vec::new();
                arg.keyboard.map(|v| {
                    if v {
                        enabled.push(AuraControl::ShutdownKeyb)
                    } else {
                        disabled.push(AuraControl::ShutdownKeyb)
                    }
                });
                arg.logo.map(|v| {
                    if v {
                        enabled.push(AuraControl::ShutdownLogo)
                    } else {
                        disabled.push(AuraControl::ShutdownLogo)
                    }
                });
                arg.lightbar.map(|v| {
                    if v {
                        enabled.push(AuraControl::ShutdownBar)
                    } else {
                        disabled.push(AuraControl::ShutdownBar)
                    }
                });
                if !enabled.is_empty() {
                    dbus.proxies().led().set_leds_enabled(enabled)?;
                }
                if !disabled.is_empty() {
                    dbus.proxies().led().set_leds_disabled(disabled)?;
                }
            }
        }
    }

    Ok(())
}

fn handle_profile(
    dbus: &RogDbusClientBlocking,
    supported: &PlatformProfileFunctions,
    cmd: &ProfileCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.platform_profile {
        println!("Profiles not supported by either this kernel or by the laptop.");
        return Err(ProfileError::NotSupported.into());
    }

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
    dbus: &RogDbusClientBlocking,
    supported: &PlatformProfileFunctions,
    cmd: &FanCurveCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.fan_curves {
        println!("Fan-curves not supported by either this kernel or by the laptop.");
        println!("This requires kernel 5.17 or the fan curve patch listed in the readme.");
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
            dbus.proxies().profile().set_fan_curve(profile, curve)?;
        }
    }

    Ok(())
}

fn handle_bios_option(
    dbus: &RogDbusClientBlocking,
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
                line.contains("sound") && supported.post_sound_toggle
                    || line.contains("GPU") && supported.dedicated_gfx_toggle
            }) {
                println!("{}", line);
            }
        }

        if let Some(opt) = cmd.post_sound_set {
            dbus.proxies().rog_bios().set_post_boot_sound(opt)?;
        }
        if cmd.post_sound_get {
            let res = dbus.proxies().rog_bios().post_boot_sound()? == 1;
            println!("Bios POST sound on: {}", res);
        }
        if let Some(opt) = cmd.dedicated_gfx_set {
            println!("Rebuilding initrd to include drivers");
            dbus.proxies().rog_bios().set_dedicated_graphic_mode(opt)?;
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
            let res = dbus.proxies().rog_bios().dedicated_graphic_mode()? == 1;
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
