use std::convert::TryFrom;
use std::env::args;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;

use anime_cli::{AnimeActions, AnimeCommand};
use aura_cli::{LedPowerCommand1, LedPowerCommand2};
use gumdrop::{Opt, Options};
use profiles_cli::{FanCurveCommand, ProfileCommand};
use rog_anime::usb::get_anime_type;
use rog_anime::{AnimTime, AnimeDataBuffer, AnimeDiagonal, AnimeGif, AnimeImage, AnimeType, Vec2};
use rog_aura::power::KbAuraPowerState;
use rog_aura::usb::{AuraDevRog1, AuraDevTuf, AuraDevice, AuraPowerDev};
use rog_aura::{self, AuraEffect};
use rog_dbus::RogDbusClientBlocking;
use rog_platform::platform::GpuMode;
use rog_platform::supported::*;
use rog_profiles::error::ProfileError;

use crate::aura_cli::{AuraPowerStates, LedBrightness};
use crate::cli_opts::*;

mod anime_cli;
mod aura_cli;
mod cli_opts;
mod profiles_cli;

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
        print_error_help(&e, None);
    }) {
        if let Ok(supported) = dbus
            .proxies()
            .supported()
            .supported_functions()
            .map_err(|e| {
                print_error_help(&e, None);
            })
        {
            if parsed.version {
                println!("asusctl v{}", env!("CARGO_PKG_VERSION"));
                println!();
                print_info();
            }

            if let Err(err) = do_parsed(&parsed, &supported, &dbus) {
                print_error_help(&*err, Some(&supported));
            }
        }
    }
}

fn print_error_help(err: &dyn std::error::Error, supported: Option<&SupportedFunctions>) {
    check_service("asusd");
    println!("\nError: {}\n", err);
    print_info();
    if let Some(supported) = supported {
        println!();
        println!("Supported laptop functions:\n\n{}", supported);
    }
}

fn print_info() {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");
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

fn do_parsed(
    parsed: &CliStart,
    supported: &SupportedFunctions,
    dbus: &RogDbusClientBlocking<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    match &parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(dbus, &supported.keyboard_led, mode)?,
        Some(CliCommand::LedPow1(pow)) => handle_led_power1(dbus, &supported.keyboard_led, pow)?,
        Some(CliCommand::LedPow2(pow)) => handle_led_power2(dbus, &supported.keyboard_led, pow)?,
        Some(CliCommand::Profile(cmd)) => handle_profile(dbus, &supported.platform_profile, cmd)?,
        Some(CliCommand::FanCurve(cmd)) => {
            handle_fan_curve(dbus, &supported.platform_profile, cmd)?;
        }
        Some(CliCommand::Graphics(_)) => do_gfx(),
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
                    let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
                    for command in commands.iter().filter(|command| {
                        if !matches!(
                            supported.keyboard_led.dev_id,
                            AuraDevice::X1854
                                | AuraDevice::X1869
                                | AuraDevice::X1866
                                | AuraDevice::Tuf
                        ) && command.trim().starts_with("led-pow-1")
                        {
                            return false;
                        }
                        if supported.keyboard_led.dev_id != AuraDevice::X19b6
                            && command.trim().starts_with("led-pow-2")
                        {
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
        dbus.proxies()
            .charge()
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
    _supported: &AnimeSupportedFunctions,
    cmd: &AnimeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if (cmd.command.is_none()
        && cmd.enable_display.is_none()
        && cmd.enable_powersave_anim.is_none()
        && cmd.brightness.is_none()
        && cmd.image_brightness.is_none()
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
    if let Some(bright) = cmd.image_brightness {
        verify_brightness(bright);
        dbus.proxies().anime().set_image_brightness(bright)?;
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

                dbus.proxies().anime().set_builtin_animations(
                    builtins.boot,
                    builtins.awake,
                    builtins.sleep,
                    builtins.shutdown,
                )?;
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
    dbus: &RogDbusClientBlocking<'_>,
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
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            for command in commands.iter().filter(|command| {
                for mode in &supported.basic_modes {
                    if command
                        .trim()
                        .starts_with(&<&str>::from(mode).to_lowercase())
                    {
                        return true;
                    }
                }
                if !supported.basic_zones.is_empty() && command.trim().starts_with("multi") {
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

fn handle_led_power1(
    dbus: &RogDbusClientBlocking<'_>,
    supported: &LedSupportedFunctions,
    power: &LedPowerCommand1,
) -> Result<(), Box<dyn std::error::Error>> {
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

    if matches!(
        supported.dev_id,
        AuraDevice::X1854 | AuraDevice::X1869 | AuraDevice::X1866
    ) {
        handle_led_power_1_do_1866(dbus, power)?;
        return Ok(());
    }

    if matches!(supported.dev_id, AuraDevice::Tuf) {
        handle_led_power_1_do_tuf(dbus, power)?;
        return Ok(());
    }

    println!("These options are for keyboards of product ID 0x1866 or TUF only");
    Ok(())
}

fn handle_led_power_1_do_1866(
    dbus: &RogDbusClientBlocking<'_>,
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
    dbus.proxies().led().set_led_power(data, true)?;

    let data = AuraPowerDev {
        old_rog: disabled,
        ..Default::default()
    };
    dbus.proxies().led().set_led_power(data, false)?;

    Ok(())
}

fn handle_led_power_1_do_tuf(
    dbus: &RogDbusClientBlocking<'_>,
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
    dbus.proxies().led().set_led_power(data, true)?;

    let data = AuraPowerDev {
        tuf: disabled,
        ..Default::default()
    };
    dbus.proxies().led().set_led_power(data, false)?;

    Ok(())
}

fn handle_led_power2(
    dbus: &RogDbusClientBlocking<'_>,
    supported: &LedSupportedFunctions,
    power: &LedPowerCommand2,
) -> Result<(), Box<dyn std::error::Error>> {
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

        if supported.dev_id != AuraDevice::X19b6 {
            println!("This option applies only to keyboards with product ID 0x19b6");
        }

        let set = |power: &mut KbAuraPowerState, set_to: &AuraPowerStates| {
            power.boot = set_to.boot;
            power.awake = set_to.awake;
            power.sleep = set_to.sleep;
            power.shutdown = set_to.shutdown;
        };

        let mut enabled = dbus.proxies().led().led_power()?;
        if let Some(cmd) = &power.command {
            match cmd {
                aura_cli::SetAuraZoneEnabled::Keyboard(k) => set(&mut enabled.rog.keyboard, k),
                aura_cli::SetAuraZoneEnabled::Logo(l) => set(&mut enabled.rog.logo, l),
                aura_cli::SetAuraZoneEnabled::Lightbar(l) => set(&mut enabled.rog.lightbar, l),
                aura_cli::SetAuraZoneEnabled::Lid(l) => set(&mut enabled.rog.lid, l),
                aura_cli::SetAuraZoneEnabled::RearGlow(r) => set(&mut enabled.rog.rear_glow, r),
            }
        }

        dbus.proxies().led().set_led_power(enabled, true)?;
    }

    Ok(())
}

fn handle_profile(
    dbus: &RogDbusClientBlocking<'_>,
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
        return Ok(());
    }

    if cmd.next {
        dbus.proxies().profile().next_profile()?;
    } else if let Some(profile) = cmd.profile_set {
        dbus.proxies().profile().set_active_profile(profile)?;
    }

    if cmd.list {
        let res = dbus.proxies().profile().profiles()?;
        for p in &res {
            println!("{:?}", p);
        }
    }

    if cmd.profile_get {
        let res = dbus.proxies().profile().active_profile()?;
        println!("Active profile is {:?}", res);
    }

    Ok(())
}

fn handle_fan_curve(
    dbus: &RogDbusClientBlocking<'_>,
    supported: &PlatformProfileFunctions,
    cmd: &FanCurveCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if supported.fans.is_empty() {
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
        return Ok(());
    }

    if (cmd.enabled.is_some() || cmd.fan.is_some() || cmd.data.is_some())
        && cmd.mod_profile.is_none()
    {
        println!("--enabled, --fan, and --data options require --mod-profile");
        return Ok(());
    }

    if cmd.get_enabled {
        // TODO:
        // let res = dbus.proxies().profile().enabled_fan_profiles()?;
        // println!("{:?}", res);
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
    dbus: &RogDbusClientBlocking<'_>,
    supported: &RogBiosSupportedFunctions,
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
                line.contains("sound") && supported.post_sound
                    || line.contains("GPU") && supported.gpu_mux
                    || line.contains("panel") && supported.panel_overdrive
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

        if let Some(opt) = cmd.gpu_mux_mode_set {
            println!("Rebuilding initrd to include drivers");
            dbus.proxies()
                .rog_bios()
                .set_gpu_mux_mode(GpuMode::from_mux(opt))?;
            println!(
                "The mode change is not active until you reboot, on boot the bios will make the \
                 required change"
            );
        }
        if cmd.gpu_mux_mode_get {
            let res = dbus.proxies().rog_bios().gpu_mux_mode()?;
            println!("Bios GPU MUX: {:?}", res);
        }

        if let Some(opt) = cmd.panel_overdrive_set {
            dbus.proxies().rog_bios().set_panel_od(opt)?;
        }
        if cmd.panel_overdrive_get {
            let res = dbus.proxies().rog_bios().panel_od()?;
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
