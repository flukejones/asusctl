use std::env;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use ::zbus::export::futures_util::lock::Mutex;
use ::zbus::Connection;
use daemon::ctrl_anime::CtrlAnime;
use log::LevelFilter;
use log::{error, info, warn};
use tokio::time::sleep;

use daemon::ctrl_anime::{config::AnimeConfig, trait_impls::CtrlAnimeZbus};
use daemon::ctrl_aura::{config::AuraConfig, controller::CtrlKbdLed, trait_impls::CtrlKbdLedZbus};
use daemon::ctrl_platform::CtrlPlatform;
use daemon::ctrl_power::CtrlPower;
use daemon::ctrl_profiles::{
    config::ProfileConfig, controller::CtrlPlatformProfile, trait_impls::ProfileZbus,
};
use daemon::laptops::LaptopLedData;
use daemon::{
    config::Config, ctrl_supported::SupportedFunctions, laptops::print_board_info, GetSupported,
};
use daemon::{CtrlTask, Reloadable, ZbusRun};
use rog_dbus::DBUS_NAME;
use rog_profiles::Profile;

static PROFILE_CONFIG_PATH: &str = "/etc/asusd/profile.conf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    info!("rog-platform v{}", rog_platform::VERSION);

    start_daemon().await?;
    Ok(())
}

/// The actual main loop for the daemon
async fn start_daemon() -> Result<(), Box<dyn Error>> {
    let supported = SupportedFunctions::get_supported();
    print_board_info();
    println!("{}", serde_json::to_string_pretty(&supported)?);

    // Start zbus server
    let mut connection = Connection::system().await?;

    let config = Config::load();
    let config = Arc::new(Mutex::new(config));

    supported.add_to_server(&mut connection).await;

    match CtrlPlatform::new(config.clone()) {
        Ok(ctrl) => {
            start_tasks(ctrl, &mut connection).await?;
        }
        Err(err) => {
            error!("CtrlPlatform: {}", err);
        }
    }

    match CtrlPower::new(config.clone()) {
        Ok(ctrl) => {
            start_tasks(ctrl, &mut connection).await?;
        }
        Err(err) => {
            error!("CtrlPower: {}", err);
        }
    }

    if Profile::is_platform_profile_supported() {
        let profile_config = ProfileConfig::load(PROFILE_CONFIG_PATH.into());
        match CtrlPlatformProfile::new(profile_config) {
            Ok(ctrl) => {
                let zbus = ProfileZbus(Arc::new(Mutex::new(ctrl)));
                start_tasks(zbus, &mut connection).await?;
            }
            Err(err) => {
                error!("Profile control: {}", err);
            }
        }
    } else {
        warn!("platform_profile support not found");
    }

    match CtrlAnime::new(AnimeConfig::load()) {
        Ok(ctrl) => {
            let zbus = CtrlAnimeZbus(Arc::new(Mutex::new(ctrl)));
            start_tasks(zbus, &mut connection).await?;
        }
        Err(err) => {
            info!("AniMe control: {}", err);
        }
    }

    let laptop = LaptopLedData::get_data();
    let aura_config = AuraConfig::load(&laptop);
    match CtrlKbdLed::new(laptop, aura_config) {
        Ok(ctrl) => {
            let zbus = CtrlKbdLedZbus(Arc::new(Mutex::new(ctrl)));
            start_tasks(zbus, &mut connection).await?;
        }
        Err(err) => {
            error!("Keyboard control: {}", err);
        }
    }

    // Request dbus name after finishing initalizing all functions
    connection.request_name(DBUS_NAME).await?;

    loop {
        // This is just a blocker to idle and ensure the reator reacts
        sleep(Duration::from_millis(1000)).await;
    }
}

async fn start_tasks<T>(mut zbus: T, connection: &mut Connection) -> Result<(), Box<dyn Error>>
where
    T: ZbusRun + Reloadable + CtrlTask + Clone,
{
    let task = zbus.clone();

    zbus.reload()
        .await
        .unwrap_or_else(|err| warn!("Controller error: {}", err));
    zbus.add_to_server(connection).await;

    task.create_tasks(CtrlKbdLedZbus::signal_context(&connection)?)
        .await
        .ok();
    Ok(())
}
