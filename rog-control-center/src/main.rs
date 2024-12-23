use std::borrow::BorrowMut;
use std::env::args;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

use config_traits::{StdConfig, StdConfigLoad1};
use dmi_id::DMIID;
use gumdrop::Options;
use log::{info, warn, LevelFilter};
use rog_control_center::cli_options::CliStart;
use rog_control_center::config::Config;
use rog_control_center::error::Result;
use rog_control_center::notify::start_notifications;
use rog_control_center::slint::ComponentHandle;
use rog_control_center::tray::init_tray;
use rog_control_center::ui::setup_window;
use rog_control_center::zbus::{
    AppState, ROGCCZbus, ROGCCZbusProxyBlocking, ZBUS_IFACE, ZBUS_PATH,
};
use rog_control_center::{print_versions, MainWindow};
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() -> Result<()> {
    let mut logger = env_logger::Builder::new();
    logger
        .filter_level(LevelFilter::Warn)
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format_timestamp(None)
        .init();

    // Try to open a proxy and check for app state first
    {
        let user_con = zbus::blocking::Connection::session()?;
        if let Ok(proxy) = ROGCCZbusProxyBlocking::new(&user_con) {
            if let Ok(state) = proxy.state() {
                info!("App is already running: {state:?}, opening the window");
                // if there is a proxy connection assume the app is already running
                proxy.set_state(AppState::MainWindowShouldOpen)?;
                std::process::exit(0);
            }
        }
    }

    // version checks
    let self_version = env!("CARGO_PKG_VERSION");
    let zbus_con = zbus::blocking::Connection::system()?;
    let platform_proxy = rog_dbus::zbus_platform::PlatformProxyBlocking::new(&zbus_con)?;
    let asusd_version = platform_proxy.version().unwrap();
    if asusd_version != self_version {
        println!("Version mismatch: asusctl = {self_version}, asusd = {asusd_version}");
        return Ok(());
    }

    // start tokio
    let rt = Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    #[cfg(feature = "tokio-debug")]
    console_subscriber::init();

    let state_zbus = ROGCCZbus::new();
    let app_state = state_zbus.clone_state();
    let _conn = zbus::connection::Builder::session()?
        .name(ZBUS_IFACE)?
        .serve_at(ZBUS_PATH, state_zbus)?
        .build()
        .await
        .map_err(|err| {
            warn!("{}: add_to_server {}", ZBUS_PATH, err);
            err
        })?;

    let dmi = DMIID::new().unwrap_or_default();
    let board_name = dmi.board_name;
    let prod_family = dmi.product_family;
    info!("Running on {board_name}, product: {prod_family}");
    let is_rog_ally = prod_family == "RC71L";

    let args: Vec<String> = args().skip(1).collect();

    let cli_parsed = match CliStart::parse_args_default(&args) {
        Ok(p) => p,
        Err(err) => {
            panic!("source {}", err);
        }
    };

    if do_cli_help(&cli_parsed) {
        return Ok(());
    }

    let supported_properties = platform_proxy.supported_properties().unwrap_or_default();

    // Startup
    let mut config = Config::new().load();
    if cli_parsed.fullscreen {
        config.start_fullscreen = true;
        if cli_parsed.width_fullscreen != 0 {
            config.fullscreen_width = cli_parsed.width_fullscreen;
        }
        if cli_parsed.height_fullscreen != 0 {
            config.fullscreen_height = cli_parsed.height_fullscreen;
        }
    } else if cli_parsed.windowed {
        config.start_fullscreen = false;
    }

    if is_rog_ally {
        config.notifications.enabled = false;
        config.enable_tray_icon = false;
        config.run_in_background = false;
        config.startup_in_background = false;
    }

    if config.startup_in_background {
        config.run_in_background = true;
    }
    config.write();

    let enable_tray_icon = config.enable_tray_icon;
    let startup_in_background = config.startup_in_background;
    let config = Arc::new(Mutex::new(config));

    start_notifications(config.clone(), &rt)?;

    if enable_tray_icon {
        init_tray(supported_properties, config.clone());
    }

    thread_local! { pub static UI: std::cell::RefCell<Option<MainWindow>> = Default::default()};
    // i_slint_backend_selector::with_platform(|_| Ok(())).unwrap();

    if !startup_in_background {
        if let Ok(mut lock) = app_state.lock() {
            *lock = AppState::MainWindowShouldOpen;
        }
    }

    if std::env::var("RUST_TRANSLATIONS").is_ok() {
        // don't care about content
        log::debug!("---- Using local-dir translations");
        slint::init_translations!("/usr/share/locale/");
    } else {
        log::debug!("Using system installed translations");
        slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/translations/"));
    }

    thread::spawn(move || {
        let mut state = AppState::StartingUp;
        loop {
            // save as a var, don't hold the lock the entire time or deadlocks happen
            if let Ok(lock) = app_state.lock() {
                state = *lock;
            }

            if state == AppState::MainWindowShouldOpen {
                if let Ok(mut lock) = app_state.lock() {
                    *lock = AppState::MainWindowOpen;
                }
                sleep(Duration::from_millis(50));

                let config_copy = config.clone();
                let app_state_copy = app_state.clone();
                slint::invoke_from_event_loop(move || {
                    UI.with(|ui| {
                        let app_state_copy = app_state_copy.clone();
                        let mut ui = ui.borrow_mut();
                        if let Some(ui) = ui.as_mut() {
                            ui.window().show().unwrap();
                            ui.window().on_close_requested(move || {
                                if let Ok(mut lock) = app_state_copy.lock() {
                                    *lock = AppState::MainWindowClosed;
                                }
                                slint::CloseRequestResponse::HideWindow
                            });
                        } else {
                            let newui = setup_window(config_copy);
                            newui.window().show().unwrap();
                            newui.window().on_close_requested(move || {
                                if let Ok(mut lock) = app_state_copy.lock() {
                                    *lock = AppState::MainWindowClosed;
                                }
                                slint::CloseRequestResponse::HideWindow
                            });
                            ui.replace(newui);
                        }
                    });
                })
                .unwrap();
            } else if state == AppState::QuitApp {
                slint::quit_event_loop().unwrap();
                exit(0);
            } else if state != AppState::MainWindowOpen {
                if let Ok(lock) = config.lock() {
                    if !lock.run_in_background {
                        slint::quit_event_loop().unwrap();
                        exit(0);
                    }
                }

                slint::invoke_from_event_loop(move || {
                    UI.with(|ui| {
                        let mut ui = ui.take();
                        if let Some(ui) = ui.borrow_mut() {
                            ui.window().hide().unwrap();
                        }
                    });
                })
                .unwrap();
            }
        }
    });

    slint::run_event_loop_until_quit().unwrap();
    rt.shutdown_background();
    Ok(())
}

// /// Bah.. the icon dosn't work on wayland anyway, but we'll leave it in for
// now. fn load_icon() -> IconData {
//     let path = PathBuf::from(APP_ICON_PATH);
//     let mut rgba = Vec::new();
//     let mut height = 512;
//     let mut width = 512;
//     if path.exists() {
//         if let Ok(data) = std::fs::read(path)
//             .map_err(|e| error!("Error reading app icon: {e:?}"))
//             .map_err(|e| error!("Error opening app icon: {e:?}"))
//         {
//             let data = std::io::Cursor::new(data);
//             let decoder = png_pong::Decoder::new(data).unwrap().into_steps();
//             let png_pong::Step { raster, delay: _ } =
// decoder.last().unwrap().unwrap();

//             if let png_pong::PngRaster::Rgba8(ras) = raster {
//                 rgba = ras.as_u8_slice().to_vec();
//                 width = ras.width();
//                 height = ras.height();
//                 info!("Loaded app icon. Not actually supported in Wayland
// yet");             }
//         }
//     } else {
//         error!("Missing {APP_ICON_PATH}");
//     }

//     IconData {
//         height,
//         width,
//         rgba
//
//
// /     }
// }

fn do_cli_help(parsed: &CliStart) -> bool {
    if parsed.help {
        println!("{}", CliStart::usage());
        println!();
        if let Some(cmdlist) = CliStart::command_list() {
            let commands: Vec<String> = cmdlist.lines().map(|s| s.to_owned()).collect();
            for command in &commands {
                println!("{}", command);
            }
        }
    }

    if parsed.version {
        print_versions();
        println!();
    }

    parsed.help
}

pub fn get_layout_path(path: &Path, layout_name: &str) -> PathBuf {
    let mut data_path = PathBuf::from(path);
    let layout_file = format!("{}_US.ron", layout_name);
    data_path.push("layouts");
    data_path.push(layout_file);
    data_path
}
