use daemon::ctrl_anime::config::AnimeConfig;
use daemon::ctrl_anime::zbus::CtrlAnimeZbus;
use daemon::ctrl_anime::*;
use daemon::ctrl_aura::config::AuraConfig;
use daemon::ctrl_aura::controller::{
    CtrlKbdLed, CtrlKbdLedReloader, CtrlKbdLedTask, CtrlKbdLedZbus,
};
use daemon::ctrl_charge::CtrlCharge;
use daemon::ctrl_profiles::config::ProfileConfig;
use daemon::ctrl_profiles::controller::CtrlPlatformTask;
use daemon::{
    config::Config, ctrl_supported::SupportedFunctions, laptops::print_board_info, GetSupported,
};
use daemon::{
    ctrl_profiles::{controller::CtrlPlatformProfile, zbus::ProfileZbus},
    laptops::LaptopLedData,
};

use ::zbus::{fdo, Connection, ObjectServer};
use daemon::{CtrlTask, Reloadable, ZbusAdd};
use log::LevelFilter;
use log::{error, info, warn};
use rog_dbus::DBUS_NAME;
use std::env;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use daemon::ctrl_rog_bios::CtrlRogBios;

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
    println!("{}", serde_json::to_string_pretty(&supported).unwrap());

    // Collect tasks for task thread
    let mut tasks: Vec<Box<dyn CtrlTask + Send>> = Vec::new();
    // Start zbus server
    let connection = Connection::new_system()?;
    fdo::DBusProxy::new(&connection)?
        .request_name(DBUS_NAME, fdo::RequestNameFlags::ReplaceExisting.into())?;
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

    let profile_config = Arc::new(Mutex::new(ProfileConfig::load(PROFILE_CONFIG_PATH.into())));
    match CtrlPlatformProfile::new(profile_config.clone()) {
        Ok(mut ctrl) => {
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Profile control: {}", err));

            let tmp = Arc::new(Mutex::new(ctrl));
            ProfileZbus::new(tmp).add_to_server(&mut object_server);

            tasks.push(Box::new(CtrlPlatformTask::new(profile_config)));
        }
        Err(err) => {
            error!("Profile control: {}", err);
        }
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

            tasks.push(Box::new(CtrlAnimeTask::new(inner)));
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
            tasks.push(Box::new(task));
        }
        Err(err) => {
            error!("Keyboard control: {}", err);
        }
    }

    // TODO: implement messaging between threads to check fails

    // Run tasks
    let handle = std::thread::Builder::new()
        .name("asusd watch".to_string())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            for ctrl in tasks.iter() {
                ctrl.do_task()
                    .map_err(|err| {
                        warn!("do_task error: {}", err);
                    })
                    .ok();
            }
        });

    // Run zbus server
    // object_server
    //     .with(
    //         &ObjectPath::from_str_unchecked("/org/asuslinux/Charge"),
    //         |obj: &CtrlCharge| {
    //             let x = obj.limit();
    //             obj.notify_charge(x as u8)
    //         },
    //     )
    //     .map_err(|err| {
    //         warn!("object_server notify_charge error: {}", err);
    //     })
    //     .ok();

    // Loop to check errors and iterate zbus server
    loop {
        if let Err(err) = &handle {
            error!("{}", err);
        }
        if let Err(err) = object_server.try_handle_next() {
            error!("{}", err);
        }
    }
}
