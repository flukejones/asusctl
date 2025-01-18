use std::convert::TryFrom;
use std::env::args;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;

use anime_cli::{AnimeActions, AnimeCommand};
use aura_cli::{LedPowerCommand1, LedPowerCommand2};
use dmi_id::DMIID;
use fan_curve_cli::FanCurveCommand;
use gumdrop::{Opt, Options};
use log::error;
use rog_anime::usb::get_anime_type;
use rog_anime::{AnimTime, AnimeDataBuffer, AnimeDiagonal, AnimeGif, AnimeImage, AnimeType, Vec2};
use rog_aura::keyboard::{AuraPowerState, LaptopAuraPower};
use rog_aura::{self, AuraDeviceType, AuraEffect, PowerZones};
use rog_dbus::asus_armoury::AsusArmouryProxyBlocking;
use rog_dbus::list_iface_blocking;
use rog_dbus::scsi_aura::ScsiAuraProxyBlocking;
use rog_dbus::zbus_anime::AnimeProxyBlocking;
use rog_dbus::zbus_aura::AuraProxyBlocking;
use rog_dbus::zbus_fan_curves::FanCurvesProxyBlocking;
use rog_dbus::zbus_platform::PlatformProxyBlocking;
use rog_dbus::zbus_slash::SlashProxyBlocking;
use rog_platform::platform::{Properties, ThrottlePolicy};
use rog_profiles::error::ProfileError;
use rog_scsi::AuraMode;
use rog_slash::SlashMode;
use ron::ser::PrettyConfig;
use scsi_cli::ScsiCommand;
use zbus::blocking::proxy::ProxyImpl;
use zbus::blocking::Connection;

use crate::aura_cli::{AuraPowerStates, LedBrightness};
use crate::cli_opts::*;
use crate::slash_cli::SlashCommand;

mod anime_cli;
mod aura_cli;
mod cli_opts;
mod fan_curve_cli;
mod scsi_cli;
mod slash_cli;

fn main() {
    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let self_version = env!("CARGO_PKG_VERSION");
    println!("Starting version {self_version}");
    let args: Vec<String> = args().skip(1).collect();

    let missing_argument_k = gumdrop::Error::missing_argument(Opt::Short('k'));
    let parsed = match CliStart::parse_args_default(&args) {
        Ok(p) => p,
        Err(err) if err.to_string() == missing_argument_k.to_string() => CliStart {
            kbd_bright: Some(LedBrightness::new(None)),
            ..Default::default()
        },
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let conn = Connection::system().unwrap();
    if let Ok(platform_proxy) = PlatformProxyBlocking::new(&conn).map_err(|e| {
        check_service("asusd");
        println!("\nError: {e}\n");
        print_info();
    }) {
        let asusd_version = platform_proxy.version().unwrap();
        if asusd_version != self_version {
            println!("Version mismatch: asusctl = {self_version}, asusd = {asusd_version}");
            return;
        }

        let supported_properties = platform_proxy.supported_properties().unwrap();
        let supported_interfaces = list_iface_blocking().unwrap();

        if parsed.version {
            println!("asusctl v{}", env!("CARGO_PKG_VERSION"));
            println!();
            print_info();
        }

        if let Err(err) = do_parsed(&parsed, &supported_interfaces, &supported_properties, conn) {
            print_error_help(&*err, &supported_interfaces, &supported_properties);
        }
    }
}

fn print_error_help(
    err: &dyn std::error::Error,
    supported_interfaces: &[String],
    supported_properties: &[Properties]
) {
    check_service("asusd");
    println!("\nError: {}\n", err);
    print_info();
    println!();
    println!("Supported interfaces:\n\n{:#?}\n", supported_interfaces);
    println!(
        "Supported properties on Platform:\n\n{:#?}\n",
        supported_properties
    );
}

fn print_info() {
    let dmi = DMIID::new().unwrap_or_default();
    let board_name = dmi.board_name;
    let prod_family = dmi.product_family;
    println!("asusctl version: {}", env!("CARGO_PKG_VERSION"));
    println!(" Product family: {}", prod_family.trim());
    println!("     Board name: {}", board_name.trim());
}

fn check_service(name: &str) -> bool {
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
    }
    false
}

fn find_iface<T>(iface_name: &str) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: ProxyImpl<'static> + From<zbus::Proxy<'static>>
{
    let conn = zbus::blocking::Connection::system().unwrap();
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "xyz.ljones.Asusd", "/").unwrap();
    let interfaces = f.get_managed_objects().unwrap();
    let mut paths = Vec::new();
    for v in interfaces.iter() {
        // let o: Vec<zbus::names::OwnedInterfaceName> = v.1.keys().map(|e|
        // e.to_owned()).collect(); println!("{}, {:?}", v.0, o);
        for k in v.1.keys() {
            if k.as_str() == iface_name {
                // println!("Found {iface_name} device at {}, {}", v.0, k);
                paths.push(v.0.clone());
            }
        }
    }
    if paths.len() > 1 {
        println!("Multiple asusd interfaces devices found");
    }
    if !paths.is_empty() {
        let mut ctrl = Vec::new();
        paths.sort_by(|a, b| a.cmp(b));
        for path in paths {
            ctrl.push(
                T::builder(&conn)
                    .path(path.clone())?
                    .destination("xyz.ljones.Asusd")?
                    .build()?
            );
        }
        return Ok(ctrl);
    }

    Err(format!("Did not find {iface_name}").into())
}

fn do_parsed(
    parsed: &CliStart,
    supported_interfaces: &[String],
    supported_properties: &[Properties],
    conn: Connection
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::Aura(mode)) => handle_led_mode(mode)?,
        Some(CliCommand::AuraPowerOld(pow)) => handle_led_power1(pow)?,
        Some(CliCommand::AuraPower(pow)) => handle_led_power2(pow)?,
        Some(CliCommand::Profile(cmd)) => {
            handle_throttle_profile(&conn, supported_properties, cmd)?
        }
        Some(CliCommand::FanCurve(cmd)) => {
            handle_fan_curve(&conn, cmd)?;
        }
        Some(CliCommand::Graphics(_)) => do_gfx(),
        Some(CliCommand::Anime(cmd)) => handle_anime(cmd)?,
        Some(CliCommand::Slash(cmd)) => handle_slash(cmd)?,
        Some(CliCommand::Scsi(cmd)) => handle_scsi(cmd)?,
        Some(CliCommand::Armoury(cmd)) => handle_armoury_command(cmd)?,
        None => {
            if (!parsed.show_supported
                && parsed.kbd_bright.is_none()
                && parsed.chg_limit.is_none()
                && !parsed.next_kbd_bright
                && !parsed.prev_kbd_bright
                && !parsed.one_shot_chg)
                || parsed.help
            {
                println!("{}", CliStart::usage());
                println!();
                if let Some(cmdlist) = CliStart::command_list() {
                    let dev_type =
                        if let Ok(proxy) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
                            // TODO: commands on all?
                            proxy
                                .first()
                                .unwrap()
                                .device_type()
                                .unwrap_or(AuraDeviceType::Unknown)
                        } else {
                            AuraDeviceType::Unknown
                        };
                    let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                    for command in commands.iter().filter(|command| {
                        if command.trim().starts_with("fan-curve")
                            && !supported_interfaces.contains(&"xyz.ljones.FanCurves".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("aura")
                            && !supported_interfaces.contains(&"xyz.ljones.Aura".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("anime")
                            && !supported_interfaces.contains(&"xyz.ljones.Anime".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("slash")
                            && !supported_interfaces.contains(&"xyz.ljones.Slash".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("platform")
                            && !supported_interfaces.contains(&"xyz.ljones.Platform".to_string())
                        {
                            return false;
                        }

                        if command.trim().starts_with("platform")
                            && !supported_interfaces.contains(&"xyz.ljones.AsusArmoury".to_string())
                        {
                            return false;
                        }

                        if !dev_type.is_old_laptop()
                            && !dev_type.is_tuf_laptop()
                            && command.trim().starts_with("aura-power-old")
                        {
                            return false;
                        }
                        if !dev_type.is_new_laptop() && command.trim().starts_with("aura-power") {
                            return false;
                        }
                        true
                    }) {
                        println!("{}", command);
                    }
                }

                println!("\nExtra help can be requested on any command or subcommand:");
                println!(" asusctl led-mode --help");
                println!(" asusctl led-mode static --help");
            }
        }
    }

    if let Some(brightness) = &parsed.kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                match brightness.level() {
                    None => {
                        let level = aura.brightness()?;
                        println!("Current keyboard led brightness: {level:?}");
                    }
                    Some(level) => aura.set_brightness(rog_aura::LedBrightness::from(level))?
                }
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.next_kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                let brightness = aura.brightness()?;
                aura.set_brightness(brightness.next())?;
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.prev_kbd_bright {
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            for aura in aura.iter() {
                let brightness = aura.brightness()?;
                aura.set_brightness(brightness.prev())?;
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.show_supported {
        println!("Supported Core Functions:\n{:#?}", supported_interfaces);
        println!(
            "Supported Platform Properties:\n{:#?}",
            supported_properties
        );
        if let Ok(aura) = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura") {
            // TODO: multiple RGB check
            let bright = aura.first().unwrap().supported_brightness()?;
            let modes = aura.first().unwrap().supported_basic_modes()?;
            let zones = aura.first().unwrap().supported_basic_zones()?;
            let power = aura.first().unwrap().supported_power_zones()?;
            println!("Supported Keyboard Brightness:\n{:#?}", bright);
            println!("Supported Aura Modes:\n{:#?}", modes);
            println!("Supported Aura Zones:\n{:#?}", zones);
            println!("Supported Aura Power Zones:\n{:#?}", power);
        } else {
            println!("No aura interface found");
        }
    }

    if let Some(chg_limit) = parsed.chg_limit {
        let proxy = PlatformProxyBlocking::new(&conn)?;
        proxy.set_charge_control_end_threshold(chg_limit)?;
    }

    if parsed.one_shot_chg {
        let proxy = PlatformProxyBlocking::new(&conn)?;
        proxy.one_shot_full_charge()?;
    }

    Ok(())
}

fn do_gfx() {
    println!(
        "Please use supergfxctl for graphics switching. supergfxctl is the result of making \
         asusctl graphics switching generic so all laptops can use it"
    );
    println!("This command will be removed in future");
}

fn handle_anime(cmd: &AnimeCommand) -> Result<(), Box<dyn std::error::Error>> {
    if (cmd.command.is_none()
        && cmd.enable_display.is_none()
        && cmd.enable_powersave_anim.is_none()
        && cmd.brightness.is_none()
        && cmd.off_when_lid_closed.is_none()
        && cmd.off_when_suspended.is_none()
        && cmd.off_when_unplugged.is_none()
        && cmd.off_with_his_head.is_none()
        && !cmd.clear)
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }

    let animes = find_iface::<AnimeProxyBlocking>("xyz.ljones.Anime").map_err(|e| {
        error!("Did not find any interface for xyz.ljones.Anime: {e:?}");
        e
    })?;

    for proxy in animes {
        if let Some(enable) = cmd.enable_display {
            proxy.set_enable_display(enable)?;
        }
        if let Some(enable) = cmd.enable_powersave_anim {
            proxy.set_builtins_enabled(enable)?;
        }
        if let Some(bright) = cmd.brightness {
            proxy.set_brightness(bright)?;
        }
        if let Some(enable) = cmd.off_when_lid_closed {
            proxy.set_off_when_lid_closed(enable)?;
        }
        if let Some(enable) = cmd.off_when_suspended {
            proxy.set_off_when_suspended(enable)?;
        }
        if let Some(enable) = cmd.off_when_unplugged {
            proxy.set_off_when_unplugged(enable)?;
        }
        if cmd.off_with_his_head.is_some() {
            println!("Did Alice _really_ make it back from Wonderland?");
        }

        let mut anime_type = get_anime_type();
        if let AnimeType::Unsupported = anime_type {
            if let Some(model) = cmd.override_type {
                anime_type = model;
            }
        }

        if cmd.clear {
            let data = vec![255u8; anime_type.data_length()];
            let tmp = AnimeDataBuffer::from_vec(anime_type, data)?;
            proxy.write(tmp)?;
        }

        if let Some(action) = cmd.command.as_ref() {
            match action {
                AnimeActions::Image(image) => {
                    if image.help_requested() || image.path.is_empty() {
                        println!("Missing arg or command\n\n{}", image.self_usage());
                        if let Some(lst) = image.self_command_list() {
                            println!("\n{}", lst);
                        }
                        return Ok(());
                    }
                    verify_brightness(image.bright);

                    let matrix = AnimeImage::from_png(
                        Path::new(&image.path),
                        image.scale,
                        image.angle,
                        Vec2::new(image.x_pos, image.y_pos),
                        image.bright,
                        anime_type
                    )?;

                    proxy.write(<AnimeDataBuffer>::try_from(&matrix)?)?;
                }
                AnimeActions::PixelImage(image) => {
                    if image.help_requested() || image.path.is_empty() {
                        println!("Missing arg or command\n\n{}", image.self_usage());
                        if let Some(lst) = image.self_command_list() {
                            println!("\n{}", lst);
                        }
                        return Ok(());
                    }
                    verify_brightness(image.bright);

                    let matrix = AnimeDiagonal::from_png(
                        Path::new(&image.path),
                        None,
                        image.bright,
                        anime_type
                    )?;

                    proxy.write(matrix.into_data_buffer(anime_type)?)?;
                }
                AnimeActions::Gif(gif) => {
                    if gif.help_requested() || gif.path.is_empty() {
                        println!("Missing arg or command\n\n{}", gif.self_usage());
                        if let Some(lst) = gif.self_command_list() {
                            println!("\n{}", lst);
                        }
                        return Ok(());
                    }
                    verify_brightness(gif.bright);

                    let matrix = AnimeGif::from_gif(
                        Path::new(&gif.path),
                        gif.scale,
                        gif.angle,
                        Vec2::new(gif.x_pos, gif.y_pos),
                        AnimTime::Count(1),
                        gif.bright,
                        anime_type
                    )?;

                    let mut loops = gif.loops as i32;
                    loop {
                        for frame in matrix.frames() {
                            proxy.write(frame.frame().clone())?;
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
                        return Ok(());
                    }
                    verify_brightness(gif.bright);

                    let matrix = AnimeGif::from_diagonal_gif(
                        Path::new(&gif.path),
                        AnimTime::Count(1),
                        gif.bright,
                        anime_type
                    )?;

                    let mut loops = gif.loops as i32;
                    loop {
                        for frame in matrix.frames() {
                            proxy.write(frame.frame().clone())?;
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
                AnimeActions::SetBuiltins(builtins) => {
                    if builtins.help_requested() || builtins.set.is_none() {
                        println!(
                            "\nAny unspecified args will be set to default (first shown var)\n"
                        );
                        println!("\n{}", builtins.self_usage());
                        if let Some(lst) = builtins.self_command_list() {
                            println!("\n{}", lst);
                        }
                        return Ok(());
                    }

                    proxy.set_builtin_animations(rog_anime::Animations {
                        boot: builtins.boot,
                        awake: builtins.awake,
                        sleep: builtins.sleep,
                        shutdown: builtins.shutdown
                    })?;
                }
            }
        }
    }
    Ok(())
}

fn verify_brightness(brightness: f32) {
    if !(0.0..=1.0).contains(&brightness) {
        println!(
            "Image and global brightness must be between 0.0 and 1.0 (inclusive), was {}",
            brightness
        );
    }
}

fn handle_slash(cmd: &SlashCommand) -> Result<(), Box<dyn std::error::Error>> {
    if (cmd.brightness.is_none()
        && cmd.interval.is_none()
        && cmd.show_on_boot.is_none()
        && cmd.show_on_shutdown.is_none()
        && cmd.show_on_sleep.is_none()
        && cmd.show_on_battery.is_none()
        && cmd.show_battery_warning.is_none()
        && cmd.mode.is_none()
        && !cmd.list
        && !cmd.enable
        && !cmd.disable)
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }

    let slashes = find_iface::<SlashProxyBlocking>("xyz.ljones.Slash")?;
    for proxy in slashes {
        if cmd.enable {
            proxy.set_enabled(true)?;
        }
        if cmd.disable {
            proxy.set_enabled(false)?;
        }
        if let Some(brightness) = cmd.brightness {
            proxy.set_brightness(brightness)?;
        }
        if let Some(interval) = cmd.interval {
            proxy.set_interval(interval)?;
        }
        if let Some(slash_mode) = cmd.mode {
            proxy.set_mode(slash_mode)?;
        }
        if let Some(show) = cmd.show_on_boot {
            proxy.set_show_on_boot(show)?;
        }

        if let Some(show) = cmd.show_on_shutdown {
            proxy.set_show_on_shutdown(show)?;
        }
        if let Some(show) = cmd.show_on_sleep {
            proxy.set_show_on_sleep(show)?;
        }
        if let Some(show) = cmd.show_on_battery {
            proxy.set_show_on_battery(show)?;
        }
        if let Some(show) = cmd.show_battery_warning {
            proxy.set_show_battery_warning(show)?;
        }
    }
    if cmd.list {
        let res = SlashMode::list();
        for p in &res {
            println!("{:?}", p);
        }
    }

    Ok(())
}

fn handle_scsi(cmd: &ScsiCommand) -> Result<(), Box<dyn std::error::Error>> {
    if (!cmd.list && cmd.enable.is_none() && cmd.mode.is_none() && cmd.colours.is_empty())
        || cmd.help
    {
        println!("Missing arg or command\n\n{}", cmd.self_usage());
        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
    }

    let scsis = find_iface::<ScsiAuraProxyBlocking>("xyz.ljones.ScsiAura")?;

    for scsi in scsis {
        if let Some(enable) = cmd.enable {
            scsi.set_enabled(enable)?;
        }

        if let Some(mode) = cmd.mode {
            dbg!(mode as u8);
            scsi.set_led_mode(mode).unwrap();
        }

        let mut mode = scsi.led_mode_data()?;
        let mut do_update = false;
        if !cmd.colours.is_empty() {
            for (count, c) in cmd.colours.iter().enumerate() {
                if count == 0 {
                    mode.colour1 = *c;
                }
                if count == 1 {
                    mode.colour2 = *c;
                }
                if count == 2 {
                    mode.colour3 = *c;
                }
                if count == 3 {
                    mode.colour4 = *c;
                }
            }
            do_update = true;
        }

        if let Some(speed) = cmd.speed {
            mode.speed = speed;
            do_update = true;
        }

        if let Some(dir) = cmd.direction {
            mode.direction = dir;
            do_update = true;
        }

        if do_update {
            scsi.set_led_mode_data(mode.clone())?;
        }

        // let mode_ret = scsi.led_mode_data()?;
        // assert_eq!(mode, mode_ret);
        println!("{mode}");
    }

    if cmd.list {
        let res = AuraMode::list();
        for p in &res {
            println!("{:?}", p);
        }
    }

    Ok(())
}

fn handle_led_mode(mode: &LedModeCommand) -> Result<(), Box<dyn std::error::Error>> {
    if mode.command.is_none() && !mode.prev_mode && !mode.next_mode {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedModeCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            // TODO: multiple rgb check
            let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
            let modes = aura.first().unwrap().supported_basic_modes()?;
            for command in commands.iter().filter(|command| {
                for mode in &modes {
                    let mut mode = <&str>::from(mode).to_string();
                    if let Some(pos) = mode.chars().skip(1).position(|c| c.is_uppercase()) {
                        mode.insert(pos + 1, '-');
                    }
                    if command.trim().starts_with(&mode.to_lowercase()) {
                        return true;
                    }
                }
                // TODO
                // if !supported.basic_zones.is_empty() && command.trim().starts_with("multi") {
                //     return true;
                // }
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
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
    if mode.next_mode {
        for aura in aura {
            let mode = aura.led_mode()?;
            let modes = aura.supported_basic_modes()?;
            let mut pos = modes.iter().position(|m| *m == mode).unwrap() + 1;
            if pos >= modes.len() {
                pos = 0;
            }
            aura.set_led_mode(modes[pos])?;
        }
    } else if mode.prev_mode {
        for aura in aura {
            let mode = aura.led_mode()?;
            let modes = aura.supported_basic_modes()?;
            let mut pos = modes.iter().position(|m| *m == mode).unwrap();
            if pos == 0 {
                pos = modes.len() - 1;
            } else {
                pos -= 1;
            }
            aura.set_led_mode(modes[pos])?;
        }
    } else if let Some(mode) = mode.command.as_ref() {
        if mode.help_requested() {
            println!("{}", mode.self_usage());
            return Ok(());
        }
        for aura in aura {
            aura.set_led_mode_data(<AuraEffect>::from(mode))?;
        }
    }

    Ok(())
}

fn handle_led_power1(power: &LedPowerCommand1) -> Result<(), Box<dyn std::error::Error>> {
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
    for aura in aura {
        let dev_type = aura.device_type()?;
        if !dev_type.is_old_laptop() && !dev_type.is_tuf_laptop() {
            println!("This option applies only to keyboards 2021+");
        }

        if power.awake.is_none()
            && power.sleep.is_none()
            && power.boot.is_none()
            && !power.keyboard
            && !power.lightbar
        {
            if !power.help {
                println!("Missing arg or command\n");
            }
            println!("{}\n", power.self_usage());
            return Ok(());
        }

        if dev_type.is_old_laptop() || dev_type.is_tuf_laptop() {
            handle_led_power_1_do_1866(&aura, power)?;
            return Ok(());
        }
    }

    println!("These options are for keyboards of product ID 0x1866 or TUF only");
    Ok(())
}

fn handle_led_power_1_do_1866(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand1
) -> Result<(), Box<dyn std::error::Error>> {
    let mut states = Vec::new();
    if power.keyboard {
        states.push(AuraPowerState {
            zone: PowerZones::Keyboard,
            boot: power.boot.unwrap_or_default(),
            awake: power.awake.unwrap_or_default(),
            sleep: power.sleep.unwrap_or_default(),
            shutdown: false
        });
    }
    if power.lightbar {
        states.push(AuraPowerState {
            zone: PowerZones::Lightbar,
            boot: power.boot.unwrap_or_default(),
            awake: power.awake.unwrap_or_default(),
            sleep: power.sleep.unwrap_or_default(),
            shutdown: false
        });
    }

    let states = LaptopAuraPower { states };
    aura.set_led_power(states)?;
    Ok(())
}

fn handle_led_power2(power: &LedPowerCommand2) -> Result<(), Box<dyn std::error::Error>> {
    let aura = find_iface::<AuraProxyBlocking>("xyz.ljones.Aura")?;
    for aura in aura {
        let dev_type = aura.device_type()?;
        if !dev_type.is_new_laptop() {
            println!("This option applies only to keyboards 2021+");
            continue;
        }

        if power.command().is_none() {
            if !power.help {
                println!("Missing arg or command\n");
            }
            println!("{}\n", power.self_usage());
            println!("Commands available");

            if let Some(cmdlist) = LedPowerCommand2::command_list() {
                let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                for command in &commands {
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

            let mut states = aura.led_power()?;
            let mut set = |zone: PowerZones, set_to: &AuraPowerStates| {
                for state in states.states.iter_mut() {
                    if state.zone == zone {
                        state.boot = set_to.boot;
                        state.awake = set_to.awake;
                        state.sleep = set_to.sleep;
                        state.shutdown = set_to.shutdown;
                        break;
                    }
                }
            };

            if let Some(cmd) = &power.command {
                match cmd {
                    aura_cli::SetAuraZoneEnabled::Keyboard(k) => set(PowerZones::Keyboard, k),
                    aura_cli::SetAuraZoneEnabled::Logo(l) => set(PowerZones::Logo, l),
                    aura_cli::SetAuraZoneEnabled::Lightbar(l) => set(PowerZones::Lightbar, l),
                    aura_cli::SetAuraZoneEnabled::Lid(l) => set(PowerZones::Lid, l),
                    aura_cli::SetAuraZoneEnabled::RearGlow(r) => set(PowerZones::RearGlow, r),
                    aura_cli::SetAuraZoneEnabled::Ally(r) => set(PowerZones::Ally, r)
                }
            }

            aura.set_led_power(states)?;
        }
    }

    Ok(())
}

fn handle_throttle_profile(
    conn: &Connection,
    supported: &[Properties],
    cmd: &ProfileCommand
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.contains(&Properties::ThrottlePolicy) {
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
        return Ok(());
    }

    let proxy = PlatformProxyBlocking::new(conn)?;
    let current = proxy.throttle_thermal_policy()?;

    if cmd.next {
        proxy.set_throttle_thermal_policy(current.next())?;
    } else if let Some(profile) = cmd.profile_set {
        proxy.set_throttle_thermal_policy(profile)?;
    }

    if cmd.list {
        let res = ThrottlePolicy::list();
        for p in &res {
            println!("{:?}", p);
        }
    }

    if cmd.profile_get {
        println!("Active profile is {current:?}");
    }

    Ok(())
}

fn handle_fan_curve(
    conn: &Connection,
    cmd: &FanCurveCommand
) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(fan_proxy) = FanCurvesProxyBlocking::new(conn).map_err(|e| {
        println!("Fan-curves not supported by either this kernel or by the laptop: {e:?}");
    }) else {
        return Err(ProfileError::NotSupported.into());
    };

    if !cmd.get_enabled && !cmd.default && cmd.mod_profile.is_none() {
        if !cmd.help {
            println!("Missing arg or command\n");
        }
        println!("{}", FanCurveCommand::usage());

        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
        return Ok(());
    }

    if (cmd.enable_fan_curves.is_some() || cmd.fan.is_some() || cmd.data.is_some())
        && cmd.mod_profile.is_none()
    {
        println!(
            "--enable-fan-curves, --enable-fan-curve, --fan, and --data options require \
             --mod-profile"
        );
        return Ok(());
    }

    let plat_proxy = PlatformProxyBlocking::new(conn)?;
    if cmd.get_enabled {
        let profile = plat_proxy.throttle_thermal_policy()?;
        let curves = fan_proxy.fan_curve_data(profile)?;
        for curve in curves.iter() {
            println!("{}", String::from(curve));
        }
    }

    if cmd.default {
        let active = plat_proxy.throttle_thermal_policy()?;
        fan_proxy.set_curves_to_defaults(active)?;
    }

    if let Some(profile) = cmd.mod_profile {
        if cmd.enable_fan_curves.is_none() && cmd.data.is_none() {
            let data = fan_proxy.fan_curve_data(profile)?;
            let ron = ron::ser::to_string_pretty(&data, PrettyConfig::new().depth_limit(4))?;
            println!("\nFan curves for {:?}\n\n{}", profile, ron);
        }

        if let Some(enabled) = cmd.enable_fan_curves {
            fan_proxy.set_fan_curves_enabled(profile, enabled)?;
        }

        if let Some(enabled) = cmd.enable_fan_curve {
            if let Some(fan) = cmd.fan {
                fan_proxy.set_profile_fan_curve_enabled(profile, fan, enabled)?;
            } else {
                println!(
                    "--enable-fan-curves, --enable-fan-curve, --fan, and --data options require \
                     --mod-profile"
                );
            }
        }

        if let Some(mut curve) = cmd.data.clone() {
            let fan = cmd.fan.unwrap_or_default();
            curve.set_fan(fan);
            fan_proxy.set_fan_curve(profile, curve)?;
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

fn print_firmware_attr(attr: &AsusArmouryProxyBlocking) -> Result<(), Box<dyn std::error::Error>> {
    let name = attr.name()?;
    println!("{}:", <&str>::from(name));

    let attrs = attr.available_attrs()?;
    if attrs.contains(&"min_value".to_string())
        && attrs.contains(&"max_value".to_string())
        && attrs.contains(&"current_value".to_string())
    {
        let c = attr.current_value()?;
        let min = attr.min_value()?;
        let max = attr.max_value()?;
        println!("  current: {min}..[{c}]..{max}");
        if attrs.contains(&"default_value".to_string()) {
            println!("  default: {}\n", attr.default_value()?);
        } else {
            println!();
        }
    } else if attrs.contains(&"possible_values".to_string())
        && attrs.contains(&"current_value".to_string())
    {
        let c = attr.current_value()?;
        let v = attr.possible_values()?;
        for p in v.iter().enumerate() {
            if p.0 == 0 {
                print!("  current: [");
            }
            if *p.1 == c {
                print!("({c})");
            } else {
                print!("{}", p.1);
            }
            if p.0 < v.len() - 1 {
                print!(",");
            }
            if p.0 == v.len() - 1 {
                print!("]");
            }
        }
        if attrs.contains(&"default_value".to_string()) {
            println!("  default: {}\n", attr.default_value()?);
        } else {
            println!("\n");
        }
    } else if attrs.contains(&"current_value".to_string()) {
        let c = attr.current_value()?;
        println!("  current: {c}\n");
    } else {
        println!();
    }
    Ok(())
}

fn handle_armoury_command(cmd: &ArmouryCommand) -> Result<(), Box<dyn std::error::Error>> {
    {
        if cmd.free.is_empty() || cmd.free.len() % 2 != 0 || cmd.help {
            const USAGE: &str = "Usage: asusctl platform panel_overdrive 1 nv_dynamic_boost 5";
            if cmd.free.len() % 2 != 0 {
                println!(
                    "Incorrect number of args, each attribute label must be paired with a setting:"
                );
                println!("{USAGE}");
                return Ok(());
            }

            if let Ok(attr) = find_iface::<AsusArmouryProxyBlocking>("xyz.ljones.AsusArmoury") {
                println!("\n{USAGE}\n");
                println!("Available firmware attributes: ");
                for attr in attr.iter() {
                    print_firmware_attr(attr)?;
                }
            }
            return Ok(());
        }

        if let Ok(attr) = find_iface::<AsusArmouryProxyBlocking>("xyz.ljones.AsusArmoury") {
            for cmd in cmd.free.chunks(2) {
                for attr in attr.iter() {
                    let name = attr.name()?;
                    if <&str>::from(name) == cmd[0] {
                        attr.set_current_value(cmd[1].parse()?)?;
                        print_firmware_attr(attr)?;
                    }
                }
            }
        }
    }
    Ok(())
}
