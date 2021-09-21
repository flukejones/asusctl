use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};

use ::zbus::{fdo, Connection, ObjectServer};
use log::LevelFilter;
use log::{error, info, warn};

use daemon::ctrl_anime::config::AnimeConfig;
use daemon::ctrl_anime::zbus::CtrlAnimeZbus;
use daemon::ctrl_anime::*;
use daemon::ctrl_aura::config::AuraConfig;
use daemon::ctrl_aura::controller::{
    CtrlKbdLed, CtrlKbdLedReloader, CtrlKbdLedTask, CtrlKbdLedZbus,
};
use daemon::ctrl_charge::CtrlCharge;
use daemon::ctrl_profiles::config::ProfileConfig;
use daemon::ctrl_profiles::controller::CtrlProfileTask;
use daemon::ctrl_rog_bios::CtrlRogBios;
use daemon::error::RogError;
use daemon::{
    config::Config, ctrl_supported::SupportedFunctions, laptops::print_board_info, GetSupported,
};
use daemon::{
    ctrl_profiles::{controller::CtrlPlatformProfile, zbus::ProfileZbus},
    laptops::LaptopLedData,
};
use daemon::{CtrlTask, Reloadable, ZbusAdd};
use rog_dbus::DBUS_NAME;
use rog_profiles::Profile;

static PROFILE_CONFIG_PATH: &str = "/etc/asusd/profile.conf";

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .init();

    let is_service = match env::var_os("IS_SERVICE") {
        Some(val) => val == "1",
        None => false,
    };

    if !is_service {
        println!("asusd schould be only run from the right systemd service");
        println!(
            "do not run in your terminal, if you need an logs please use journalctl -b -u asusd"
        );
        println!("asusd will now exit");
        return Ok(());
    }

    info!("       daemon v{}", daemon::VERSION);
    info!("    rog-anime v{}", rog_anime::VERSION);
    info!("     rog-aura v{}", rog_aura::VERSION);
    info!("     rog-dbus v{}", rog_dbus::VERSION);
    info!(" rog-profiles v{}", rog_profiles::VERSION);
    info!("rog-supported v{}", rog_supported::VERSION);

    start_daemon()?;
    Ok(())
}

/// The actual main loop for the daemon
fn start_daemon() -> Result<(), Box<dyn Error>> {
    let supported = SupportedFunctions::get_supported();
    print_board_info();
    println!("{}", serde_json::to_string_pretty(&supported)?);

    // Start zbus server
    let connection = Connection::new_system()?;
    let fdo_connection = fdo::DBusProxy::new(&connection)?;
    let mut object_server = ObjectServer::new(&connection);

    let config = Config::load();
    let config = Arc::new(Mutex::new(config));

    supported.add_to_server(&mut object_server);

    match CtrlRogBios::new(config.clone()) {
        Ok(mut ctrl) => {
            // Do a reload of any settings
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
            // Then register to dbus server
            ctrl.add_to_server(&mut object_server);
        }
        Err(err) => {
            error!("rog_bios_control: {}", err);
        }
    }

    match CtrlCharge::new(config) {
        Ok(mut ctrl) => {
            // Do a reload of any settings
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
            // Then register to dbus server
            ctrl.add_to_server(&mut object_server);
        }
        Err(err) => {
            error!("charge_control: {}", err);
        }
    }

    if Profile::is_platform_profile_supported() {
        let profile_config = ProfileConfig::load(PROFILE_CONFIG_PATH.into());
        match CtrlPlatformProfile::new(profile_config) {
            Ok(mut ctrl) => {
                ctrl.reload()
                    .unwrap_or_else(|err| warn!("Profile control: {}", err));

                let tmp = Arc::new(Mutex::new(ctrl));
                ProfileZbus::new(tmp.clone()).add_to_server(&mut object_server);

                let task = CtrlProfileTask::new(tmp);
                thread::Builder::new().name("profile tasks".into()).spawn(
                    move || -> Result<(), RogError> {
                        loop {
                            task.do_task()?;
                            sleep(Duration::from_millis(100));
                        }
                    },
                )?;
            }
            Err(err) => {
                error!("Profile control: {}", err);
            }
        }
    } else {
        warn!("platform_profile support not found. This requires kernel 5.15.x or the patch applied: https://lkml.org/lkml/2021/8/18/1022");
    }

    match CtrlAnime::new(AnimeConfig::load()) {
        Ok(ctrl) => {
            let inner = Arc::new(Mutex::new(ctrl));

            let mut reload = CtrlAnimeReloader(inner.clone());
            reload
                .reload()
                .unwrap_or_else(|err| warn!("AniMe: {}", err));

            let zbus = CtrlAnimeZbus(inner.clone());
            zbus.add_to_server(&mut object_server);

            let task = CtrlAnimeTask::new(inner);
            thread::Builder::new().name("anime tasks".into()).spawn(
                move || -> Result<(), RogError> {
                    loop {
                        task.do_task()?;
                    }
                },
            )?;
        }
        Err(err) => {
            error!("AniMe control: {}", err);
        }
    }

    let laptop = LaptopLedData::get_data();
    let aura_config = AuraConfig::load(&laptop);
    match CtrlKbdLed::new(laptop, aura_config) {
        Ok(ctrl) => {
            let inner = Arc::new(Mutex::new(ctrl));

            let mut reload = CtrlKbdLedReloader(inner.clone());
            reload
                .reload()
                .unwrap_or_else(|err| warn!("Keyboard LED control: {}", err));

            CtrlKbdLedZbus::new(inner.clone()).add_to_server(&mut object_server);

            let task = CtrlKbdLedTask::new(inner);
            thread::Builder::new().name("keyboard tasks".into()).spawn(
                move || -> Result<(), RogError> {
                    loop {
                        task.do_task()?;
                    }
                },
            )?;
        }
        Err(err) => {
            error!("Keyboard control: {}", err);
        }
    }

    // Request dbus name after finishing initalizing all functions
    fdo_connection.request_name(DBUS_NAME, fdo::RequestNameFlags::ReplaceExisting.into())?;

    // Loop to check errors and iterate zbus server
    loop {
        if let Err(err) = object_server.try_handle_next() {
            error!("{}", err);
        }
    }
}
