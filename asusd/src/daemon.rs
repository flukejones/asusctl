use std::env;
use std::error::Error;
use std::sync::Arc;

use ::zbus::export::futures_util::lock::Mutex;
use ::zbus::Connection;
use asusd::config::Config;
use asusd::ctrl_anime::config::AnimeConfig;
use asusd::ctrl_anime::trait_impls::CtrlAnimeZbus;
use asusd::ctrl_anime::CtrlAnime;
use asusd::ctrl_aura::manager::AuraManager;
use asusd::ctrl_fancurves::CtrlFanCurveZbus;
use asusd::ctrl_platform::CtrlPlatform;
use asusd::{print_board_info, start_tasks, CtrlTask, DBUS_NAME};
use config_traits::{StdConfig, StdConfigLoad2, StdConfigLoad3};
use log::{error, info};
use zbus::fdo::ObjectManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
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

    info!("       daemon v{}", asusd::VERSION);
    info!("    rog-anime v{}", rog_anime::VERSION);
    info!("     rog-aura v{}", rog_aura::VERSION);
    info!(" rog-profiles v{}", rog_profiles::VERSION);
    info!("rog-platform v{}", rog_platform::VERSION);

    start_daemon().await?;
    Ok(())
}

/// The actual main loop for the daemon
async fn start_daemon() -> Result<(), Box<dyn Error>> {
    // let supported = SupportedFunctions::get_supported();
    print_board_info();
    // println!("{:?}", supported.supported_functions());

    // Start zbus server
    let mut connection = Connection::system().await?;
    connection
        .object_server()
        .at("/org", ObjectManager)
        .await
        .unwrap();

    let config = Config::new().load();
    let cfg_path = config.file_path();
    let config = Arc::new(Mutex::new(config));

    // supported.add_to_server(&mut connection).await;

    match CtrlFanCurveZbus::new() {
        Ok(ctrl) => {
            let sig_ctx = CtrlFanCurveZbus::signal_context(&connection)?;
            start_tasks(ctrl, &mut connection, sig_ctx).await?;
        }
        Err(err) => {
            error!("FanCurves: {}", err);
        }
    }

    match CtrlPlatform::new(
        config.clone(),
        &cfg_path,
        CtrlPlatform::signal_context(&connection)?,
    ) {
        Ok(ctrl) => {
            let sig_ctx = CtrlPlatform::signal_context(&connection)?;
            start_tasks(ctrl, &mut connection, sig_ctx).await?;
        }
        Err(err) => {
            error!("CtrlPlatform: {}", err);
        }
    }

    match CtrlAnime::new(AnimeConfig::new().load()) {
        Ok(ctrl) => {
            let zbus = CtrlAnimeZbus(Arc::new(Mutex::new(ctrl)));
            let sig_ctx = CtrlAnimeZbus::signal_context(&connection)?;
            start_tasks(zbus, &mut connection, sig_ctx).await?;
        }
        Err(err) => {
            info!("AniMe control: {}", err);
        }
    }

    let _ = AuraManager::new(connection.clone()).await?;

    // Request dbus name after finishing initalizing all functions
    connection.request_name(DBUS_NAME).await?;

    loop {
        // This is just a blocker to idle and ensure the reator reacts
        connection.executor().tick().await;
    }
}
