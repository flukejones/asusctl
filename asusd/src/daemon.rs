use std::env;
use std::error::Error;
use std::sync::Arc;

use ::zbus::export::futures_util::lock::Mutex;
use ::zbus::Connection;
use asusd::asus_armoury::start_attributes_zbus;
use asusd::aura_manager::DeviceManager;
use asusd::config::Config;
use asusd::ctrl_fancurves::CtrlFanCurveZbus;
use asusd::ctrl_platform::CtrlPlatform;
use asusd::{print_board_info, start_tasks, CtrlTask, DBUS_NAME};
use config_traits::{StdConfig, StdConfigLoad1};
use log::{error, info};
use rog_platform::platform::RogPlatform;
use rog_platform::power::AsusPower;
use zbus::fdo::ObjectManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // console_subscriber::init();
    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let is_service = match env::var_os("IS_SERVICE") {
        Some(val) => val == "1",
        None => true
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
    info!("    rog-slash v{}", rog_slash::VERSION);
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
    let mut server = Connection::system().await?;
    server.object_server().at("/", ObjectManager).await.unwrap();

    let config = Config::new().load();
    let cfg_path = config.file_path();
    let config = Arc::new(Mutex::new(config));

    // supported.add_to_server(&mut connection).await;
    let platform = RogPlatform::new()?; // TODO: maybe needs async mutex?
    let power = AsusPower::new()?; // TODO: maybe needs async mutex?
    start_attributes_zbus(&server, platform, power, config.clone()).await?;

    match CtrlFanCurveZbus::new() {
        Ok(ctrl) => {
            let sig_ctx = CtrlFanCurveZbus::signal_context(&server)?;
            start_tasks(ctrl, &mut server, sig_ctx).await?;
        }
        Err(err) => {
            error!("FanCurves: {}", err);
        }
    }

    match CtrlPlatform::new(
        config.clone(),
        &cfg_path,
        CtrlPlatform::signal_context(&server)?
    ) {
        Ok(ctrl) => {
            let sig_ctx = CtrlPlatform::signal_context(&server)?;
            start_tasks(ctrl, &mut server, sig_ctx).await?;
        }
        Err(err) => {
            error!("CtrlPlatform: {}", err);
        }
    }

    let _ = DeviceManager::new(server.clone()).await?;

    // Request dbus name after finishing initalizing all functions
    server.request_name(DBUS_NAME).await?;

    info!("Startup success, begining dbus server loop");
    loop {
        // This is just a blocker to idle and ensure the reator reacts
        server.executor().tick().await;
    }
}
