mod anime_cli;
mod aura_cli;
mod cli_opts;
mod profiles_cli;

use crate::aura_cli::{LedBrightness, SetAuraBuiltin};
use crate::cli_opts::*;
use anime_cli::{AnimeActions, AnimeCommand};
use gumdrop::{Opt, Options};
use profiles_cli::ProfileCommand;
use rog_anime::{AnimeDataBuffer, AnimeImage, Vec2, ANIME_DATA_LEN};
use rog_aura::{self, AuraEffect};
use rog_dbus::RogDbusClient;
use rog_supported::SupportedFunctions;
use rog_supported::{
    AnimeSupportedFunctions, LedSupportedFunctions, PlatformProfileFunctions,
    RogBiosSupportedFunctions,
};
use std::{env::args, path::Path, sync::mpsc::channel};
use supergfxctl::{
    gfx_vendors::GfxRequiredUserAction,
    special::{get_asus_gsync_gfx_mode, has_asus_gsync_gfx_mode},
    zbus_proxy::GfxProxy,
};
use zbus::Connection;

const PLEASE: &str =
    "Please use `systemctl status asusd` and `journalctl -b -u asusd` for more information";
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

    let (dbus, _) = RogDbusClient::new().map_err(|e| {
        println!("\nIs asusd running?\n");
        println!("{}", PLEASE);
        println!("{}\n", CONFIG_ADVICE);
        e
    })?;

    let supported = dbus
        .proxies()
        .supported()
        .get_supported_functions()
        .map_err(|e| {
            println!("\nIs asusd running?\n");
            println!("{}", PLEASE);
            println!("{}\n", CONFIG_ADVICE);
            e
        })?;

    if parsed.version {
        print_versions();
        println!();
        print_laptop_info();
        println!("{}\n", PLEASE);
        return Ok(());
    }

    if let Err(err) = do_parsed(&parsed, &supported, &dbus) {
        print_error_help(err, &supported);
    }

    Ok(())
}

fn print_error_help(err: Box<dyn std::error::Error>, supported: &SupportedFunctions) {
    println!("Error: {}\n", err);
    print_versions();
    println!();
    print_laptop_info();
    println!();
    println!("Supported laptop functions:\n\n{}", supported);
    println!();
    println!("{}", PLEASE);
    println!("The above may give some indication that an option is not supported");
    println!("or that a config file must be removed or fixed");
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
    println!("  supergfxctl v{}", supergfxctl::VERSION);
}

fn print_laptop_info() {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");

    println!("Product family: {}", prod_family.trim());
    println!("Board name: {}", board_name.trim());
}

fn do_parsed(
    parsed: &CliStart,
    supported: &SupportedFunctions,
    dbus: &RogDbusClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(dbus, &supported.keyboard_led, mode)?,
        Some(CliCommand::Profile(cmd)) => handle_profile(dbus, &supported.platform_profile, cmd)?,
        Some(CliCommand::Graphics(cmd)) => do_gfx(cmd)?,
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

fn do_gfx(command: &GraphicsCommand) -> Result<(), Box<dyn std::error::Error>> {
    if command.mode.is_none() && !command.get && !command.pow && !command.force || command.help {
        println!("{}", command.self_usage());
    }

    let conn = Connection::new_system()?;
    let proxy = GfxProxy::new(&conn)?;

    let (tx, rx) = channel();
    proxy.connect_notify_action(tx)?;

    if let Some(mode) = command.mode {
        if has_asus_gsync_gfx_mode() && get_asus_gsync_gfx_mode()? == 1 {
            println!("You can not change modes until you turn dedicated/G-Sync off and reboot");
            std::process::exit(-1);
        }

        println!("If anything fails check `journalctl -b -u asusd`\n");

        proxy.gfx_write_mode(&mode).map_err(|err|{
            println!("Graphics mode change error. You may be in an invalid state.");
            println!("Check mode with `asusctl graphics -g` and switch to opposite\nmode to correct it, e.g: if integrated, switch to hybrid, or if nvidia, switch to integrated.\n");
            err
        })?;

        loop {
            proxy.next_signal()?;

            if let Ok(res) = rx.try_recv() {
                match res {
                    GfxRequiredUserAction::Integrated => {
                        println!(
                            "You must change to Integrated before you can change to {}",
                            <&str>::from(mode)
                        );
                    }
                    GfxRequiredUserAction::Logout | GfxRequiredUserAction::Reboot => {
                        println!(
                            "Graphics mode changed to {}. User action required is: {}",
                            <&str>::from(mode),
                            <&str>::from(&res)
                        );
                    }
                    GfxRequiredUserAction::None => {
                        println!("Graphics mode changed to {}", <&str>::from(mode));
                    }
                }
            }
            std::process::exit(0)
        }
    }

    if command.get {
        let res = proxy.gfx_get_mode()?;
        println!("Current graphics mode: {}", <&str>::from(res));
    }
    if command.pow {
        let res = proxy.gfx_get_pwr()?;
        println!("Current power status: {}", <&str>::from(&res));
    }

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
    println!("Warning: Profiles should work fine but now depend on power-profiles-daemon v0.9+");
    println!("Warning: Fan-curve support is coming in a 4.1.x release");
    if !cmd.next && !cmd.list {
        if !cmd.help {
            println!("Missing arg or command\n");
        }
        let usage: Vec<String> = ProfileCommand::usage()
            .lines()
            .map(|s| s.to_string())
            .collect();
        for line in usage
            .iter()
            .filter(|line| !line.contains("--curve") || supported.fan_curves)
        {
            println!("{}", line);
        }

        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }

        // println!("Note: turbo, frequency, fan preset and fan curve options will apply to");
        // println!("      to the currently active profile unless a profile name is specified");
        std::process::exit(1);
    }

    if cmd.next {
        dbus.proxies().profile().next_profile()?;
    }
    if cmd.list {
        let res = dbus.proxies().profile().profiles()?;
        res.iter().for_each(|p| println!("{:?}", p));
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
