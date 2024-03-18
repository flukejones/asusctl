use std::convert::TryFrom;
use std::env::args;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;

use anime_cli::{AnimeActions, AnimeCommand};
use asusd::ctrl_fancurves::FAN_CURVE_ZBUS_NAME;
use aura_cli::{LedPowerCommand1, LedPowerCommand2};
use dmi_id::DMIID;
use fan_curve_cli::FanCurveCommand;
use gumdrop::{Opt, Options};
use rog_anime::usb::get_anime_type;
use rog_anime::{AnimTime, AnimeDataBuffer, AnimeDiagonal, AnimeGif, AnimeImage, AnimeType, Vec2};
use rog_aura::power::KbAuraPowerState;
use rog_aura::usb::{AuraDevRog1, AuraDevTuf, AuraDevice, AuraPowerDev};
use rog_aura::{self, AuraEffect};
use rog_dbus::zbus_aura::AuraProxyBlocking;
use rog_dbus::RogDbusClientBlocking;
use rog_platform::platform::{GpuMode, Properties, ThrottlePolicy};
use rog_profiles::error::ProfileError;

use crate::aura_cli::{AuraPowerStates, LedBrightness};
use crate::cli_opts::*;

mod anime_cli;
mod aura_cli;
mod cli_opts;
mod fan_curve_cli;

fn main() {
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

    if let Ok((dbus, _)) = RogDbusClientBlocking::new().map_err(|e| {
        check_service("asusd");
        println!("\nError: {e}\n");
        print_info();
    }) {
        let supported_properties = dbus.proxies().platform().supported_properties().unwrap();
        let supported_interfaces = dbus.proxies().platform().supported_interfaces().unwrap();

        if parsed.version {
            println!("asusctl v{}", env!("CARGO_PKG_VERSION"));
            println!();
            print_info();
        }

        if let Err(err) = do_parsed(&parsed, &supported_interfaces, &supported_properties, &dbus) {
            print_error_help(&*err, &supported_interfaces, &supported_properties);
        }
    }
}

fn print_error_help(
    err: &dyn std::error::Error,
    supported_interfaces: &[String],
    supported_properties: &[Properties],
) {
    check_service("asusd");
    println!("\nError: {}\n", err);
    print_info();
    println!();
    println!("Supported interfaces:\n\n{:#?}\n", supported_interfaces);
    println!("Supported properties:\n\n{:#?}\n", supported_properties);
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

fn find_aura_iface() -> Result<AuraProxyBlocking<'static>, Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::system().unwrap();
    let f = zbus::blocking::fdo::ObjectManagerProxy::new(&conn, "org.asuslinux.Daemon", "/org")
        .unwrap();
    let interfaces = f.get_managed_objects().unwrap();
    let mut aura_paths = Vec::new();
    for v in interfaces.iter() {
        // let o: Vec<zbus::names::OwnedInterfaceName> = v.1.keys().map(|e|
        // e.to_owned()).collect(); println!("{}, {:?}", v.0, o);
        for k in v.1.keys() {
            if k.as_str() == "org.asuslinux.Aura" {
                println!("Found aura device at {}, {}", v.0, k);
                aura_paths.push(v.0.clone());
            }
        }
    }
    if aura_paths.len() > 1 {
        println!("Multiple aura devices found: {aura_paths:?}");
        println!("TODO: enable selection");
    }
    if let Some(path) = aura_paths.first() {
        return Ok(AuraProxyBlocking::builder(&conn)
            .path(path.clone())?
            .destination("org.asuslinux.Daemon")?
            .build()?);
    }

    Err("No Aura interface".into())
}

fn do_parsed(
    parsed: &CliStart,
    supported_interfaces: &[String],
    supported_properties: &[Properties],
    dbus: &RogDbusClientBlocking<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(&find_aura_iface()?, mode)?,
        Some(CliCommand::LedPow1(pow)) => handle_led_power1(&find_aura_iface()?, pow)?,
        Some(CliCommand::LedPow2(pow)) => handle_led_power2(&find_aura_iface()?, pow)?,
        Some(CliCommand::Profile(cmd)) => handle_throttle_profile(dbus, supported_properties, cmd)?,
        Some(CliCommand::FanCurve(cmd)) => {
            handle_fan_curve(dbus, supported_interfaces, cmd)?;
        }
        Some(CliCommand::Graphics(_)) => do_gfx(),
        Some(CliCommand::Anime(cmd)) => handle_anime(dbus, cmd)?,
        Some(CliCommand::Bios(cmd)) => handle_platform_properties(dbus, supported_properties, cmd)?,
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
                    let dev_type = if let Ok(proxy) = find_aura_iface() {
                        proxy.device_type().unwrap_or(AuraDevice::Unknown)
                    } else {
                        AuraDevice::Unknown
                    };
                    let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                    for command in commands.iter().filter(|command| {
                        if !dev_type.is_old_style()
                            && !dev_type.is_tuf_style()
                            && command.trim().starts_with("led-pow-1")
                        {
                            return false;
                        }
                        if !dev_type.is_new_style() && command.trim().starts_with("led-pow-2") {
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
        if let Ok(aura) = find_aura_iface() {
            match brightness.level() {
                None => {
                    let level = aura.brightness()?;
                    println!("Current keyboard led brightness: {level:?}");
                }
                Some(level) => aura.set_brightness(rog_aura::LedBrightness::from(level))?,
            }
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.next_kbd_bright {
        if let Ok(aura) = find_aura_iface() {
            let brightness = aura.brightness()?;
            aura.set_brightness(brightness.next())?;
        } else {
            println!("No aura interface found");
        }
    }

    if parsed.prev_kbd_bright {
        if let Ok(aura) = find_aura_iface() {
            let brightness = aura.brightness()?;
            aura.set_brightness(brightness.prev())?;
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
        if let Ok(aura) = find_aura_iface() {
            let bright = aura.supported_brightness()?;
            let modes = aura.supported_basic_modes()?;
            let zones = aura.supported_basic_zones()?;
            let power = aura.supported_power_zones()?;
            println!("Supported Keyboard Brightness:\n{:#?}", bright);
            println!("Supported Aura Modes:\n{:#?}", modes);
            println!("Supported Aura Zones:\n{:#?}", zones);
            println!("Supported Aura Power Zones:\n{:#?}", power);
        } else {
            println!("No aura interface found");
        }
    }

    if let Some(chg_limit) = parsed.chg_limit {
        dbus.proxies()
            .platform()
            .set_charge_control_end_threshold(chg_limit)?;
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

fn handle_anime(
    dbus: &RogDbusClientBlocking<'_>,
    cmd: &AnimeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
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
    if let Some(enable) = cmd.enable_display {
        dbus.proxies().anime().set_enable_display(enable)?;
    }
    if let Some(enable) = cmd.enable_powersave_anim {
        dbus.proxies().anime().set_builtins_enabled(enable)?;
    }
    if let Some(bright) = cmd.brightness {
        dbus.proxies().anime().set_brightness(bright)?;
    }
    if let Some(enable) = cmd.off_when_lid_closed {
        dbus.proxies().anime().set_off_when_lid_closed(enable)?;
    }
    if let Some(enable) = cmd.off_when_suspended {
        dbus.proxies().anime().set_off_when_suspended(enable)?;
    }
    if let Some(enable) = cmd.off_when_unplugged {
        dbus.proxies().anime().set_off_when_unplugged(enable)?;
    }
    if cmd.off_with_his_head.is_some() {
        println!("Did Alice _really_ make it back from Wonderland?");
    }

    let mut anime_type = get_anime_type()?;
    if let AnimeType::Unknown = anime_type {
        if let Some(model) = cmd.override_type {
            anime_type = model;
        }
    }

    if cmd.clear {
        let data = vec![255u8; anime_type.data_length()];
        let tmp = AnimeDataBuffer::from_vec(anime_type, data)?;
        dbus.proxies().anime().write(tmp)?;
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
                    anime_type,
                )?;

                dbus.proxies()
                    .anime()
                    .write(<AnimeDataBuffer>::try_from(&matrix)?)?;
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
                    anime_type,
                )?;

                dbus.proxies()
                    .anime()
                    .write(matrix.into_data_buffer(anime_type)?)?;
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
                    return Ok(());
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
            AnimeActions::SetBuiltins(builtins) => {
                if builtins.help_requested() || builtins.set.is_none() {
                    println!("\nAny unspecified args will be set to default (first shown var)\n");
                    println!("\n{}", builtins.self_usage());
                    if let Some(lst) = builtins.self_command_list() {
                        println!("\n{}", lst);
                    }
                    return Ok(());
                }

                dbus.proxies()
                    .anime()
                    .set_builtin_animations(rog_anime::Animations {
                        boot: builtins.boot,
                        awake: builtins.awake,
                        sleep: builtins.sleep,
                        shutdown: builtins.shutdown,
                    })?;
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

fn handle_led_mode(
    aura: &AuraProxyBlocking,
    mode: &LedModeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    // if !supported.contains(&AURA_ZBUS_NAME.to_string()) {
    //     println!("This laptop does not support power options");
    //     return Err(PlatformError::NotSupported.into());
    // }

    if mode.command.is_none() && !mode.prev_mode && !mode.next_mode {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        if let Some(cmdlist) = LedModeCommand::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            let modes = aura.supported_basic_modes()?;
            for command in commands.iter().filter(|command| {
                for mode in &modes {
                    if command
                        .trim()
                        .starts_with(&<&str>::from(mode).to_lowercase())
                    {
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
    if mode.next_mode {
        let mode = aura.led_mode()?;
        let modes = aura.supported_basic_modes()?;
        let mut pos = modes.iter().position(|m| *m == mode).unwrap() + 1;
        if pos >= modes.len() {
            pos = 0;
        }
        aura.set_led_mode(modes[pos])?;
    } else if mode.prev_mode {
        let mode = aura.led_mode()?;
        let modes = aura.supported_basic_modes()?;
        let mut pos = modes.iter().position(|m| *m == mode).unwrap();
        if pos == 0 {
            pos = modes.len() - 1;
        } else {
            pos -= 1;
        }
        aura.set_led_mode(modes[pos])?;
    } else if let Some(mode) = mode.command.as_ref() {
        if mode.help_requested() {
            println!("{}", mode.self_usage());
            return Ok(());
        }
        aura.set_led_mode_data(<AuraEffect>::from(mode))?;
    }

    Ok(())
}

fn handle_led_power1(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand1,
) -> Result<(), Box<dyn std::error::Error>> {
    // if !supported.contains(&AURA_ZBUS_NAME.to_string()) {
    //     println!("This laptop does not support power options");
    //     return Err(PlatformError::NotSupported.into());
    // }
    let dev_type = aura.device_type()?;
    if !dev_type.is_old_style() && !dev_type.is_tuf_style() {
        println!("This option applies only to keyboards 2021+");
    }

    if power.awake.is_none()
        && power.sleep.is_none()
        && power.boot.is_none()
        && power.keyboard.is_none()
        && power.lightbar.is_none()
    {
        if !power.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", power.self_usage());
        return Ok(());
    }

    if dev_type.is_old_style() {
        handle_led_power_1_do_1866(aura, power)?;
        return Ok(());
    }

    if dev_type.is_tuf_style() {
        handle_led_power_1_do_tuf(aura, power)?;
        return Ok(());
    }

    println!("These options are for keyboards of product ID 0x1866 or TUF only");
    Ok(())
}

fn handle_led_power_1_do_1866(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand1,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut enabled: Vec<AuraDevRog1> = Vec::new();
    let mut disabled: Vec<AuraDevRog1> = Vec::new();

    let mut check = |e: Option<bool>, a: AuraDevRog1| {
        if let Some(arg) = e {
            if arg {
                enabled.push(a);
            } else {
                disabled.push(a);
            }
        }
    };

    check(power.awake, AuraDevRog1::Awake);
    check(power.boot, AuraDevRog1::Boot);
    check(power.sleep, AuraDevRog1::Sleep);
    check(power.keyboard, AuraDevRog1::Keyboard);
    check(power.lightbar, AuraDevRog1::Lightbar);

    let data = AuraPowerDev {
        old_rog: enabled,
        ..Default::default()
    };
    aura.set_led_power(data)?; // TODO: verify this

    Ok(())
}

fn handle_led_power_1_do_tuf(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand1,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut enabled: Vec<AuraDevTuf> = Vec::new();
    let mut disabled: Vec<AuraDevTuf> = Vec::new();

    let mut check = |e: Option<bool>, a: AuraDevTuf| {
        if let Some(arg) = e {
            if arg {
                enabled.push(a);
            } else {
                disabled.push(a);
            }
        }
    };

    check(power.awake, AuraDevTuf::Awake);
    check(power.boot, AuraDevTuf::Boot);
    check(power.sleep, AuraDevTuf::Sleep);
    check(power.keyboard, AuraDevTuf::Keyboard);

    let data = AuraPowerDev {
        tuf: enabled,
        ..Default::default()
    };
    aura.set_led_power(data)?; // TODO: verify this

    Ok(())
}

fn handle_led_power2(
    aura: &AuraProxyBlocking,
    power: &LedPowerCommand2,
) -> Result<(), Box<dyn std::error::Error>> {
    // if !supported.contains(&AURA_ZBUS_NAME.to_string()) {
    //     println!("This laptop does not support power options");
    //     return Err(PlatformError::NotSupported.into());
    // }
    let dev_type = aura.device_type()?;
    if !dev_type.is_new_style() {
        println!("This option applies only to keyboards 2021+");
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

        let set = |power: &mut KbAuraPowerState, set_to: &AuraPowerStates| {
            power.boot = set_to.boot;
            power.awake = set_to.awake;
            power.sleep = set_to.sleep;
            power.shutdown = set_to.shutdown;
        };

        let mut enabled = aura.led_power()?;
        if let Some(cmd) = &power.command {
            match cmd {
                aura_cli::SetAuraZoneEnabled::Keyboard(k) => set(&mut enabled.rog.keyboard, k),
                aura_cli::SetAuraZoneEnabled::Logo(l) => set(&mut enabled.rog.logo, l),
                aura_cli::SetAuraZoneEnabled::Lightbar(l) => set(&mut enabled.rog.lightbar, l),
                aura_cli::SetAuraZoneEnabled::Lid(l) => set(&mut enabled.rog.lid, l),
                aura_cli::SetAuraZoneEnabled::RearGlow(r) => set(&mut enabled.rog.rear_glow, r),
            }
        }

        aura.set_led_power(enabled)?;
    }

    Ok(())
}

fn handle_throttle_profile(
    dbus: &RogDbusClientBlocking<'_>,
    supported: &[Properties],
    cmd: &ProfileCommand,
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
    let current = dbus.proxies().platform().throttle_thermal_policy()?;

    if cmd.next {
        dbus.proxies()
            .platform()
            .set_throttle_thermal_policy(current.next())?;
    } else if let Some(profile) = cmd.profile_set {
        dbus.proxies()
            .platform()
            .set_throttle_thermal_policy(profile)?;
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
    dbus: &RogDbusClientBlocking<'_>,
    supported: &[String],
    cmd: &FanCurveCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !supported.contains(&FAN_CURVE_ZBUS_NAME.to_string()) {
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

    if cmd.get_enabled {
        let profile = dbus.proxies().platform().throttle_thermal_policy()?;
        let curves = dbus.proxies().fan_curves().fan_curve_data(profile)?;
        for curve in curves.iter() {
            println!("{}", String::from(curve));
        }
    }

    if cmd.default {
        let active = dbus.proxies().platform().throttle_thermal_policy()?;
        dbus.proxies().fan_curves().set_curves_to_defaults(active)?;
    }

    if let Some(profile) = cmd.mod_profile {
        if cmd.enable_fan_curves.is_none() && cmd.data.is_none() {
            let data = dbus.proxies().fan_curves().fan_curve_data(profile)?;
            let data = toml::to_string(&data)?;
            println!("\nFan curves for {:?}\n\n{}", profile, data);
        }

        if let Some(enabled) = cmd.enable_fan_curves {
            dbus.proxies()
                .fan_curves()
                .set_fan_curves_enabled(profile, enabled)?;
        }

        if let Some(enabled) = cmd.enable_fan_curve {
            if let Some(fan) = cmd.fan {
                dbus.proxies()
                    .fan_curves()
                    .set_profile_fan_curve_enabled(profile, fan, enabled)?;
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
            dbus.proxies().fan_curves().set_fan_curve(profile, curve)?;
        }
    }

    Ok(())
}

fn handle_platform_properties(
    dbus: &RogDbusClientBlocking<'_>,
    supported: &[Properties],
    cmd: &BiosCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        if (cmd.gpu_mux_mode_set.is_none()
            && !cmd.gpu_mux_mode_get
            && cmd.post_sound_set.is_none()
            && !cmd.post_sound_get
            && cmd.panel_overdrive_set.is_none()
            && !cmd.panel_overdrive_get)
            || cmd.help
        {
            println!("Missing arg or command\n");

            let usage: Vec<String> = BiosCommand::usage().lines().map(|s| s.to_owned()).collect();

            for line in usage.iter().filter(|line| {
                line.contains("sound") && supported.contains(&Properties::PostAnimationSound)
                    || line.contains("GPU") && supported.contains(&Properties::GpuMuxMode)
                    || line.contains("panel") && supported.contains(&Properties::PanelOd)
            }) {
                println!("{}", line);
            }
        }

        if let Some(opt) = cmd.post_sound_set {
            dbus.proxies().platform().set_boot_sound(opt)?;
        }
        if cmd.post_sound_get {
            let res = dbus.proxies().platform().boot_sound()?;
            println!("Bios POST sound on: {}", res);
        }

        if let Some(opt) = cmd.gpu_mux_mode_set {
            println!("Rebuilding initrd to include drivers");
            dbus.proxies()
                .platform()
                .set_gpu_mux_mode(GpuMode::from_mux(opt))?;
            println!(
                "The mode change is not active until you reboot, on boot the bios will make the \
                 required change"
            );
        }
        if cmd.gpu_mux_mode_get {
            let res = dbus.proxies().platform().gpu_mux_mode()?;
            println!("Bios GPU MUX: {:?}", res);
        }

        if let Some(opt) = cmd.panel_overdrive_set {
            dbus.proxies().platform().set_panel_od(opt)?;
        }
        if cmd.panel_overdrive_get {
            let res = dbus.proxies().platform().panel_od()?;
            println!("Panel overdrive on: {}", res);
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
