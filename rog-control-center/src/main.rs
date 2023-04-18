use std::env::args;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use eframe::{IconData, NativeOptions};
use gumdrop::Options;
use log::{error, info, warn};
use rog_aura::aura_detection::{LaptopLedData, LedSupportFile};
use rog_aura::layouts::KeyLayout;
use rog_control_center::cli_options::CliStart;
use rog_control_center::config::Config;
use rog_control_center::error::Result;
use rog_control_center::startup_error::AppErrorShow;
use rog_control_center::system_state::SystemState;
use rog_control_center::tray::init_tray;
use rog_control_center::update_and_notify::{start_notifications, EnabledNotifications};
use rog_control_center::{
    get_ipc_file, on_tmp_dir_exists, print_versions, RogApp, RogDbusClientBlocking, SHOWING_GUI,
    SHOW_GUI,
};
use rog_platform::supported::SupportedFunctions;
use tokio::runtime::Runtime;

#[cfg(not(feature = "mocking"))]
const DATA_DIR: &str = "/usr/share/rog-gui/";
#[cfg(feature = "mocking")]
const DATA_DIR: &str = env!("CARGO_MANIFEST_DIR");
const BOARD_NAME: &str = "/sys/class/dmi/id/board_name";
const APP_ICON_PATH: &str = "/usr/share/icons/hicolor/512x512/apps/rog-control-center.png";

fn main() -> Result<()> {
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

    let mut logger = env_logger::Builder::new();
    logger
        .parse_default_env()
        .target(env_logger::Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();

    // start tokio
    let rt = Runtime::new().expect("Unable to create Runtime");
    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        vsync: true,
        decorated: true,
        transparent: false,
        min_window_size: Some(egui::vec2(960.0, 670.0)),
        max_window_size: Some(egui::vec2(960.0, 670.0)),
        run_and_return: true,
        icon_data: Some(load_icon()),
        ..Default::default()
    };

    let (dbus, _) = RogDbusClientBlocking::new()
        .map_err(|e| {
            eframe::run_native(
                "ROG Control Center",
                native_options.clone(),
                Box::new(move |_| Box::new(AppErrorShow::new(e.to_string()))),
            )
            .map_err(|e| error!("{e}"))
            .ok();
        })
        .unwrap();

    let supported = match dbus.proxies().supported().supported_functions() {
        Ok(s) => s,
        Err(e) => {
            eframe::run_native(
                "ROG Control Center",
                native_options.clone(),
                Box::new(move |_| Box::new(AppErrorShow::new(e.to_string()))),
            )
            .map_err(|e| error!("{e}"))
            .ok();
            SupportedFunctions::default()
        }
    };

    // Startup
    let mut config = Config::load()?;
    let running_in_bg = Arc::new(AtomicBool::new(config.startup_in_background));

    if config.startup_in_background {
        config.run_in_background = true;
        let tmp = config.enabled_notifications.clone(); // ends up being a double clone, oh well.
        config.save(&tmp)?;
    }
    let enabled_notifications = EnabledNotifications::tokio_mutex(&config);

    // Find and load a matching layout for laptop
    let mut board_name = std::fs::read_to_string(BOARD_NAME).map_err(|e| {
        println!("DOH! {BOARD_NAME}, {e}");
        e
    })?;

    let mut led_support = LaptopLedData::get_data();

    let mut path = PathBuf::from(DATA_DIR);
    let mut layout_name = None;
    let mut layouts = Vec::new();
    if cli_parsed.board_name.is_some() || cli_parsed.layout_viewing {
        if cfg!(feature = "mocking") {
            path.pop();
            path.push("rog-aura");
            path.push("data");
        }
        layouts = KeyLayout::layout_files(path.clone()).unwrap();

        if let Some(name) = &cli_parsed.board_name {
            if let Some(modes) = LedSupportFile::load_from_config() {
                if let Some(data) = modes.matcher(name) {
                    led_support = data;
                }
            }
            board_name = name.clone();
            for layout in &layouts {
                if layout
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(&led_support.layout_name.to_lowercase())
                {
                    layout_name = Some(layout.clone());
                }
            }
        } else {
            board_name = "GQ401QM".to_owned();
        };

        if cli_parsed.layout_viewing {
            layout_name = Some(layouts[0].clone());
            board_name = layouts[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .split_once('_')
                .unwrap()
                .0
                .to_owned();
            led_support.layout_name = board_name.clone();
        }
    }

    let layout = KeyLayout::find_layout(led_support, path)
        .map_err(|e| {
            println!("DERP! , {e}");
        })
        .unwrap_or_else(|_| {
            warn!("Did not find a keyboard layout matching {board_name}");
            KeyLayout::default_layout()
        });

    // tmp-dir must live to the end of program life
    let _tmp_dir = match tempfile::Builder::new()
        .prefix("rog-gui")
        .rand_bytes(0)
        .tempdir()
    {
        Ok(tmp) => tmp,
        Err(_) => on_tmp_dir_exists().unwrap(),
    };

    let states = setup_page_state_and_notifs(
        layout_name,
        layout,
        layouts,
        &enabled_notifications,
        &config,
        &supported,
    )?;

    init_tray(supported, states.clone());

    loop {
        if !running_in_bg.load(Ordering::Acquire) {
            // blocks until window is closed
            start_app(states.clone(), native_options.clone())?;
        }

        let config = Config::load()?;
        if !config.run_in_background || cli_parsed.board_name.is_some() || cli_parsed.layout_viewing
        {
            break;
        }

        if config.run_in_background && !running_in_bg.load(Ordering::Acquire) {
            running_in_bg.store(true, Ordering::SeqCst);

            let running_in_bg = running_in_bg.clone();
            thread::spawn(move || {
                let mut buf = [0u8; 4];
                // blocks until it is read, typically the read will happen after a second
                // process writes to the IPC (so there is data to actually read)
                if get_ipc_file().unwrap().read(&mut buf).is_ok() && buf[0] == SHOW_GUI {
                    running_in_bg.store(false, Ordering::SeqCst);
                }
            });
        }
        // Prevent hogging CPU
        thread::sleep(Duration::from_millis(500));
    }

    // loop {
    //     // This is just a blocker to idle and ensure the reator reacts
    //     sleep(Duration::from_millis(1000)).await;
    // }
    Ok(())
}

fn setup_page_state_and_notifs(
    layout_testing: Option<PathBuf>,
    keyboard_layout: KeyLayout,
    keyboard_layouts: Vec<PathBuf>,
    enabled_notifications: &Arc<Mutex<EnabledNotifications>>,
    config: &Config,
    supported: &SupportedFunctions,
) -> Result<Arc<Mutex<SystemState>>> {
    let page_states = Arc::new(Mutex::new(SystemState::new(
        layout_testing,
        keyboard_layout,
        keyboard_layouts,
        enabled_notifications.clone(),
        supported,
    )?));

    start_notifications(config, &page_states, enabled_notifications)?;

    Ok(page_states)
}

fn start_app(states: Arc<Mutex<SystemState>>, native_options: NativeOptions) -> Result<()> {
    let mut ipc_file = get_ipc_file()?;
    ipc_file.write_all(&[SHOWING_GUI])?;
    eframe::run_native(
        "ROG Control Center",
        native_options,
        Box::new(move |cc| Box::new(RogApp::new(Config::load().unwrap(), states, cc).unwrap())),
    )?;
    Ok(())
}

/// Bah.. the icon dosn't work on wayland anyway, but we'll leave it in for now.
fn load_icon() -> IconData {
    let path = PathBuf::from(APP_ICON_PATH);
    let mut rgba = Vec::new();
    let mut height = 512;
    let mut width = 512;
    if path.exists() {
        if let Ok(data) = std::fs::read(path)
            .map_err(|e| error!("Error reading app icon: {e:?}"))
            .map_err(|e| error!("Error opening app icon: {e:?}"))
        {
            let data = std::io::Cursor::new(data);
            let decoder = png_pong::Decoder::new(data).unwrap().into_steps();
            let png_pong::Step { raster, delay: _ } = decoder.last().unwrap().unwrap();

            if let png_pong::PngRaster::Rgba8(ras) = raster {
                rgba = ras.as_u8_slice().to_vec();
                width = ras.width();
                height = ras.height();
                info!("Loaded app icon. Not actually supported in Wayland yet");
            }
        }
    } else {
        error!("Missing {APP_ICON_PATH}");
    }

    IconData {
        height,
        width,
        rgba,
    }
}

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
