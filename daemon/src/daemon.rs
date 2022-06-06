use std::env;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};

use ::zbus::Connection;
use daemon::ctrl_profiles::controller::CtrlProfileTask;
use log::LevelFilter;
use log::{error, info, warn};
use smol::Executor;

use daemon::ctrl_anime::config::AnimeConfig;
use daemon::ctrl_anime::zbus::CtrlAnimeZbus;
use daemon::ctrl_anime::*;
use daemon::ctrl_aura::config::AuraConfig;
use daemon::ctrl_aura::controller::{
    CtrlKbdLed, CtrlKbdLedReloader, CtrlKbdLedTask, CtrlKbdLedZbus,
};
use daemon::ctrl_charge::CtrlCharge;
use daemon::ctrl_profiles::config::ProfileConfig;
use daemon::ctrl_rog_bios::CtrlRogBios;
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

    let mut executor = Executor::new();

    smol::block_on(start_daemon(&mut executor))?;
    Ok(())
}

/// The actual main loop for the daemon
async fn start_daemon(executor: &mut Executor<'_>) -> Result<(), Box<dyn Error>> {
    let supported = SupportedFunctions::get_supported();
    print_board_info();
    println!("{}", serde_json::to_string_pretty(&supported)?);

    // Start zbus server
    let mut connection = Connection::system().await?;

    let config = Config::load();
    let config = Arc::new(Mutex::new(config));

    supported.add_to_server(&mut connection).await;

    match CtrlRogBios::new(config.clone()) {
        Ok(mut ctrl) => {
            // Do a reload of any settings
            ctrl.reload()
                .unwrap_or_else(|err| warn!("Battery charge limit: {}", err));
            // Then register to dbus server
            ctrl.add_to_server(&mut connection).await;
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
            ctrl.add_to_server(&mut connection).await;
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
                let task = CtrlProfileTask::new(tmp.clone());
                task.create_task(executor).await.ok();

                let task = ProfileZbus::new(tmp.clone());
                task.create_task(executor).await.ok();

                task.add_to_server(&mut connection).await;
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
            zbus.add_to_server(&mut connection).await;

            let task = CtrlAnimeTask::new(inner).await;
            task.create_task(executor).await.ok();
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

            CtrlKbdLedZbus::new(inner.clone())
                .add_to_server(&mut connection)
                .await;

            let task = CtrlKbdLedTask::new(inner);
            task.create_task(executor).await.ok();
        }
        Err(err) => {
            error!("Keyboard control: {}", err);
        }
    }

    // Request dbus name after finishing initalizing all functions
    connection.request_name(DBUS_NAME).await?;
    dbg!();
    loop {
        smol::block_on(executor.tick());
    }
}
